#![allow(non_snake_case)]
#![allow(clippy::new_without_default)]

pub mod cpu;

pub use crate::cpu::Cpu as SharpLr35902;

#[cfg(test)]
mod cpu_test;
