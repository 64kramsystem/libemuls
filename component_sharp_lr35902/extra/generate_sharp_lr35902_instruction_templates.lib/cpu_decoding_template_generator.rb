require_relative "formatting_helpers"
require_relative "instructions_data"

class CpuDecodingTemplateGenerator
  include FormattingHelpers
  include InstructionsData

  def initialize
    @buffer = StringIO.new
  end

  # While the instructions could be decoded algorithmically up to a certain extent, it's not worth
  # the complexity. For example, `PUSH nn` and `INC nn` use different bitmasks for the registers
  # (the 4th argument shares the same bitmask (base + 11), but point to different registers in the
  # two cases).

  def add_code!(opcode, opcode_family, opcode_data, instruction_data)
    opcode_hex = "%02X" % opcode
    operand_types = instruction_data.fetch(:operand_types)
    operand_names = opcode_data.fetch("operands").map { |data| data["name"] }

    generate_matcher_line!(opcode_hex, operand_types)
    generate_execution_method_call!(opcode_family, instruction_data, operand_types, operand_names)
    generate_closure!(opcode_data, opcode_hex)
  end

  def code
    @buffer.string
  end

  private

  # Matcher line. Example:
  #
  #     [0x36, immediate @ _] => {
  #
  def generate_matcher_line!(opcode_hex, operand_types)
    @buffer.print "            [0x#{opcode_hex}"

    # Registers don't use matcher bindings, and there can't be immediates on both sides, so we can
    # use simple testing logic.
    # Note that this makes it more complex to append a prefix to the variable name indicating the
    # source/destination nature.

    operand_type_types = operand_types.map(&:type)

    if operand_type_types.include?(IMMEDIATE_OPERAND_8)
      @buffer.print ", immediate @ _"
    elsif operand_type_types.include?(IMMEDIATE_OPERAND_16)
      @buffer.print ", immediate_low @ _, immediate_high @ _"
    end

    @buffer.puts "] => {"
  end

  def generate_execution_method_call!(opcode_family, instruction_data, operand_types, operand_names)
    operand_params = []

    operand_names.zip(operand_types).each do |operand_name, operand_type|
      case operand_type.type
      when REGISTER_OPERAND_8
        operand_params << "Register8::#{operand_name}"
      when REGISTER_OPERAND_16
        register_high, register_low = operand_name.chars
        operand_params.push("Register8::#{register_high}", "Register8::#{register_low}")
      when REGISTER_SP
        operand_params << "Register16::#{operand_name}"
      when IMMEDIATE_OPERAND_8
        operand_params << "immediate"
      when IMMEDIATE_OPERAND_16
        operand_params.push("immediate_high", "immediate_low")
      when nil
        # Do nothing
      else
        raise "Unexpected operand type: #{operand_type.type}"
      end
    end

    flag_params = instruction_data.fetch(:flags_data).each_with_object([]) do |(flag, state), flag_params|
      case state
      when "0", "1", flag
        flag_params << "Flag::#{flag.downcase}"
      when "-"
        # ignore
      else
        raise "Invalid flag state: #{state}"
      end
    end

    all_execution_params = [*operand_params, *flag_params].join(", ")

    @buffer.puts "                self.execute_#{opcode_family}(#{all_execution_params});"
  end

  # Closure (cycles and closing brace)
  #
  def generate_closure!(opcode_data, opcode_hex)
    cycles = opcode_data.fetch("cycles")[0] || raise("Missing #{opcode_hex} cycles!")

    @buffer.puts <<-RUST
                #{cycles}
            }
    RUST
  end
end
