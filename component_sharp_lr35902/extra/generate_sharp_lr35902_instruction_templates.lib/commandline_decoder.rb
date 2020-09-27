module CommandlineDecoder
  extend self

  def execute
    if (ARGV & %w[-h --help]).size > 0
      puts "Usage: #{File.basename($PROGRAM_NAME)} [0x<opcode_hex>]"
      exit 0
    elsif ARGV.size > 1
      puts "Usage: #{File.basename($PROGRAM_NAME)} [0x<opcode_hex>]"
      exit 1
    else
      opcode = ARGV[0]&.hex
    end
  end
end
