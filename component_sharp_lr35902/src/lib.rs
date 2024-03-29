#![allow(non_snake_case)]
#![allow(clippy::new_without_default)]

mod utils;

pub mod cpu;

pub use cpu::Cpu as SharpLr35902;

#[cfg(test)]
mod cpu_test;
