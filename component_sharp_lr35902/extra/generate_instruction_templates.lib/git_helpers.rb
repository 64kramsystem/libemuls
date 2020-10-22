require 'English'
require 'shellwords'

module GitHelpers
  extend self

  def restore_files(*files)
    command = "git checkout HEAD -- " + files.map { |file| file.shellescape }.join(" ")
    execute command
  end

  private

  def execute(command)
    success = system(command)

    exit $CHILD_STATUS.exitstatus if !success
  end
end
