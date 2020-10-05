#![allow(non_camel_case_types)]

use crate::utils;
use rand::RngCore;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub struct Registers8Pair {
    pub l: u8,
    pub h: u8,
}

#[repr(C)]
pub union Register16 {
    pub r16: u16,
    pub r8: Registers8Pair,
}

// Naming of the register enums is tricky. Using `Register<width>` is accurate, however it clashes
// with the struct. Since such enums are extensively used, using an abbreviation works the problem
// around while arguably maintaining sufficient expressivity.
// The Flag enum doesn't have the clashing problem, and it's short enough not to require any
// attention.
//
#[derive(Copy, Clone)]
pub(crate) enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

// AF is not included, as keeping the flags stored as register is not convenient; the only cases where
// they're treated as such is `PUSH/POP AF`.
//
#[derive(Copy, Clone)]
pub(crate) enum Reg16 {
    BC,
    DE,
    HL,
    SP,
    PC,
}

// The `f` suffix is not required in this context, since there is no ambiguity like in the Cpu struct.
//
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
    BC: Register16,
    DE: Register16,
    HL: Register16,
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

impl Index<Reg8> for Cpu {
    type Output = u8;

    fn index(&self, register: Reg8) -> &Self::Output {
        match register {
            Reg8::A => &self.A,
            Reg8::B => unsafe { &self.BC.r8.h },
            Reg8::C => unsafe { &self.BC.r8.l },
            Reg8::D => unsafe { &self.DE.r8.h },
            Reg8::E => unsafe { &self.DE.r8.l },
            Reg8::H => unsafe { &self.HL.r8.h },
            Reg8::L => unsafe { &self.HL.r8.l },
        }
    }
}

impl IndexMut<Reg8> for Cpu {
    fn index_mut(&mut self, register: Reg8) -> &mut Self::Output {
        match register {
            Reg8::A => &mut self.A,
            Reg8::B => unsafe { &mut self.BC.r8.h },
            Reg8::C => unsafe { &mut self.BC.r8.l },
            Reg8::D => unsafe { &mut self.DE.r8.h },
            Reg8::E => unsafe { &mut self.DE.r8.l },
            Reg8::H => unsafe { &mut self.HL.r8.h },
            Reg8::L => unsafe { &mut self.HL.r8.l },
        }
    }
}

impl Index<Reg16> for Cpu {
    type Output = u16;

    fn index(&self, register: Reg16) -> &Self::Output {
        match register {
            Reg16::BC => unsafe { &self.BC.r16 },
            Reg16::DE => unsafe { &self.DE.r16 },
            Reg16::HL => unsafe { &self.HL.r16 },
            Reg16::SP => &self.SP,
            Reg16::PC => &self.PC,
        }
    }
}

impl IndexMut<Reg16> for Cpu {
    fn index_mut(&mut self, register: Reg16) -> &mut Self::Output {
        match register {
            Reg16::BC => unsafe { &mut self.BC.r16 },
            Reg16::DE => unsafe { &mut self.DE.r16 },
            Reg16::HL => unsafe { &mut self.HL.r16 },
            Reg16::SP => &mut self.SP,
            Reg16::PC => &mut self.PC,
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
            BC: Register16 { r16: 0 },
            DE: Register16 { r16: 0 },
            HL: Register16 { r16: 0 },
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
}
