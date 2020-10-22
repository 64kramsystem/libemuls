require_relative "../shared.lib/formatting_helpers"
require_relative "../shared.lib/operand_types"
require_relative "instructions_code"

class TestTemplatesGenerator
  include FormattingHelpers
  include OperandTypes

  ORDERED_FLAGS = 'znhc'.chars.each_with_index.to_h

  def initialize
    @buffer = StringIO.new
  end

  def add_code!(opcode, instruction, instruction_encoded, opcode_data, instruction_data, instruction_code)
    generate_header!(opcode, opcode_data, instruction, instruction_data)

    if is_jump_instruction?(instruction_data)
      generate_jump_tests!(opcode, opcode_data, instruction_data, instruction_code)
    else
      generate_base_test!(opcode, opcode_data, instruction_data, instruction_code)
      generate_flag_tests!(opcode, opcode_data, instruction_data, instruction_code)
    end

    generate_closure!(instruction_data)
  end

  def code
    @buffer.string
  end

  private

  def generate_header!(opcode, opcode_data, instruction, instruction_data)
    prefix_value = "0x#{instruction_data.fetch("prefix")} " if instruction_data.key?("prefix")

    operand_names = opcode_data.fetch("operands")
    operand_types = instruction_data.fetch("operand_types")

    non_immediate_operands =
      operand_names
        .zip(operand_types)
        .select { |_, type| ![IMMEDIATE_OPERAND_8, IMMEDIATE_OPERAND_16].include?(type) }
        .map(&:first)

    operand_names_description = ": #{non_immediate_operands.join(", ")}" if non_immediate_operands.size > 0

    @buffer.print <<-RUST
            context "#{instruction} [#{prefix_value}0x#{opcode}#{operand_names_description}]" {
    RUST
  end

  def is_jump_instruction?(instruction_data)
    instruction_data.fetch("operand_types").include?("cc")
  end

  # This type of test is not strictly needed, as the test metadata could have logic to gather the input
  # values. However, with the assumption that two tests need to be generated (not/jump), a for each
  # would be required, which is excessive.
  #
  # Note that we generate test input parameters that have an inconsistency with the others, as they only
  # pass the jump data (flag, flag_value, condition_matching), while ignoring the other if present (e.g.
  # `nn`). Other tests don't use that anyway, so this is (barely) acceptable.
  #
  def generate_jump_tests!(opcode, opcode_data, instruction_data, instruction_code)
    jump_condition = opcode_data.fetch("operands")[0]

    # Sanity check.
    #
    raise if jump_condition !~ /^N?[CZ]$/

    @buffer.puts

    flag = jump_condition[-1].downcase
    flag_match_condition = jump_condition[0] != "N"

    [false, true].each do |flag_value|
      condition_matching = flag_value == flag_match_condition

      title = "with jump condition #{jump_condition}, jump #{"not " if !condition_matching}performed: "

      test_input_params = [flag, flag_value, condition_matching]

      generate_test_body!(opcode, opcode_data, instruction_data, instruction_code, title, test_input_params)
    end
  end

  def generate_base_test!(opcode, opcode_data, instruction_data, instruction_code)
    flags_set = instruction_data.fetch("flags_set")

    title = "without conditional flag modifications"

    test_input_params = opcode_data.fetch("operands")

    # Funny boolean class test: `state == !!state`.
    #
    unconditional_flags = flags_set.select { |_, state| state == true || state == false }

    flags_preset = unconditional_flags.map do |flag, state|
      "cpu.set_flag(Flag::#{flag.downcase}, #{!state});"
    end

    flag_expectations = unconditional_flags.map do |flag, state|
      "#{flag.downcase}f => #{state},"
    end

    generate_test_body!(
      opcode, opcode_data, instruction_data, instruction_code, title, test_input_params,
      test_key_prefix: InstructionsCode::BASE, flags_preset: flags_preset, flag_expectations: flag_expectations
    )
  end

  def generate_flag_tests!(opcode, opcode_data, instruction_data, instruction_code)
    flags_set = instruction_data.fetch("flags_set")

    conditional_flags = flags_set.select { |_, state| state != true && state != false }

    test_input_params = opcode_data.fetch("operands")

    conditional_flags.each do |flag, _|
      @buffer.puts

      title = "with flag #{flag} modified"

      # In this case, the flag presets/expectations are specified by the user.
      #
      generate_test_body!(
        opcode, opcode_data, instruction_data, instruction_code, title, test_input_params,
        test_key_prefix: flag
      )
    end
  end

  def generate_test_body!(opcode, opcode_data, instruction_data, instruction_code, title, test_input_params, test_key_prefix: //, flags_preset: [], flag_expectations: [])
    testing_block = instruction_code.fetch(:testing)

    tests_data =
      testing_block
      .(*test_input_params)
      .select { |key, _| key.to_s.start_with?(/#{test_key_prefix}\b/) }

    if tests_data.empty?
      prefix_value = "0x#{instruction_data.fetch("prefix")}/" if instruction_data.key?("prefix")
      raise "No testing metadata found for opcode #{prefix_value}0x#{opcode}, with flag prefix #{test_key_prefix.inspect}"
    end

    tests_data.each do |test_key, skip: nil, extra_instruction_bytes: nil, presets: nil, expectations: nil|
      next if skip

      if test_key != test_key_prefix
        title_suffix = test_key.sub(test_key_prefix, '')
      end

      @buffer.puts <<-RUST
                it "#{title}#{title_suffix}" {
      RUST

      prefix_value = "0x#{instruction_data.fetch("prefix")}, " if instruction_data.key?("prefix")
      extra_instruction_bytes_str = extra_instruction_bytes.to_a.map { |byte| ", #{hex(byte)}" }.join

      @buffer.puts <<-RUST
                    let instruction_bytes = [#{prefix_value}0x#{opcode}#{extra_instruction_bytes_str}];

      RUST

      presets ||= ""

      if !presets.include?("cpu[Reg16::PC] = ")
        presets = "cpu[Reg16::PC] = 0x21;\n#{presets}"
      end

      presets.each_line.map(&:strip).each do |preset_statement|
        if preset_statement.empty?
          @buffer.puts
        else
          @buffer.puts <<-RUST
                    #{preset_statement}
          RUST
        end
      end

      flags_preset.each do |preset_statement|
        @buffer.puts <<-RUST
                    #{preset_statement}
        RUST
      end

      @buffer.puts <<-RUST

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
      RUST

      all_expectations = expectations.to_s.lines

      if expectations !~ /PC =>/
        instruction_size = instruction_data.fetch("instruction_size")
        start_pc = 0x21
        end_pc = start_pc + instruction_size

        pc_expectation = "PC => #{hex(end_pc)},"

        all_expectations.push(pc_expectation)
      end

      all_expectations.concat(flag_expectations)

      # Sorting is mandated by the macro.
      #
      all_expectations = all_expectations.sort_by do |expectation|
        case expectation
        when /^[A-Z]/
          -10
        when /^.f/
          ORDERED_FLAGS.fetch(expectation[0])
        when /^mem/
          10
        else
          raise "Unexpected expectation: #{expectation}"
        end
      end

      all_expectations.each do |expectation|
        @buffer.puts "                        #{expectation}"
      end

      cycles = instruction_data.fetch("cycles")

      @buffer.puts <<-RUST
                        cycles: #{cycles}
                    );
                }
      RUST
    end
  end

  # Closing brace, with trailing space.
  #
  def generate_closure!(instruction_data)
    @buffer.puts <<-RUST
            }

    RUST
  end
end
