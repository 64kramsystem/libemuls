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
    flags_data = instruction_data.fetch("flags_data")
    operation_code = instruction_code.fetch(:operation_code)

    flags_data.each do |flag, state|
      case state
      when "0"
        @buffer.puts "      self[Flag::#{flag.downcase}] = false;"
      when "1"
        @buffer.puts "      self[Flag::#{flag.downcase}] = true;"
      when "*"
        if flag == "Z"
          @buffer.puts <<-RUST
      if carry {
          self[Flag::z] = true;
      }
          RUST
        else
          # Make sure the operation code takes care of it!
          #
          raise "Missing #{instruction_encoded} #{flag} flag setting!" if operation_code !~ /self\[Flag::#{flag.downcase}\] = /
        end
      when "-"
        # unaffected; do nothing
      else
        raise "Invalid flag state: #{state}"
      end
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
