#![allow(non_camel_case_types)]

use crate::utils;
use rand::RngCore;
use std::convert::TryInto;
use std::ops::{Index, IndexMut};
use strum_macros::EnumIter;

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
#[derive(Copy, Clone, Debug, EnumIter)]
pub(crate) enum Reg8 {
    A,
    F,
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
#[derive(Copy, Clone, Debug, EnumIter)]
pub(crate) enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

// The `f` suffix is not required in this context, since there is no ambiguity like in the Cpu struct.
//
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Flag {
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
    // WATCH OUT! For consistency, registers/flags must be accessed via Index[Mut] trait, with the
    // exception of flag setting, to perform via `flag_set()`.

    // Registers; flags are part of the `F` register.
    //
    AF: Register16,
    BC: Register16,
    DE: Register16,
    HL: Register16,
    SP: u16,
    PC: u16,

    // Internal RAM. Until there is properly memory management, this is kept trivial.
    //
    pub internal_ram: [u8; 0x10_000],
}

impl Index<Reg8> for Cpu {
    type Output = u8;

    fn index(&self, register: Reg8) -> &Self::Output {
        match register {
            Reg8::A => unsafe { &self.AF.r8.h },
            Reg8::F => unsafe { &self.AF.r8.l },
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
            Reg8::A => unsafe { &mut self.AF.r8.h },
            Reg8::F => unsafe { &mut self.AF.r8.l },
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
            Reg16::AF => unsafe { &self.AF.r16 },
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
            Reg16::AF => unsafe { &mut self.AF.r16 },
            Reg16::BC => unsafe { &mut self.BC.r16 },
            Reg16::DE => unsafe { &mut self.DE.r16 },
            Reg16::HL => unsafe { &mut self.HL.r16 },
            Reg16::SP => &mut self.SP,
            Reg16::PC => &mut self.PC,
        }
    }
}

// Flags and `Index[Mut]` don't play very well together, for a variety of reasons, basically, the
// mismatch between the return value (int or bool), and the underlying representation (sub-byte).
//
impl Cpu {
    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::z => (unsafe { self.AF.r8.l } & 0b1000_0000 > 0),
            Flag::n => (unsafe { self.AF.r8.l } & 0b0100_0000 > 0),
            Flag::h => (unsafe { self.AF.r8.l } & 0b0010_0000 > 0),
            Flag::c => (unsafe { self.AF.r8.l } & 0b0001_0000 > 0),
        }
    }

    // IndexMut must return a reference, so unfortunately, can't be used to set bits in a bitfield.
    //
    pub(crate) fn set_flag(&mut self, flag: Flag, value: bool) {
        let bit_position = match flag {
            Flag::z => 7,
            Flag::n => 6,
            Flag::h => 5,
            Flag::c => 4,
        };

        // See https://stackoverflow.com/a/17804619.
        //
        self[Reg8::F] = (self[Reg8::F] & !(1 << bit_position)) | ((value as u8) << bit_position);
    }
}

impl Cpu {
    pub fn new() -> Self {
        let mut internal_ram = [0; 0x10_000];
        rand::thread_rng().fill_bytes(&mut internal_ram);

        Cpu {
            AF: Register16 { r16: 0 },
            BC: Register16 { r16: 0 },
            DE: Register16 { r16: 0 },
            HL: Register16 { r16: 0 },
            SP: 0,
            PC: 0,
            internal_ram,
        }
    }

    /// # Arguments/return value:
    ///
    /// * `instruction_bytes` - instruction, in bytes
    /// * returns the number of clock ticks spent
    ///
    pub fn execute(&mut self, instruction_bytes: &[u8]) -> u8 {
        // Workaround until proper execution from memory is implemented.
        //
        let pc = self[Reg16::PC] as usize;
        self.internal_ram[pc..pc + instruction_bytes.len()].copy_from_slice(&instruction_bytes);

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

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // HELPERS
    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// carry_position
    ///
    /// WATCH OUT #1!: 0-based;
    /// WATCH OUT #2!: if the result is larger (e.g. 16 bits) than the operands (e.g. 8 bits), don't
    ///   forget to size it accordingly. An example of mistake is to use an `overflowing_add()` and
    ///   discard the carry; in such case, it's best to use the resulting carry, rather than this API.
    ///
    /// The formula is based on XOR-based addition; see https://stackoverflow.com/q/62006764/210029.
    ///
    fn compute_carry_flag(operand1: u16, operand2: u16, result: u16, carry_position: u8) -> bool {
        (operand1 ^ operand2 ^ result) & (1 << carry_position) > 0
    }
}
