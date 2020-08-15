// From the version (nightly 792c645 2020-08-17). Not really meaningful, as triggered on structs
// without fields.
//
#![allow(clippy::new_without_default)]

mod io_frontend;
mod keycode;
mod logger;
mod stdout_logger;

pub use crate::io_frontend::IoFrontend;
pub use crate::keycode::Keycode;
pub use crate::logger::Logger;
pub use crate::stdout_logger::StdoutLogger;
