require 'json'
require 'open-uri'

require_relative "cpu_decoding_template_generator"
require_relative "cpu_execution_templates_generator"
require_relative "templates_generator"
require_relative "instructions_code"
require_relative "test_templates_generator"

class CpuTemplatesGenerator
  OPCODES_ADDRESS = 'https://gbdev.io/gb-opcodes/Opcodes.json'

  DECODING_REPLACEMENT_START_PATTERN = '// __OPCODES_DECODING_REPLACEMENT_START__'
  DECODING_REPLACEMENT_END_PATTERN = '// __OPCODES_DECODING_REPLACEMENT_END__'
  EXECUTION_REPLACEMENT_START_PATTERN = '// __OPCODES_EXECUTION_REPLACEMENT_START__'
  EXECUTION_REPLACEMENT_END_PATTERN = '// __OPCODES_EXECUTION_REPLACEMENT_END__'
  TESTS_REPLACEMENT_START_PATTERN = '// __TESTS_REPLACEMENT_START__'
  TESTS_REPLACEMENT_END_PATTERN = '// __TESTS_REPLACEMENT_END__'

  def initialize(instructions_file, cpu_file, tests_file)
    @instructions_file = instructions_file
    @cpu_file = cpu_file
    @tests_file = tests_file
  end

  def execute(only_opcodes: [])
    instructions_data = JSON.parse(IO.read(@instructions_file))
    cpu_decoding_code, cpu_execution_code, tests_code = generate_templates(instructions_data, only_opcodes: only_opcodes)
    insert_content_in_source_files(cpu_decoding_code, cpu_execution_code, tests_code)
  end

  def download_json_page_content
    page_content = open(OPCODES_ADDRESS).read
    prettified_data = JSON.pretty_generate(JSON.parse(page_content))
    IO.write(@instructions_file, prettified_data)
  end

  def find_and_read_json_page_content
    IO.read(@instructions_file)
  end

  def generate_templates(instructions_data, only_opcodes:)
    decoding_generator = CpuDecodingTemplateGenerator.new
    execution_generator = CpuExecutionTemplatesGenerator.new
    tests_generator = TestTemplatesGenerator.new

    excess_instructions_code = InstructionsCode::INSTRUCTIONS_CODE.keys - instructions_data.keys

    if excess_instructions_code.size > 0
      raise "Excess instructions code: #{excess_instructions_code}"
    end

    instructions_data.each do |instruction, instruction_data|
      instruction_code = InstructionsCode::INSTRUCTIONS_CODE[instruction]

      if instruction_code.nil?
        puts "WRITEME: #{instruction}" unless only_opcodes.size > 0
        next
      end

      instruction_encoded = instruction
        .gsub(/\((\w+)\)/, 'I\1')               # indirect: `(HL)` -> `IHL`
        .gsub(/,? /, "_")

      opcodes_data = instruction_data.fetch("opcodes")
      only_opcodes = only_opcodes.map(&:upcase)

      if only_opcodes.size > 0
        opcodes_data = opcodes_data.select { |opcode, _| only_opcodes.include?(opcode.upcase) }
      end

      opcodes_data.each do |opcode, opcode_data|
        decoding_generator.add_code!(opcode, instruction_encoded, opcode_data, instruction_data)
        tests_generator.add_code!(opcode, instruction, instruction_encoded, opcode_data, instruction_data, instruction_code)
      end

      if opcodes_data.size > 0
        execution_generator.add_code!(instruction_encoded, instruction_data, instruction_code)
      end
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
end
