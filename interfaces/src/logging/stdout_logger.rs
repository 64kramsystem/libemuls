use super::Logger;

pub struct StdoutLogger {}

impl StdoutLogger {
    pub fn new() -> StdoutLogger {
        StdoutLogger {}
    }
}

impl Logger for StdoutLogger {
    fn log(&mut self, message: String) {
        println!("{}", message);
    }
}
