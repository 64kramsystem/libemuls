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

    case operand_types.map(&:type)
    when []
      # Nothing
    when [REGISTER_OPERAND_8]
      # Nothing
    when [REGISTER_OPERAND_16]
      # Nothing
    when [REGISTER_OPERAND_8, IMMEDIATE_OPERAND_8]
      @buffer.print ", immediate @ _"
    when [REGISTER_OPERAND_8, REGISTER_OPERAND_8]
      # Nothing
    when [REGISTER_OPERAND_8, IMMEDIATE_OPERAND_16]
      @buffer.print ", immediate_low @ _, immediate_high @ _"
    when [REGISTER_OPERAND_8, REGISTER_OPERAND_16]
      # Nothing
    when [REGISTER_OPERAND_16, REGISTER_OPERAND_8]
      # Nothing
    when [REGISTER_OPERAND_16, IMMEDIATE_OPERAND_8]
      @buffer.print ", immediate @ _"
    when [IMMEDIATE_OPERAND_16, REGISTER_OPERAND_8]
      @buffer.print ", immediate_low @ _, immediate_high @ _"
    when [IMMEDIATE_OPERAND_8, REGISTER_OPERAND_8]
      @buffer.print ", immediate @ _"
    else
      # This is for safety; it's easy to miss a tuple.
      #
      raise "Unrecognized operand types: #{operand_types}"
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

    operand_params =
      case operand_types.map(&:type)
      when []
        # Nothing
      when [REGISTER_OPERAND_8]
        "&mut self.#{operand_names[0]}"
      when [REGISTER_OPERAND_16]
        register_high, register_low = operand_names[0].chars
        [
          "&self.#{register_high}",
          "&self.#{register_low}",
        ]
      when [REGISTER_OPERAND_8, IMMEDIATE_OPERAND_8]
        [
          "&mut self.#{operand_names[0]}",
          "immediate"
        ]
      when [REGISTER_OPERAND_8, REGISTER_OPERAND_8]
        if instruction_data.fetch(:any_shared_register)
          variable_assignments << "let src_register = &self.#{operand_names[1]} as *const u8;"
          source_register_ref = "src_register"
        else
          source_register_ref = "&self.#{operand_names[1]}"
        end

        [
          "&mut self.#{operand_names[0]}",
          source_register_ref,
        ]
      when [REGISTER_OPERAND_8, IMMEDIATE_OPERAND_16]
        [
          "&mut self.#{operand_names[0]}",
          "&immediate_high",
          "&immediate_low",
        ]
      when [REGISTER_OPERAND_8, REGISTER_OPERAND_16]
        if instruction_data.fetch(:any_shared_register)
          variable_assignments << "let dst_register = &mut self.#{operand_names[0]} as *mut u8;"
          dest_register_ref = "dst_register"
        else
          dest_register_ref = "&mut self.#{operand_names[0]}"
        end

        src_register_high, src_register_low = operand_names[1].chars

        # The mutable is required for operations that mutate the source, e.g. LDD.
        [
          dest_register_ref,
          "&mut self.#{src_register_high}",
          "&mut self.#{src_register_low}",
        ]
      when [REGISTER_OPERAND_16, REGISTER_OPERAND_8]
        if instruction_data.fetch(:any_shared_register)
          variable_assignments << "let src_register = &mut self.#{operand_names[1]} as *mut u8;"
          src_register_ref = "src_register"
        else
          src_register_ref = "&self.#{operand_names[1]}"
        end

        dst_register_high, dst_register_low = operand_names[0].chars

        # The mutable is required for operations that mutate the source, e.g. LDD.
        [
          "&mut self.#{dst_register_high}",
          "&mut self.#{dst_register_low}",
          src_register_ref
        ]
      when [REGISTER_OPERAND_16, IMMEDIATE_OPERAND_8]
        dst_register_high, dst_register_low = operand_names[0].chars
        [
          "&self.#{dst_register_high}",
          "&self.#{dst_register_low}",
          "immediate"
        ]
      when [IMMEDIATE_OPERAND_16, REGISTER_OPERAND_8]
        [
          "&immediate_high",
          "&immediate_low",
          "&self.#{operand_names[1]}"
        ]
      when [IMMEDIATE_OPERAND_8, REGISTER_OPERAND_8]
        [
          "&immediate",
          "&self.#{operand_names[1]}"
        ]
      else
        raise "Unrecognized operand types: #{operand_types}"
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
