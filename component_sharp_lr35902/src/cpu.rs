use crate::utils;
use rand::RngCore;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub(crate) enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Copy, Clone)]
pub(crate) enum Register16 {
    SP,
    PC,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub(crate) enum Flag {
    z,
    n,
    h,
    c,
}

// Preliminary, supersimplified implementation.
// Based on a cursory look at the manuals, there is one operation that mass-sets the flags, so it
// makes sense to store them individually.
// The execution is temporary, designed with in mind the testability only.
//
pub struct Cpu {
    // WATCH OUT! For consistency, registers/flags must be accessed via Index[Mut] trait.

    // Registers
    //
    A: u8,

    B: u8,
    C: u8,

    D: u8,
    E: u8,

    H: u8,
    L: u8,

    SP: u16,
    PC: u16,

    // Flags
    //
    zf: bool,
    nf: bool,
    hf: bool,
    cf: bool,

    // Internal RAM. Until there is properly memory management, this is kept trivial.
    //
    pub internal_ram: [u8; 0x10_000],
}

impl Index<Register8> for Cpu {
    type Output = u8;

    fn index(&self, register: Register8) -> &Self::Output {
        match register {
            Register8::A => &self.A,
            Register8::B => &self.B,
            Register8::C => &self.C,
            Register8::D => &self.D,
            Register8::E => &self.E,
            Register8::H => &self.H,
            Register8::L => &self.L,
        }
    }
}

impl IndexMut<Register8> for Cpu {
    fn index_mut(&mut self, register: Register8) -> &mut Self::Output {
        match register {
            Register8::A => &mut self.A,
            Register8::B => &mut self.B,
            Register8::C => &mut self.C,
            Register8::D => &mut self.D,
            Register8::E => &mut self.E,
            Register8::H => &mut self.H,
            Register8::L => &mut self.L,
        }
    }
}

impl Index<Register16> for Cpu {
    type Output = u16;

    fn index(&self, register: Register16) -> &Self::Output {
        match register {
            Register16::SP => &self.SP,
            Register16::PC => &self.PC,
        }
    }
}

impl IndexMut<Register16> for Cpu {
    fn index_mut(&mut self, register: Register16) -> &mut Self::Output {
        match register {
            Register16::SP => &mut self.SP,
            Register16::PC => &mut self.PC,
        }
    }
}

impl Index<Flag> for Cpu {
    type Output = bool;

    fn index(&self, register: Flag) -> &Self::Output {
        match register {
            Flag::z => &self.zf,
            Flag::n => &self.nf,
            Flag::h => &self.hf,
            Flag::c => &self.cf,
        }
    }
}

impl IndexMut<Flag> for Cpu {
    fn index_mut(&mut self, register: Flag) -> &mut Self::Output {
        match register {
            Flag::z => &mut self.zf,
            Flag::n => &mut self.nf,
            Flag::h => &mut self.hf,
            Flag::c => &mut self.cf,
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        let mut internal_ram = [0; 0x10_000];
        rand::thread_rng().fill_bytes(&mut internal_ram);

        Cpu {
            A: 0,
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,
            SP: 0,
            PC: 0,
            zf: false,
            nf: false,
            hf: false,
            cf: false,
            internal_ram,
        }
    }

    /// # Arguments/return value:
    ///
    /// * `instruction_bytes` - instruction, in bytes
    /// * returns the number of clock ticks spent
    ///
    pub fn execute(&mut self, instruction_bytes: &[u8]) -> u8 {
        match instruction_bytes {
            // __OPCODES_DECODING_REPLACEMENT_START__
            // __OPCODES_DECODING_REPLACEMENT_END__
            _ => {
                let formatted_instruction = utils::format_hex(instruction_bytes);
                panic!("Unsupported instruction!: {}", formatted_instruction)
            }
        }
    }

    // __OPCODES_EXECUTION_REPLACEMENT_START__
    // __OPCODES_EXECUTION_REPLACEMENT_END__

    // HELPERS /////////////////////////////////////////////////////////////////////////////////////

    // Composes an address from a high and a low byte.
    //
    pub fn compose_address(high_byte: u8, low_byte: u8) -> usize {
        ((high_byte as usize) << 8) + (low_byte as usize)
    }
}
