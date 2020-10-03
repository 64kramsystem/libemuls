require_relative "../shared.lib/formatting_helpers"
require_relative "../shared.lib/operand_types"

class CpuDecodingTemplateGenerator
  include FormattingHelpers
  include OperandTypes

  def initialize
    @buffer = StringIO.new
  end

  # While the instructions could be decoded algorithmically up to a certain extent, it's not worth
  # the complexity. For example, `PUSH nn` and `INC nn` use different bitmasks for the registers
  # (the 4th argument shares the same bitmask (base + 11), but point to different registers in the
  # two cases).

  def add_code!(opcode_hex, instruction_encoded, opcode_data, instruction_data)
    generate_matcher_line!(opcode_hex, instruction_data)
    generate_execution_method_call!(opcode_hex, instruction_encoded, instruction_data)
    generate_closure!(instruction_data)
  end

  def code
    @buffer.string
  end

  private

  # Matcher line. Example:
  #
  #     [0x36, immediate @ _] => {
  #
  def generate_matcher_line!(opcode_hex, instruction_data)
    operand_types = instruction_data.fetch("operand_types")

    @buffer.print "            [0x#{opcode_hex}"

    # Registers don't use matcher bindings, and there can't be immediates on both sides, so we can
    # use simple testing logic.
    # Note that this makes it more complex to append a prefix to the variable name indicating the
    # source/destination nature.

    if operand_types.include?(IMMEDIATE_OPERAND_8)
      @buffer.print ", immediate @ _"
    elsif operand_types.include?(IMMEDIATE_OPERAND_16)
      @buffer.print ", immediate_low @ _, immediate_high @ _"
    end

    @buffer.puts "] => {"
  end

  def generate_execution_method_call!(opcode_hex, instruction_encoded, instruction_data)
    operand_types = instruction_data.fetch("operand_types")
    opcode_data = instruction_data.fetch("opcodes").fetch(opcode_hex)
    operand_names = opcode_data.fetch("operands")

    operand_params = operand_names.zip(operand_types).each_with_object([]) do |(operand_name, operand_type), operand_params|
      case operand_type
      when REGISTER_OPERAND_8
        operand_params << "Reg8::#{operand_name}"
      when REGISTER_OPERAND_16
        operand_params.push("Reg16::#{operand_name}")
      when IMMEDIATE_OPERAND_8
        operand_params << "immediate"
      when IMMEDIATE_OPERAND_16
        operand_params.push("immediate_high", "immediate_low")
      when FLAG_OPERAND
        operand_params.push("Flag::#{operand_name[-1].downcase}")
      else
        raise "Unexpected operand type: #{operand_type.type}"
      end
    end

    all_execution_params = operand_params.join(", ")

    @buffer.puts "                self.execute_#{instruction_encoded}(#{all_execution_params});"
  end

  # Closure (cycles and closing brace)
  #
  def generate_closure!(instruction_data)
    cycles = instruction_data.fetch("cycles")

    @buffer.puts <<-RUST
                #{cycles}
            }
    RUST
  end
end
