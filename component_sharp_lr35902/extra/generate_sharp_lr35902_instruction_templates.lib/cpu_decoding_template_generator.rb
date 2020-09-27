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
  #     [0x36, value @ _] => {
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

  # Execution method call. Some examples:
  #
  #     Self::execute_LD_r_n(&mut self.PC, &mut self.L, immediate);
  #
  #     let src_register = &self.L as *const u8;
  #     Self::execute_LD_r1_r2(&mut self.PC, &mut self.L, src_register);
  #
  #     Self::execute_INC_n(&mut self.PC, &mut self.A, &mut self.zf, &mut self.nf, &mut self.hf);
  #
  def generate_execution_method_call!(opcode_family, instruction_data, operand_types, operand_names)
    # Currently used only for raw pointers.
    #
    variable_assignments = []

    operand_params = []

    # When there is overlapping of register, for simplicity, we pass two raw pointers, even when not
    # necessary.
    # For the same reason, we always pass variables as mutable.
    #
    operand_names.zip(operand_types, ["dst", "src"]).each do |operand_name, operand_type, position|
      case operand_type.type
      when REGISTER_OPERAND_8
        if instruction_data.fetch(:any_shared_register)
          variable_assignments << "let #{position}_register = &mut self.#{operand_name} as *mut u8;"
          operand_params << "#{position}_register"
        else
          operand_params << "&mut self.#{operand_name}"
        end
      when REGISTER_OPERAND_16
        register_high, register_low = operand_name.chars
        operand_params.push("&mut self.#{register_high}", "&mut self.#{register_low}")
      when REGISTER_SP
        operand_params << "&mut self.#{operand_name}"
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
        flag_params << "&mut self.#{flag.downcase}f"
      when "-"
        # ignore
      else
        raise "Invalid flag state: #{state}"
      end
    end

    variable_assignments.each do |variable_assignment|
      @buffer.puts "                #{variable_assignment}"
    end

    internal_ram_param = \
      if operand_types.empty?
        []
      elsif operand_types[0].indirect
        ["&mut self.internal_ram"]
      elsif operand_types[1]&.indirect
        ["&self.internal_ram"]
      end

    all_execution_params = ["&mut self.PC", *internal_ram_param, *operand_params, *flag_params].join(", ")

    @buffer.puts "                Self::execute_#{opcode_family}(#{all_execution_params});"
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
