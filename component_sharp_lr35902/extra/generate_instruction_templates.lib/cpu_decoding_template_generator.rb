require_relative "../shared.lib/formatting_helpers"
require_relative "../shared.lib/operand_types"

class CpuDecodingTemplateGenerator
  include FormattingHelpers
  include OperandTypes

  def initialize
    @buffer = StringIO.new
  end

  # The instructions could be decoded algorithmically; the register formula is either `high_nibble + 1`
  # or `(high_nibble + 1) & 3`). In this case, from a metadata perspective, we'd:
  #
  # - split up instructions involving multiple registers, e.g. `LD r1, r2` becomes `LD A, r`, `LD B, r`
  #   and so on;
  # - encode the register indexing in the operand type (e.g. `r_m3` for registers to mask);
  # - drop the `operands` data.
  #
  # From the generator/cpu perspective, this introduces machine-specific code, however, like the carry
  # computations, it can be abstracted by defining and calling `Cpu` associated methods.
  #
  # The advantages are:
  #
  # - the "operands" data is not needed anymore in the instructions metadata;
  # - less entries in the decoder switch/case.
  #
  # However:
  #
  # - there will be considerably more entries (at least, for some intructions) in the instructions
  #   metadata;
  # - as a consequence, it won't be clear anymore what the reference should be; if the existing one
  #   should stay, a (temporary) intermediate one should be created, and the transformation code added;
  # - the generation won't know which register an opcode refers to, which is needed to generate
  #   descriptions (UTs), so it will require specific code.
  #
  # It needs to be verified if two-register instructions have still a regular register encoding; if
  # so, denormalization isn't needed.
  #
  def add_code!(opcode_hex, instruction_encoded, opcode_data, instruction_data)
    generate_matcher_line!(opcode_hex, instruction_data)
    generate_variables_assignment!(opcode_hex, instruction_data)
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

    prefix_value = "0x#{instruction_data.fetch("prefix")}, " if instruction_data.key?("prefix")
    @buffer.print "            [#{prefix_value}0x#{opcode_hex}"

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

  def generate_variables_assignment!(opcode_hex, instruction_data)
    operand_types = instruction_data.fetch("operand_types")
    opcode_data = instruction_data.fetch("opcodes").fetch(opcode_hex)
    operand_names = opcode_data.fetch("operands")

    operand_names.zip(operand_types).each do |operand_name, operand_type|
      case operand_type
      when IMMEDIATE_OPERAND_16
        # The reference is unnecessary, but we pass it for consistency with the 8-bit immediates.
        #
        @buffer.puts <<-RUST
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
        RUST
      when FLAG_OPERAND
        flag_condition = !operand_name.start_with?('N')

        @buffer.puts <<-RUST
                let flag_condition = #{flag_condition};
        RUST
      end
    end
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
        operand_params << "Reg16::#{operand_name}"
      when IMMEDIATE_OPERAND_8
        operand_params << "immediate"
      when IMMEDIATE_OPERAND_16
        operand_params << "immediate"
      when FLAG_OPERAND
        operand_params << "Flag::#{operand_name[-1].downcase}"
        operand_params << "flag_condition"
      else
        raise "Unexpected operand type: #{operand_type.type}"
      end
    end

    all_execution_params = operand_params.join(", ")

    @buffer.puts <<-RUST
                self.execute_#{instruction_encoded}(#{all_execution_params});
    RUST
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
