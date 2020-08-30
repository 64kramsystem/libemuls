use super::Logger;

/// Logs to stdout.
///
pub struct StdoutLogger {}

impl StdoutLogger {
    pub fn new() -> StdoutLogger {
        StdoutLogger {}
    }
}

impl Logger for StdoutLogger {
    /// Logs the message to stdout, appending a newline.
    ///
    fn log(&mut self, message: String) {
        println!("{}", message);
    }
}
