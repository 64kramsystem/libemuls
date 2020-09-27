require_relative "formatting_helpers"
require_relative "instructions_data"

class TestTemplatesGenerator
  include FormattingHelpers
  include InstructionsData

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
  def add_code!(opcode, opcode_family, opcode_data, instruction_data)
    generate_header!(opcode, opcode_data, instruction_data)

    generate_unconditional_test!(opcode, opcode_data, instruction_data)
    generate_conditional_test!(opcode, opcode_data, instruction_data)

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
  def generate_header!(opcode, opcode_data, instruction_data)
    mnemonic = opcode_data.fetch("mnemonic")
    operand_names = opcode_data.fetch("operands")
    operand_types = instruction_data.fetch(:operand_types)

    operands_description = operand_names
      .map { |data| data.fetch("name") }
      .zip(operand_types)
      .map { |name, type| type.indirect ? "(#{name})" : name }
      .join(", ")

    @buffer.print <<-RUST
            context "#{mnemonic} #{operands_description} [#{hex(opcode)}]" {
    RUST
  end

  def generate_unconditional_test!(opcode, opcode_data, instruction_data)
    title = %Q[it "without conditional flag modifications" {]

    flag_data = instruction_data.fetch(:flags_data)

    unconditional_flags = flag_data.select { |_, state| state == "0" || state == "1" }
    boolean = {"0" => "false", "1" => "true"}
    reverse = {"0" => "1", "1" => "0"}

    flags_preset = unconditional_flags.map do |flag, state|
      "cpu.#{flag.downcase}f = #{boolean[reverse[state]]};"
    end

    flag_expectations = unconditional_flags.map do |flag, state|
      "#{flag.downcase}f => #{state},"
    end

    generate_test_body!(opcode, opcode_data, instruction_data, title, BASE, flags_preset, flag_expectations)
  end

  def generate_conditional_test!(opcode, opcode_data, instruction_data)
    flag_data = instruction_data.fetch(:flags_data)

    conditional_flags = flag_data.select { |flag, state| flag == state }

    conditional_flags.each do |flag, state|
      @buffer.puts

      # In this case, the presets/expectations are in the metadata.
      #
      title = %Q[it "with flag #{flag} modified" {]
      flags_preset = []
      flag_expectations = []

      generate_test_body!(opcode, opcode_data, instruction_data, title, flag, flags_preset, flag_expectations)
    end
  end

  def generate_test_body!(opcode, opcode_data, instruction_data, title, flag, flags_preset, flag_expectations)
    @buffer.puts <<-RUST
                #{title}
    RUST

    opcode_operands = opcode_data.fetch("operands").map { |data| data.fetch("name") }
    testing_block = instruction_data.fetch(:testing)

    extra_instruction_bytes, presets, expectations = begin
        testing_block
          .(*opcode_operands)
          .fetch(flag)
          .values_at(:extra_instruction_bytes, :presets, :expectations)
        rescue KeyError
          raise "Flag #{flag} testing metadata not found for opcode #{hex(opcode)}"
        end

    extra_instruction_bytes_str = extra_instruction_bytes.to_a.map { |byte| ", #{hex(byte)}" }.join

    @buffer.puts <<-RUST
                    let instruction_bytes = [#{hex(opcode)}#{extra_instruction_bytes_str}];

    RUST

    presets = "cpu.PC = 0x21;\n#{presets}"

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

    instruction_size = instruction_data.fetch(:instruction_size)
    start_pc = 0x21
    end_pc = start_pc + instruction_size
    cycles = opcode_data.fetch("cycles")[0]

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

  # Closing brace, with trailing space.
  #
  def generate_closure!(instruction_data)
    @buffer.puts <<-RUST
            }

    RUST
  end
end
