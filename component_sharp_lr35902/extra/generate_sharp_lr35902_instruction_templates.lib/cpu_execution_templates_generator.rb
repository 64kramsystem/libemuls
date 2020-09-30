require_relative "instructions_data"

class CpuExecutionTemplatesGenerator
  include InstructionsData

  def initialize
    @buffer = StringIO.new
  end

  def add_code!(opcode_family, instruction_data)
    generate_method_signature!(opcode_family, instruction_data)
    generate_register_operations!(instruction_data)
    generate_flag_operations!(instruction_data)
    generate_closure!
  end

  def code
    @buffer.string
  end

  private

  def generate_method_signature!(opcode_family, instruction_data)
    operand_types = instruction_data.fetch(:operand_types)

    @buffer.print "    fn execute_#{opcode_family}(&mut self"

    operand_types.zip(["dst", "src"]) do |operand_type, position|
      case operand_type.type
      when REGISTER_OPERAND_8
        @buffer.print ", #{position}_register: Register8"
      when REGISTER_OPERAND_16
        @buffer.print ", #{position}_register_high: Register8, #{position}_register_low: Register8"
      when REGISTER_SP
        @buffer.print ", #{position}_register: Register16"
      when IMMEDIATE_OPERAND_8
        @buffer.print ", immediate: &u8"
      when IMMEDIATE_OPERAND_16
        @buffer.print ", immediate_high: &u8, immediate_low: &u8"
      when nil
        # Do nothing
      else
        raise "Unexpected operand 0 type: #{operand_types[0].type}"
      end
    end

    flags_data = instruction_data.fetch(:flags_data)

    flags_data.each do |flag, state|
      case state
      when "0", "1", flag
        @buffer.print ", #{flag.downcase}f: Flag"
      when "-"
        # ignore
      else
        raise "Invalid flag state: #{state}"
      end
    end

    @buffer.puts ") {"
  end

  def generate_register_operations!(instruction_data)
    instruction_size = instruction_data.fetch(:instruction_size)

    @buffer.puts <<-RUST
      self[Register16::PC] += #{instruction_size};

    RUST

    operation_code = instruction_data[:operation_code]

    if operation_code
      operation_code.each_line do |operation_statement|
        if operation_statement.strip.empty?
          @buffer.puts
        else
          @buffer.puts "      #{operation_statement}"
        end
      end
      @buffer.puts
    end
  end

  def generate_flag_operations!(instruction_data)
    flags_data = instruction_data.fetch(:flags_data)
    operation_code = instruction_data[:operation_code]

    flags_data.each do |flag, state|
      case state
      when "0"
        @buffer.puts "      self[#{flag.downcase}f] = false;"
      when "1"
        @buffer.puts "      self[#{flag.downcase}f] = true;"
      when flag
        if flag == "Z"
          @buffer.puts <<-RUST
      if carry {
        self[zf] = true;
      }
          RUST
        else
          # Make sure the operation code takes care of it!
          #
          raise "Missing #{opcode_family} #{flag} flag setting!" if operation_code !~ /self\[#{flag.downcase}f\] = /
        end
      when "-"
        # do nothing
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
