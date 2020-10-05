require_relative "../shared.lib/formatting_helpers"
require_relative "../shared.lib/operand_types"
require_relative "instructions_code"

class TestTemplatesGenerator
  include FormattingHelpers
  include OperandTypes

  def initialize
    @buffer = StringIO.new
  end

  # Creates tests with the form:
  #
  #     context "INC A (0x3C)" {
  #         it "without conditional flag modifications" {
  #             // ...
  #         }
  #
  #         it "with flag Z modified" {
  #             // ...
  #         }
  #
  # Originally, for instuctions that don't have conditional flag modifications, the context
  # was not created (the only test would carry that description) , however, this was a headache, and
  # it's perfectly ok to always have a context (from the consistency point of view, it's actually
  # better).
  #
  # The exact naming of the flag concepts is a bit too verbose, so we oversimplify, and use
  # "un/conditional".
  #
  def add_code!(opcode, instruction, instruction_encoded, opcode_data, instruction_data, instruction_code)
    generate_header!(opcode, opcode_data, instruction, instruction_data)

    generate_unconditional_test!(opcode, opcode_data, instruction_data, instruction_code)
    generate_conditional_test!(opcode, opcode_data, instruction_data, instruction_code)

    generate_closure!(instruction_data)
  end

  def code
    @buffer.string
  end

  private

  # "Header": Context (optional), and main test method.
  #
  # `context`, or `it` depending on flags being changed or not
  #
  def generate_header!(opcode, opcode_data, instruction, instruction_data)
    operand_names = opcode_data.fetch("operands")
    operand_types = instruction_data.fetch("operand_types")

    non_immediate_operands =
      operand_names
        .zip(operand_types)
        .select { |_, type| ![IMMEDIATE_OPERAND_8, IMMEDIATE_OPERAND_16].include?(type) }
        .map(&:first)

    operand_names_description = ": #{non_immediate_operands.join(", ")}" if non_immediate_operands.size > 0

    @buffer.print <<-RUST
            context "#{instruction} [0x#{opcode}#{operand_names_description}]" {
    RUST
  end

  def generate_unconditional_test!(opcode, opcode_data, instruction_data, instruction_code)
    title = "without conditional flag modifications"

    flag_data = instruction_data.fetch("flags_data")

    unconditional_flags = flag_data.select { |_, state| state == "0" || state == "1" }
    boolean = {"0" => "false", "1" => "true"}
    reverse = {"0" => "1", "1" => "0"}

    flags_preset = unconditional_flags.map do |flag, state|
      "cpu.set_flag(Flag::#{flag.downcase}, #{boolean[reverse[state]]});"
    end

    flag_expectations = unconditional_flags.map do |flag, state|
      "#{flag.downcase}f => #{state},"
    end

    generate_test_body!(opcode, opcode_data, instruction_data, instruction_code, title, InstructionsCode::BASE, flags_preset, flag_expectations)
  end

  def generate_conditional_test!(opcode, opcode_data, instruction_data, instruction_code)
    flag_data = instruction_data.fetch("flags_data")

    conditional_flags = flag_data.select { |_, state| state == "*" }

    conditional_flags.each do |flag, _|
      @buffer.puts

      # In this case, the presets/expectations are in the metadata.
      #
      title = "with flag #{flag} modified"
      flags_preset = []
      flag_expectations = []

      generate_test_body!(opcode, opcode_data, instruction_data, instruction_code, title, flag, flags_preset, flag_expectations)
    end
  end

  def generate_test_body!(opcode, opcode_data, instruction_data, instruction_code, title, flag_test_prefix, flags_preset, flag_expectations)
    testing_block = instruction_code.fetch(:testing)
    opcode_operands = opcode_data.fetch("operands")

    flag_tests_data =
      testing_block
      .(*opcode_operands)
      .select { |key, _| key.to_s.start_with?(/#{flag_test_prefix}\b/) }

    if flag_tests_data.empty?
      raise "No testing metadata found for opcode 0x#{opcode}, with flag prefix #{flag_test_prefix.inspect}"
    end

    flag_tests_data.each do |flag_test_key, extra_instruction_bytes: nil, presets: nil, expectations:|
      if flag_test_key != flag_test_prefix
        suffix_specification = flag_test_key.sub(flag_test_prefix, '')
      end

      @buffer.puts <<-RUST
                it "#{title}#{suffix_specification}" {
      RUST

      extra_instruction_bytes_str = extra_instruction_bytes.to_a.map { |byte| ", #{hex(byte)}" }.join

      @buffer.puts <<-RUST
                    let instruction_bytes = [0x#{opcode}#{extra_instruction_bytes_str}];

      RUST

      presets = "cpu[Reg16::PC] = 0x21;\n#{presets}"

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

      instruction_size = instruction_data.fetch("instruction_size")
      start_pc = 0x21
      end_pc = start_pc + instruction_size
      cycles = instruction_data.fetch("cycles")

      pc_expectation = "PC => #{hex(end_pc)},"

      all_expectations = expectations.to_s.lines.push(pc_expectation).concat(flag_expectations)

      # Sorting is mandated by the macro.
      #
      all_expectations = all_expectations.sort_by do |expectation|
        case expectation
        when /^[A-Z]/
          -1
        when /^.f/
          0
        when /^mem/
          1
        else
          raise "Unexpected expectation: #{expectation}"
        end
      end

      all_expectations.each do |expectation|
        @buffer.puts "                        #{expectation}"
      end

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
