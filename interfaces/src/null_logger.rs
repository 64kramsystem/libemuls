use crate::logger::Logger;

pub struct NullLogger {}

impl NullLogger {
    pub fn new() -> NullLogger {
        NullLogger {}
    }
}

impl Logger for NullLogger {}
