// From the version (nightly 792c645 2020-08-17). Not really meaningful, as triggered on structs
// without fields.
//
#![allow(clippy::new_without_default)]

mod io_frontend;

pub mod audio;
pub mod events;
pub mod logging;
pub mod video;

pub use io_frontend::IoFrontend;
