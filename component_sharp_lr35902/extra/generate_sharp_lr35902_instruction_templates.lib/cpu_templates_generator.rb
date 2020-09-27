require 'json'
require 'open-uri'

require_relative "cpu_decoding_template_generator"
require_relative "cpu_execution_templates_generator"
require_relative "cpu_templates_generator"
require_relative "instructions_data"
require_relative "operand_type"
require_relative "test_templates_generator"

class CpuTemplatesGenerator
  include InstructionsData

  OPCODES_ADDRESS = 'https://gbdev.io/gb-opcodes/Opcodes.json'

  DECODING_REPLACEMENT_START_PATTERN = '// __OPCODES_DECODING_REPLACEMENT_START__'
  DECODING_REPLACEMENT_END_PATTERN = '// __OPCODES_DECODING_REPLACEMENT_END__'
  EXECUTION_REPLACEMENT_START_PATTERN = '// __OPCODES_EXECUTION_REPLACEMENT_START__'
  EXECUTION_REPLACEMENT_END_PATTERN = '// __OPCODES_EXECUTION_REPLACEMENT_END__'
  TESTS_REPLACEMENT_START_PATTERN = '// __TESTS_REPLACEMENT_START__'
  TESTS_REPLACEMENT_END_PATTERN = '// __TESTS_REPLACEMENT_END__'

  def initialize(opcodes_file, cpu_file, tests_file)
    @opcodes_file = opcodes_file
    @cpu_file = cpu_file
    @tests_file = tests_file
  end

  def execute(only_opcode: nil)
    check_instructions_data
    download_json_page_content if !File.exists?(@opcodes_file)
    json_page_content = find_and_read_json_page_content
    json_data = JSON.parse(json_page_content)
    cpu_decoding_code, cpu_execution_code, tests_code = generate_templates(json_data, only_opcode: only_opcode)
    insert_content_in_source_files(cpu_decoding_code, cpu_execution_code, tests_code)
  end

  def check_instructions_data
    opcodes_count = INSTRUCTIONS_DATA
      .values
      .flat_map { |instruction_data| instruction_data.fetch(:opcodes) }
      .each_with_object({}) { |opcode, count| count[opcode] ||= 0; count[opcode] += 1 } # tally() on Ruby 2.7

    duplicated_opcodes = opcodes_count.select { |_, count| count > 1}

    if duplicated_opcodes.size > 0
      duplicated_opcodes_hex = duplicated_opcodes.keys.map { |opcode| hex(opcode) }
      raise "Found duplicated opcodes!: #{duplicated_opcodes_hex}"
    end
  end

  def download_json_page_content
    page_content = open(OPCODES_ADDRESS).read
    prettified_data = JSON.pretty_generate(JSON.parse(page_content))
    IO.write(@opcodes_file, prettified_data)
  end

  def find_and_read_json_page_content
    IO.read(@opcodes_file)
  end

  # For samples, see the corresponding `.md` document.
  #
  def generate_templates(json_data, only_opcode:)
    decoding_generator = CpuDecodingTemplateGenerator.new
    execution_generator = CpuExecutionTemplatesGenerator.new
    tests_generator = TestTemplatesGenerator.new

    # Instruction data is mutated. Due to the Proc stored, there's no trivial solution to perform a
    # deep clone; since the data is consumed only once, keeping the worklflow simple is an acceptable
    # solution.
    #
    INSTRUCTIONS_DATA.each do |opcode_family, instruction_data|
      opcodes = instruction_data.fetch(:opcodes)

      next if only_opcode && !opcodes.include?(only_opcode)

      opcode_family_encoded = opcode_family
        .gsub(/\((\w+)\)/, 'I\1')               # indirect:        `(HL)` -> `IHL`
        .gsub(/\(\$(\w+) \+ (\w)\)/, 'I_\1_\2') # Indirect+displ.: `($FF00 + C)` -> I_FF00_C
        .gsub(/,? /, "_")

      prefixed_json_entry = instruction_data.fetch(:prefixed) ? "prefixed" : "unprefixed"

      transform_opcode_data = instruction_data[:transform_opcode_data]

      opcodes.each do |opcode|
        next if only_opcode && opcode != only_opcode

        opcode_data = json_data.fetch(prefixed_json_entry).fetch(hex(opcode))

        transform_opcode_data&.(opcode_data)
      end

      # This procedure is essentially a denormalization of the (extra) instruction data.
      #
      extra_instruction_data = check_and_extract_instruction_data_from_opcodes_data(opcodes, json_data, prefixed_json_entry, instruction_data)
      instruction_data.merge!(extra_instruction_data)

      opcodes.each do |opcode|
        next if only_opcode && opcode != only_opcode

        opcode_data = json_data.fetch(prefixed_json_entry).fetch(hex(opcode))

        decoding_generator.add_code!(opcode, opcode_family_encoded, opcode_data, instruction_data)
        tests_generator.add_code!(opcode, opcode_family, opcode_data, instruction_data)
      end

      execution_generator.add_code!(opcode_family_encoded, instruction_data)
    end

    # Remove the trailing empty line, if any.
    #
    [decoding_generator, execution_generator, tests_generator].map { |generator| generator.code.sub(/^\n\Z/, '') }
  end

  def insert_content_in_source_files(cpu_decoding_code, cpu_execution_code, tests_code)
    cpu_file_content = IO.read(@cpu_file)

    new_cpu_file_content = cpu_file_content
      .sub(/^( *#{DECODING_REPLACEMENT_START_PATTERN}\n).*(^ *#{DECODING_REPLACEMENT_END_PATTERN}\n)/m, "\\1#{cpu_decoding_code}\\2")
      .sub(/^( *#{EXECUTION_REPLACEMENT_START_PATTERN}\n).*(^ *#{EXECUTION_REPLACEMENT_END_PATTERN}\n)/m, "\\1#{cpu_execution_code}\\2")

    IO.write(@cpu_file, new_cpu_file_content)

    tests_file_content = IO.read(@tests_file)

    new_tests_file_content = tests_file_content
      .sub(/^( *#{TESTS_REPLACEMENT_START_PATTERN}\n).*(^ *#{TESTS_REPLACEMENT_END_PATTERN})/m, "\\1#{tests_code}\\2")

    IO.write(@tests_file, new_tests_file_content)
  end

  ##################################################################################################
  # DATA MANIPULATION
  ##################################################################################################

  # Returns {flags_data, instruction_size, operand_types, any_shared_register}.
  #
  def check_and_extract_instruction_data_from_opcodes_data(opcodes, json_data, prefixed_json_entry, instruction_data)
    any_shared_register = false

    all_instruction_data = opcodes.map do |opcode|
      opcode_data = json_data.fetch(prefixed_json_entry).fetch(hex(opcode))

      flags_data = opcode_data.fetch("flags")
      instruction_size = opcode_data.fetch("bytes")
      operands_data = opcode_data.fetch("operands")

      registers_8bit_used, registers_16bit_used = [], []

      operand_types = operands_data.map do |operand_data|
        operand_name = operand_data.fetch("name")

        # WATCH OUT!! The JSON gets the "immediate" metadata wrong; the correct semantic is is the
        # opposite (see https://git.io/JU8JY), and it really refers to the indirection.
        #
        indirect = !operand_data.fetch("immediate")

        case operand_name
        when "d8"
          OperandType.new(IMMEDIATE_OPERAND_8, indirect)
        when "d16"
          OperandType.new(IMMEDIATE_OPERAND_16, indirect)
        when "a8"
          OperandType.new(IMMEDIATE_OPERAND_8, indirect)
        when "a16"
          OperandType.new(IMMEDIATE_OPERAND_16, indirect)
        when *REGISTERS_8B
          registers_8bit_used << operand_name
          OperandType.new(REGISTER_OPERAND_8, indirect)
        when *REGISTERS_16B
          registers_16bit_used << operand_name
          OperandType.new(REGISTER_OPERAND_16, indirect)
        when "SP"
          OperandType.new(REGISTER_SP, indirect)
        else
          debugger
          raise("Unsupported operand type for opcode %02X: #{operand_data}" % opcode)
        end
      end

      # St00pid simple logic.
      #
      any_shared_register ||= \
        registers_8bit_used.uniq.size != registers_8bit_used.size ||
        registers_8bit_used.any? { |register_8_bit| registers_16bit_used.any? { |register_16_bit| register_16_bit.include?(register_8_bit) } }

      {
        flags_data: flags_data,
        instruction_size: instruction_size,
        operand_types: operand_types,
      }
    end

    unique_instruction_data = all_instruction_data.uniq

    if unique_instruction_data.size == 1
      unique_instruction_data[0].merge(any_shared_register: any_shared_register)
    else
      debugger
      raise "Instruction data not unique for opcodes set: #{opcodes.map(&method(:hex))}"
    end
  end
end
