module CommandlineDecoder
  extend self

  HELP = <<~HELP
    Usage: #{File.basename($PROGRAM_NAME)} [<opcode_>{,<opcode_...>}]

    Opcodes are hex, case-independent, and without prefix, e.g. `06,3C`.
  HELP

  def execute
    if (ARGV & %w[-h --help]).size > 0
      puts HELP
      exit 0
    elsif ARGV.size > 1
      puts HELP
      exit 1
    else
      ARGV[0].to_s.split(",")
    end
  end
end
