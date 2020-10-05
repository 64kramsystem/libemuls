require_relative "../shared.lib/operand_types"

class CpuExecutionTemplatesGenerator
  include OperandTypes

  def initialize
    @buffer = StringIO.new
  end

  def add_code!(instruction_encoded, instruction_data, instruction_code)
    generate_method_signature!(instruction_encoded, instruction_data)
    generate_register_operations!(instruction_data, instruction_code)
    generate_flag_operations!(instruction_encoded, instruction_data, instruction_code)

    generate_closure!
  end

  def code
    @buffer.string
  end

  private

  def generate_method_signature!(instruction_encoded, instruction_data)
    operand_types = instruction_data.fetch("operand_types")

    @buffer.print "    fn execute_#{instruction_encoded}(&mut self"

    operand_types.zip(["dst", "src"]).each do |operand_type, register_position|
      case operand_type
      when REGISTER_OPERAND_8
        @buffer.print ", #{register_position}_register: Reg8"
      when REGISTER_OPERAND_16
        @buffer.print ", #{register_position}_register: Reg16"
      when IMMEDIATE_OPERAND_8
        @buffer.print ", immediate: &u8"
      when IMMEDIATE_OPERAND_16
        @buffer.print ", immediate_high: &u8, immediate_low: &u8"
      when FLAG_OPERAND
        @buffer.print ", condition_flag: Flag"
      else
        raise "Unexpected operand type: #{operand_type.type}"
      end
    end

    @buffer.puts ") {"
  end

  def generate_register_operations!(instruction_data, instruction_code)
    instruction_size = instruction_data.fetch("instruction_size")

    @buffer.puts <<-RUST
        self[Reg16::PC] += #{instruction_size};

    RUST

    operation_code = instruction_code.fetch(:operation_code)

    if operation_code
      operation_code.each_line do |operation_statement|
        if operation_statement.strip.empty?
          @buffer.puts
        else
          @buffer.puts "        #{operation_statement}"
        end
      end
      @buffer.puts
    end
  end

  def generate_flag_operations!(instruction_encoded, instruction_data, instruction_code)
    flags_set_data = instruction_data.fetch("flags_set")
    operation_code = instruction_code.fetch(:operation_code)

    flags_set_data.each do |flag, state|
      next if operation_code =~ /self.set_flag\(Flag::#{flag.downcase},/

      flag_operation = \
        case flag
        when 'H', 'C'
          generate_h_c_flag_operation!(flag, state)
        when 'Z'
          generate_z_flag_operation!(state)
        else
          generate_n_flag_operation!(state)
        end

      if flag_operation
        @buffer.print flag_operation
      elsif operation_code !~ /self.set_flag\(Flag::/
        # In addition to catching incomplete code, this also catches potentials meta/data errors.
        #
        raise "Missing #{flag.upcase} flag implementation (code) for instruction #{instruction_encoded} (state: #{state})!"
      end
    end
  end

  def generate_h_c_flag_operation!(flag, state)
    case state
    when false, true
      <<-RUST
        self.set_flag(Flag::#{flag.downcase}, #{state});
      RUST
    when 4, 8, 12, 16
      <<-RUST
        let flag_#{flag.downcase}_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, #{state});
        self.set_flag(Flag::#{flag.downcase}, flag_#{flag.downcase}_value);
      RUST
    end
  end

  def generate_z_flag_operation!(state)
    case state
    when false, true
      <<-RUST
        self.set_flag(Flag::z, #{state});
      RUST
    when "*"
      <<-RUST
        self.set_flag(Flag::z, result == 0);
      RUST
    end
  end

  def generate_n_flag_operation!(state)
    case state
    when false, true
      <<-RUST
        self.set_flag(Flag::n, #{state});
      RUST
    end
  end

  # Closing brace.
  #
  def generate_closure!
    @buffer.puts <<-RUST
    }

    RUST
  end
end
