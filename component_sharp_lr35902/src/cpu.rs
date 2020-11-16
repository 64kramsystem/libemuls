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
            [0x06, immediate @ _] => {
                self.execute_LD_r_n(Reg8::B, immediate);
                8
            }
            [0x0E, immediate @ _] => {
                self.execute_LD_r_n(Reg8::C, immediate);
                8
            }
            [0x16, immediate @ _] => {
                self.execute_LD_r_n(Reg8::D, immediate);
                8
            }
            [0x1E, immediate @ _] => {
                self.execute_LD_r_n(Reg8::E, immediate);
                8
            }
            [0x26, immediate @ _] => {
                self.execute_LD_r_n(Reg8::H, immediate);
                8
            }
            [0x2E, immediate @ _] => {
                self.execute_LD_r_n(Reg8::L, immediate);
                8
            }
            [0x3E, immediate @ _] => {
                self.execute_LD_r_n(Reg8::A, immediate);
                8
            }
            [0x78] => {
                self.execute_LD_r1_r2(Reg8::A, Reg8::B);
                4
            }
            [0x79] => {
                self.execute_LD_r1_r2(Reg8::A, Reg8::C);
                4
            }
            [0x7A] => {
                self.execute_LD_r1_r2(Reg8::A, Reg8::D);
                4
            }
            [0x7B] => {
                self.execute_LD_r1_r2(Reg8::A, Reg8::E);
                4
            }
            [0x7C] => {
                self.execute_LD_r1_r2(Reg8::A, Reg8::H);
                4
            }
            [0x7D] => {
                self.execute_LD_r1_r2(Reg8::A, Reg8::L);
                4
            }
            [0x41] => {
                self.execute_LD_r1_r2(Reg8::B, Reg8::C);
                4
            }
            [0x42] => {
                self.execute_LD_r1_r2(Reg8::B, Reg8::D);
                4
            }
            [0x43] => {
                self.execute_LD_r1_r2(Reg8::B, Reg8::E);
                4
            }
            [0x44] => {
                self.execute_LD_r1_r2(Reg8::B, Reg8::H);
                4
            }
            [0x45] => {
                self.execute_LD_r1_r2(Reg8::B, Reg8::L);
                4
            }
            [0x48] => {
                self.execute_LD_r1_r2(Reg8::C, Reg8::B);
                4
            }
            [0x4A] => {
                self.execute_LD_r1_r2(Reg8::C, Reg8::D);
                4
            }
            [0x4B] => {
                self.execute_LD_r1_r2(Reg8::C, Reg8::E);
                4
            }
            [0x4C] => {
                self.execute_LD_r1_r2(Reg8::C, Reg8::H);
                4
            }
            [0x4D] => {
                self.execute_LD_r1_r2(Reg8::C, Reg8::L);
                4
            }
            [0x50] => {
                self.execute_LD_r1_r2(Reg8::D, Reg8::B);
                4
            }
            [0x51] => {
                self.execute_LD_r1_r2(Reg8::D, Reg8::C);
                4
            }
            [0x53] => {
                self.execute_LD_r1_r2(Reg8::D, Reg8::E);
                4
            }
            [0x54] => {
                self.execute_LD_r1_r2(Reg8::D, Reg8::H);
                4
            }
            [0x55] => {
                self.execute_LD_r1_r2(Reg8::D, Reg8::L);
                4
            }
            [0x58] => {
                self.execute_LD_r1_r2(Reg8::E, Reg8::B);
                4
            }
            [0x59] => {
                self.execute_LD_r1_r2(Reg8::E, Reg8::C);
                4
            }
            [0x5A] => {
                self.execute_LD_r1_r2(Reg8::E, Reg8::D);
                4
            }
            [0x5C] => {
                self.execute_LD_r1_r2(Reg8::E, Reg8::H);
                4
            }
            [0x5D] => {
                self.execute_LD_r1_r2(Reg8::E, Reg8::L);
                4
            }
            [0x60] => {
                self.execute_LD_r1_r2(Reg8::H, Reg8::B);
                4
            }
            [0x61] => {
                self.execute_LD_r1_r2(Reg8::H, Reg8::C);
                4
            }
            [0x62] => {
                self.execute_LD_r1_r2(Reg8::H, Reg8::D);
                4
            }
            [0x63] => {
                self.execute_LD_r1_r2(Reg8::H, Reg8::E);
                4
            }
            [0x65] => {
                self.execute_LD_r1_r2(Reg8::H, Reg8::L);
                4
            }
            [0x68] => {
                self.execute_LD_r1_r2(Reg8::L, Reg8::B);
                4
            }
            [0x69] => {
                self.execute_LD_r1_r2(Reg8::L, Reg8::C);
                4
            }
            [0x6A] => {
                self.execute_LD_r1_r2(Reg8::L, Reg8::D);
                4
            }
            [0x6B] => {
                self.execute_LD_r1_r2(Reg8::L, Reg8::E);
                4
            }
            [0x6C] => {
                self.execute_LD_r1_r2(Reg8::L, Reg8::H);
                4
            }
            [0x47] => {
                self.execute_LD_r1_r2(Reg8::B, Reg8::A);
                4
            }
            [0x4F] => {
                self.execute_LD_r1_r2(Reg8::C, Reg8::A);
                4
            }
            [0x57] => {
                self.execute_LD_r1_r2(Reg8::D, Reg8::A);
                4
            }
            [0x5F] => {
                self.execute_LD_r1_r2(Reg8::E, Reg8::A);
                4
            }
            [0x67] => {
                self.execute_LD_r1_r2(Reg8::H, Reg8::A);
                4
            }
            [0x6F] => {
                self.execute_LD_r1_r2(Reg8::L, Reg8::A);
                4
            }
            [0x7F] => {
                self.execute_LD_r1_r2(Reg8::A, Reg8::A);
                4
            }
            [0x40] => {
                self.execute_LD_r1_r2(Reg8::B, Reg8::B);
                4
            }
            [0x49] => {
                self.execute_LD_r1_r2(Reg8::C, Reg8::C);
                4
            }
            [0x52] => {
                self.execute_LD_r1_r2(Reg8::D, Reg8::D);
                4
            }
            [0x5B] => {
                self.execute_LD_r1_r2(Reg8::E, Reg8::E);
                4
            }
            [0x64] => {
                self.execute_LD_r1_r2(Reg8::H, Reg8::H);
                4
            }
            [0x6D] => {
                self.execute_LD_r1_r2(Reg8::L, Reg8::L);
                4
            }
            [0x46] => {
                self.execute_LD_r1_Irr2(Reg8::B, Reg16::HL);
                8
            }
            [0x4E] => {
                self.execute_LD_r1_Irr2(Reg8::C, Reg16::HL);
                8
            }
            [0x56] => {
                self.execute_LD_r1_Irr2(Reg8::D, Reg16::HL);
                8
            }
            [0x5E] => {
                self.execute_LD_r1_Irr2(Reg8::E, Reg16::HL);
                8
            }
            [0x7E] => {
                self.execute_LD_r1_Irr2(Reg8::A, Reg16::HL);
                8
            }
            [0x0A] => {
                self.execute_LD_r1_Irr2(Reg8::A, Reg16::BC);
                8
            }
            [0x1A] => {
                self.execute_LD_r1_Irr2(Reg8::A, Reg16::DE);
                8
            }
            [0x66] => {
                self.execute_LD_r1_Irr2(Reg8::H, Reg16::HL);
                8
            }
            [0x6E] => {
                self.execute_LD_r1_Irr2(Reg8::L, Reg16::HL);
                8
            }
            [0x70] => {
                self.execute_LD_Irr1_r2(Reg16::HL, Reg8::B);
                8
            }
            [0x71] => {
                self.execute_LD_Irr1_r2(Reg16::HL, Reg8::C);
                8
            }
            [0x72] => {
                self.execute_LD_Irr1_r2(Reg16::HL, Reg8::D);
                8
            }
            [0x73] => {
                self.execute_LD_Irr1_r2(Reg16::HL, Reg8::E);
                8
            }
            [0x74] => {
                self.execute_LD_Irr1_r2(Reg16::HL, Reg8::H);
                8
            }
            [0x75] => {
                self.execute_LD_Irr1_r2(Reg16::HL, Reg8::L);
                8
            }
            [0x02] => {
                self.execute_LD_Irr1_r2(Reg16::BC, Reg8::A);
                8
            }
            [0x12] => {
                self.execute_LD_Irr1_r2(Reg16::DE, Reg8::A);
                8
            }
            [0x77] => {
                self.execute_LD_Irr1_r2(Reg16::HL, Reg8::A);
                8
            }
            [0x36, immediate @ _] => {
                self.execute_LD_IHL_n(immediate);
                12
            }
            [0xFA, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_LD_A_Inn(immediate);
                16
            }
            [0xEA, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_LD_Inn_A(immediate);
                16
            }
            [0xF2] => {
                self.execute_LD_A_IC();
                8
            }
            [0xE2] => {
                self.execute_LD_IC_A();
                8
            }
            [0x3A] => {
                self.execute_LDD_A_IHL();
                8
            }
            [0x32] => {
                self.execute_LDD_IHL_A();
                8
            }
            [0x2A] => {
                self.execute_LDI_A_IHL();
                8
            }
            [0x22] => {
                self.execute_LDI_IHL_A();
                8
            }
            [0xE0, immediate @ _] => {
                self.execute_LDH_In_A(immediate);
                12
            }
            [0xF0, immediate @ _] => {
                self.execute_LDH_A_In(immediate);
                12
            }
            [0x01, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_LD_rr_nn(Reg16::BC, immediate);
                12
            }
            [0x11, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_LD_rr_nn(Reg16::DE, immediate);
                12
            }
            [0x21, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_LD_rr_nn(Reg16::HL, immediate);
                12
            }
            [0x31, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_LD_rr_nn(Reg16::SP, immediate);
                12
            }
            [0xF9] => {
                self.execute_LD_SP_HL();
                8
            }
            [0xF8, immediate @ _] => {
                self.execute_LDHL_SP_n(immediate);
                12
            }
            [0x08, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_LD_Inn_SP(immediate);
                20
            }
            [0xF5] => {
                self.execute_PUSH_rr(Reg16::AF);
                16
            }
            [0xC5] => {
                self.execute_PUSH_rr(Reg16::BC);
                16
            }
            [0xD5] => {
                self.execute_PUSH_rr(Reg16::DE);
                16
            }
            [0xE5] => {
                self.execute_PUSH_rr(Reg16::HL);
                16
            }
            [0xC1] => {
                self.execute_POP_rr(Reg16::BC);
                12
            }
            [0xD1] => {
                self.execute_POP_rr(Reg16::DE);
                12
            }
            [0xE1] => {
                self.execute_POP_rr(Reg16::HL);
                12
            }
            [0xF1] => {
                self.execute_POP_AF();
                12
            }
            [0x87] => {
                self.execute_ADD_A_r(Reg8::A);
                4
            }
            [0x80] => {
                self.execute_ADD_A_r(Reg8::B);
                4
            }
            [0x81] => {
                self.execute_ADD_A_r(Reg8::C);
                4
            }
            [0x82] => {
                self.execute_ADD_A_r(Reg8::D);
                4
            }
            [0x83] => {
                self.execute_ADD_A_r(Reg8::E);
                4
            }
            [0x84] => {
                self.execute_ADD_A_r(Reg8::H);
                4
            }
            [0x85] => {
                self.execute_ADD_A_r(Reg8::L);
                4
            }
            [0x86] => {
                self.execute_ADD_A_IHL();
                8
            }
            [0xC6, immediate @ _] => {
                self.execute_ADD_A_n(immediate);
                8
            }
            [0x8F] => {
                self.execute_ADC_A_r(Reg8::A);
                4
            }
            [0x88] => {
                self.execute_ADC_A_r(Reg8::B);
                4
            }
            [0x89] => {
                self.execute_ADC_A_r(Reg8::C);
                4
            }
            [0x8A] => {
                self.execute_ADC_A_r(Reg8::D);
                4
            }
            [0x8B] => {
                self.execute_ADC_A_r(Reg8::E);
                4
            }
            [0x8C] => {
                self.execute_ADC_A_r(Reg8::H);
                4
            }
            [0x8D] => {
                self.execute_ADC_A_r(Reg8::L);
                4
            }
            [0x8E] => {
                self.execute_ADC_A_IHL();
                8
            }
            [0xCE, immediate @ _] => {
                self.execute_ADC_A_n(immediate);
                8
            }
            [0x97] => {
                self.execute_SUB_A_r(Reg8::A);
                4
            }
            [0x90] => {
                self.execute_SUB_A_r(Reg8::B);
                4
            }
            [0x91] => {
                self.execute_SUB_A_r(Reg8::C);
                4
            }
            [0x92] => {
                self.execute_SUB_A_r(Reg8::D);
                4
            }
            [0x93] => {
                self.execute_SUB_A_r(Reg8::E);
                4
            }
            [0x94] => {
                self.execute_SUB_A_r(Reg8::H);
                4
            }
            [0x95] => {
                self.execute_SUB_A_r(Reg8::L);
                4
            }
            [0x96] => {
                self.execute_SUB_A_IHL();
                8
            }
            [0xD6, immediate @ _] => {
                self.execute_SUB_A_n(immediate);
                8
            }
            [0x9F] => {
                self.execute_SBC_A_r(Reg8::A);
                4
            }
            [0x98] => {
                self.execute_SBC_A_r(Reg8::B);
                4
            }
            [0x99] => {
                self.execute_SBC_A_r(Reg8::C);
                4
            }
            [0x9A] => {
                self.execute_SBC_A_r(Reg8::D);
                4
            }
            [0x9B] => {
                self.execute_SBC_A_r(Reg8::E);
                4
            }
            [0x9C] => {
                self.execute_SBC_A_r(Reg8::H);
                4
            }
            [0x9D] => {
                self.execute_SBC_A_r(Reg8::L);
                4
            }
            [0x9E] => {
                self.execute_SBC_A_IHL();
                8
            }
            [0xDE, immediate @ _] => {
                self.execute_SBC_A_n(immediate);
                8
            }
            [0xA7] => {
                self.execute_AND_A_r(Reg8::A);
                4
            }
            [0xA0] => {
                self.execute_AND_A_r(Reg8::B);
                4
            }
            [0xA1] => {
                self.execute_AND_A_r(Reg8::C);
                4
            }
            [0xA2] => {
                self.execute_AND_A_r(Reg8::D);
                4
            }
            [0xA3] => {
                self.execute_AND_A_r(Reg8::E);
                4
            }
            [0xA4] => {
                self.execute_AND_A_r(Reg8::H);
                4
            }
            [0xA5] => {
                self.execute_AND_A_r(Reg8::L);
                4
            }
            [0xA6] => {
                self.execute_AND_A_IHL();
                8
            }
            [0xE6, immediate @ _] => {
                self.execute_AND_A_n(immediate);
                8
            }
            [0xB7] => {
                self.execute_OR_A_r(Reg8::A);
                4
            }
            [0xB0] => {
                self.execute_OR_A_r(Reg8::B);
                4
            }
            [0xB1] => {
                self.execute_OR_A_r(Reg8::C);
                4
            }
            [0xB2] => {
                self.execute_OR_A_r(Reg8::D);
                4
            }
            [0xB3] => {
                self.execute_OR_A_r(Reg8::E);
                4
            }
            [0xB4] => {
                self.execute_OR_A_r(Reg8::H);
                4
            }
            [0xB5] => {
                self.execute_OR_A_r(Reg8::L);
                4
            }
            [0xB6] => {
                self.execute_OR_A_IHL();
                8
            }
            [0xF6, immediate @ _] => {
                self.execute_OR_A_n(immediate);
                8
            }
            [0xAF] => {
                self.execute_XOR_A_r(Reg8::A);
                4
            }
            [0xA8] => {
                self.execute_XOR_A_r(Reg8::B);
                4
            }
            [0xA9] => {
                self.execute_XOR_A_r(Reg8::C);
                4
            }
            [0xAA] => {
                self.execute_XOR_A_r(Reg8::D);
                4
            }
            [0xAB] => {
                self.execute_XOR_A_r(Reg8::E);
                4
            }
            [0xAC] => {
                self.execute_XOR_A_r(Reg8::H);
                4
            }
            [0xAD] => {
                self.execute_XOR_A_r(Reg8::L);
                4
            }
            [0xAE] => {
                self.execute_XOR_A_IHL();
                8
            }
            [0xEE, immediate @ _] => {
                self.execute_XOR_A_n(immediate);
                8
            }
            [0xBF] => {
                self.execute_CP_A_r(Reg8::A);
                4
            }
            [0xB8] => {
                self.execute_CP_A_r(Reg8::B);
                4
            }
            [0xB9] => {
                self.execute_CP_A_r(Reg8::C);
                4
            }
            [0xBA] => {
                self.execute_CP_A_r(Reg8::D);
                4
            }
            [0xBB] => {
                self.execute_CP_A_r(Reg8::E);
                4
            }
            [0xBC] => {
                self.execute_CP_A_r(Reg8::H);
                4
            }
            [0xBD] => {
                self.execute_CP_A_r(Reg8::L);
                4
            }
            [0xBE] => {
                self.execute_CP_A_IHL();
                8
            }
            [0xFE, immediate @ _] => {
                self.execute_CP_A_n(immediate);
                8
            }
            [0x3C] => {
                self.execute_INC_r(Reg8::A);
                4
            }
            [0x04] => {
                self.execute_INC_r(Reg8::B);
                4
            }
            [0x0C] => {
                self.execute_INC_r(Reg8::C);
                4
            }
            [0x14] => {
                self.execute_INC_r(Reg8::D);
                4
            }
            [0x1C] => {
                self.execute_INC_r(Reg8::E);
                4
            }
            [0x24] => {
                self.execute_INC_r(Reg8::H);
                4
            }
            [0x2C] => {
                self.execute_INC_r(Reg8::L);
                4
            }
            [0x34] => {
                self.execute_INC_IHL();
                12
            }
            [0x3D] => {
                self.execute_DEC_r(Reg8::A);
                4
            }
            [0x05] => {
                self.execute_DEC_r(Reg8::B);
                4
            }
            [0x0D] => {
                self.execute_DEC_r(Reg8::C);
                4
            }
            [0x15] => {
                self.execute_DEC_r(Reg8::D);
                4
            }
            [0x1D] => {
                self.execute_DEC_r(Reg8::E);
                4
            }
            [0x25] => {
                self.execute_DEC_r(Reg8::H);
                4
            }
            [0x2D] => {
                self.execute_DEC_r(Reg8::L);
                4
            }
            [0x35] => {
                self.execute_DEC_IHL();
                12
            }
            [0x09] => {
                self.execute_ADD_HL_rr(Reg16::BC);
                8
            }
            [0x19] => {
                self.execute_ADD_HL_rr(Reg16::DE);
                8
            }
            [0x29] => {
                self.execute_ADD_HL_rr(Reg16::HL);
                8
            }
            [0x39] => {
                self.execute_ADD_HL_rr(Reg16::SP);
                8
            }
            [0xE8, immediate @ _] => {
                self.execute_ADD_SP_n(immediate);
                16
            }
            [0x03] => {
                self.execute_INC_rr(Reg16::BC);
                8
            }
            [0x13] => {
                self.execute_INC_rr(Reg16::DE);
                8
            }
            [0x23] => {
                self.execute_INC_rr(Reg16::HL);
                8
            }
            [0x33] => {
                self.execute_INC_rr(Reg16::SP);
                8
            }
            [0x0B] => {
                self.execute_DEC_rr(Reg16::BC);
                8
            }
            [0x1B] => {
                self.execute_DEC_rr(Reg16::DE);
                8
            }
            [0x2B] => {
                self.execute_DEC_rr(Reg16::HL);
                8
            }
            [0x3B] => {
                self.execute_DEC_rr(Reg16::SP);
                8
            }
            [0xCB, 0x37] => {
                self.execute_SWAP_r(Reg8::A);
                8
            }
            [0xCB, 0x30] => {
                self.execute_SWAP_r(Reg8::B);
                8
            }
            [0xCB, 0x31] => {
                self.execute_SWAP_r(Reg8::C);
                8
            }
            [0xCB, 0x32] => {
                self.execute_SWAP_r(Reg8::D);
                8
            }
            [0xCB, 0x33] => {
                self.execute_SWAP_r(Reg8::E);
                8
            }
            [0xCB, 0x34] => {
                self.execute_SWAP_r(Reg8::H);
                8
            }
            [0xCB, 0x35] => {
                self.execute_SWAP_r(Reg8::L);
                8
            }
            [0xCB, 0x36] => {
                self.execute_SWAP_IHL();
                16
            }
            [0x27] => {
                self.execute_DAA();
                4
            }
            [0x2F] => {
                self.execute_CPL();
                4
            }
            [0x3F] => {
                self.execute_CCF();
                4
            }
            [0x37] => {
                self.execute_SCF();
                4
            }
            [0x00] => {
                self.execute_NOP();
                4
            }
            [0x07] => {
                self.execute_RLCA();
                4
            }
            [0x17] => {
                self.execute_RLA();
                4
            }
            [0x0F] => {
                self.execute_RRCA();
                4
            }
            [0x1F] => {
                self.execute_RRA();
                4
            }
            [0xCB, 0x07] => {
                self.execute_RLC_r(Reg8::A);
                8
            }
            [0xCB, 0x00] => {
                self.execute_RLC_r(Reg8::B);
                8
            }
            [0xCB, 0x01] => {
                self.execute_RLC_r(Reg8::C);
                8
            }
            [0xCB, 0x02] => {
                self.execute_RLC_r(Reg8::D);
                8
            }
            [0xCB, 0x03] => {
                self.execute_RLC_r(Reg8::E);
                8
            }
            [0xCB, 0x04] => {
                self.execute_RLC_r(Reg8::H);
                8
            }
            [0xCB, 0x05] => {
                self.execute_RLC_r(Reg8::L);
                8
            }
            [0xCB, 0x06] => {
                self.execute_RLC_IHL();
                16
            }
            [0xCB, 0x17] => {
                self.execute_RL_r(Reg8::A);
                8
            }
            [0xCB, 0x10] => {
                self.execute_RL_r(Reg8::B);
                8
            }
            [0xCB, 0x11] => {
                self.execute_RL_r(Reg8::C);
                8
            }
            [0xCB, 0x12] => {
                self.execute_RL_r(Reg8::D);
                8
            }
            [0xCB, 0x13] => {
                self.execute_RL_r(Reg8::E);
                8
            }
            [0xCB, 0x14] => {
                self.execute_RL_r(Reg8::H);
                8
            }
            [0xCB, 0x15] => {
                self.execute_RL_r(Reg8::L);
                8
            }
            [0xCB, 0x16] => {
                self.execute_RL_IHL();
                16
            }
            [0xCB, 0x0F] => {
                self.execute_RRC_r(Reg8::A);
                8
            }
            [0xCB, 0x08] => {
                self.execute_RRC_r(Reg8::B);
                8
            }
            [0xCB, 0x09] => {
                self.execute_RRC_r(Reg8::C);
                8
            }
            [0xCB, 0x0A] => {
                self.execute_RRC_r(Reg8::D);
                8
            }
            [0xCB, 0x0B] => {
                self.execute_RRC_r(Reg8::E);
                8
            }
            [0xCB, 0x0C] => {
                self.execute_RRC_r(Reg8::H);
                8
            }
            [0xCB, 0x0D] => {
                self.execute_RRC_r(Reg8::L);
                8
            }
            [0xCB, 0x0E] => {
                self.execute_RRC_IHL();
                16
            }
            [0xCB, 0x1F] => {
                self.execute_RR_r(Reg8::A);
                8
            }
            [0xCB, 0x18] => {
                self.execute_RR_r(Reg8::B);
                8
            }
            [0xCB, 0x19] => {
                self.execute_RR_r(Reg8::C);
                8
            }
            [0xCB, 0x1A] => {
                self.execute_RR_r(Reg8::D);
                8
            }
            [0xCB, 0x1B] => {
                self.execute_RR_r(Reg8::E);
                8
            }
            [0xCB, 0x1C] => {
                self.execute_RR_r(Reg8::H);
                8
            }
            [0xCB, 0x1D] => {
                self.execute_RR_r(Reg8::L);
                8
            }
            [0xCB, 0x1E] => {
                self.execute_RR_IHL();
                16
            }
            [0xCB, 0x27] => {
                self.execute_SLA_r(Reg8::A);
                8
            }
            [0xCB, 0x20] => {
                self.execute_SLA_r(Reg8::B);
                8
            }
            [0xCB, 0x21] => {
                self.execute_SLA_r(Reg8::C);
                8
            }
            [0xCB, 0x22] => {
                self.execute_SLA_r(Reg8::D);
                8
            }
            [0xCB, 0x23] => {
                self.execute_SLA_r(Reg8::E);
                8
            }
            [0xCB, 0x24] => {
                self.execute_SLA_r(Reg8::H);
                8
            }
            [0xCB, 0x25] => {
                self.execute_SLA_r(Reg8::L);
                8
            }
            [0xCB, 0x26] => {
                self.execute_SLA_IHL();
                16
            }
            [0xCB, 0x2F] => {
                self.execute_SRA_r(Reg8::A);
                8
            }
            [0xCB, 0x28] => {
                self.execute_SRA_r(Reg8::B);
                8
            }
            [0xCB, 0x29] => {
                self.execute_SRA_r(Reg8::C);
                8
            }
            [0xCB, 0x2A] => {
                self.execute_SRA_r(Reg8::D);
                8
            }
            [0xCB, 0x2B] => {
                self.execute_SRA_r(Reg8::E);
                8
            }
            [0xCB, 0x2C] => {
                self.execute_SRA_r(Reg8::H);
                8
            }
            [0xCB, 0x2D] => {
                self.execute_SRA_r(Reg8::L);
                8
            }
            [0xCB, 0x2E] => {
                self.execute_SRA_IHL();
                16
            }
            [0xCB, 0x3F] => {
                self.execute_SRL_r(Reg8::A);
                8
            }
            [0xCB, 0x38] => {
                self.execute_SRL_r(Reg8::B);
                8
            }
            [0xCB, 0x39] => {
                self.execute_SRL_r(Reg8::C);
                8
            }
            [0xCB, 0x3A] => {
                self.execute_SRL_r(Reg8::D);
                8
            }
            [0xCB, 0x3B] => {
                self.execute_SRL_r(Reg8::E);
                8
            }
            [0xCB, 0x3C] => {
                self.execute_SRL_r(Reg8::H);
                8
            }
            [0xCB, 0x3D] => {
                self.execute_SRL_r(Reg8::L);
                8
            }
            [0xCB, 0x3E] => {
                self.execute_SRL_IHL();
                16
            }
            [0xCB, 0x47, immediate @ _] => {
                self.execute_BIT_n_r(immediate, Reg8::A);
                8
            }
            [0xCB, 0x40, immediate @ _] => {
                self.execute_BIT_n_r(immediate, Reg8::B);
                8
            }
            [0xCB, 0x41, immediate @ _] => {
                self.execute_BIT_n_r(immediate, Reg8::C);
                8
            }
            [0xCB, 0x42, immediate @ _] => {
                self.execute_BIT_n_r(immediate, Reg8::D);
                8
            }
            [0xCB, 0x43, immediate @ _] => {
                self.execute_BIT_n_r(immediate, Reg8::E);
                8
            }
            [0xCB, 0x44, immediate @ _] => {
                self.execute_BIT_n_r(immediate, Reg8::H);
                8
            }
            [0xCB, 0x45, immediate @ _] => {
                self.execute_BIT_n_r(immediate, Reg8::L);
                8
            }
            [0xCB, 0x46, immediate @ _] => {
                self.execute_BIT_n_IHL(immediate);
                12
            }
            [0xCB, 0xC7, immediate @ _] => {
                self.execute_SET_n_r(immediate, Reg8::A);
                8
            }
            [0xCB, 0xC0, immediate @ _] => {
                self.execute_SET_n_r(immediate, Reg8::B);
                8
            }
            [0xCB, 0xC1, immediate @ _] => {
                self.execute_SET_n_r(immediate, Reg8::C);
                8
            }
            [0xCB, 0xC2, immediate @ _] => {
                self.execute_SET_n_r(immediate, Reg8::D);
                8
            }
            [0xCB, 0xC3, immediate @ _] => {
                self.execute_SET_n_r(immediate, Reg8::E);
                8
            }
            [0xCB, 0xC4, immediate @ _] => {
                self.execute_SET_n_r(immediate, Reg8::H);
                8
            }
            [0xCB, 0xC5, immediate @ _] => {
                self.execute_SET_n_r(immediate, Reg8::L);
                8
            }
            [0xCB, 0xC6, immediate @ _] => {
                self.execute_SET_n_IHL(immediate);
                16
            }
            [0xCB, 0x87, immediate @ _] => {
                self.execute_RES_n_r(immediate, Reg8::A);
                8
            }
            [0xCB, 0x80, immediate @ _] => {
                self.execute_RES_n_r(immediate, Reg8::B);
                8
            }
            [0xCB, 0x81, immediate @ _] => {
                self.execute_RES_n_r(immediate, Reg8::C);
                8
            }
            [0xCB, 0x82, immediate @ _] => {
                self.execute_RES_n_r(immediate, Reg8::D);
                8
            }
            [0xCB, 0x83, immediate @ _] => {
                self.execute_RES_n_r(immediate, Reg8::E);
                8
            }
            [0xCB, 0x84, immediate @ _] => {
                self.execute_RES_n_r(immediate, Reg8::H);
                8
            }
            [0xCB, 0x85, immediate @ _] => {
                self.execute_RES_n_r(immediate, Reg8::L);
                8
            }
            [0xCB, 0x86, immediate @ _] => {
                self.execute_RES_n_IHL(immediate);
                16
            }
            [0xC3, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_JP_nn(immediate);
                16
            }
            [0xC2, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = false;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_JP_cc_nn(Flag::z, flag_condition, immediate);
                16
            }
            [0xCA, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = true;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_JP_cc_nn(Flag::z, flag_condition, immediate);
                16
            }
            [0xD2, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = false;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_JP_cc_nn(Flag::c, flag_condition, immediate);
                16
            }
            [0xDA, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = true;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_JP_cc_nn(Flag::c, flag_condition, immediate);
                16
            }
            [0xE9] => {
                self.execute_JP_IHL();
                4
            }
            [0x18, immediate @ _] => {
                self.execute_JR_n(immediate);
                12
            }
            [0x20, immediate @ _] => {
                let flag_condition = false;
                self.execute_JR_cc_n(Flag::z, flag_condition, immediate);
                12
            }
            [0x28, immediate @ _] => {
                let flag_condition = true;
                self.execute_JR_cc_n(Flag::z, flag_condition, immediate);
                12
            }
            [0x30, immediate @ _] => {
                let flag_condition = false;
                self.execute_JR_cc_n(Flag::c, flag_condition, immediate);
                12
            }
            [0x38, immediate @ _] => {
                let flag_condition = true;
                self.execute_JR_cc_n(Flag::c, flag_condition, immediate);
                12
            }
            [0xCD, immediate_low @ _, immediate_high @ _] => {
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_CALL_nn(immediate);
                24
            }
            [0xC4, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = false;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_CALL_cc_nn(Flag::z, flag_condition, immediate);
                24
            }
            [0xCC, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = true;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_CALL_cc_nn(Flag::z, flag_condition, immediate);
                24
            }
            [0xD4, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = false;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_CALL_cc_nn(Flag::c, flag_condition, immediate);
                24
            }
            [0xDC, immediate_low @ _, immediate_high @ _] => {
                let flag_condition = true;
                let immediate = &u16::from_le_bytes([*immediate_low, *immediate_high]);
                self.execute_CALL_cc_nn(Flag::c, flag_condition, immediate);
                24
            }
            [0xC7] => {
                self.execute_RST();
                16
            }
            [0xCF] => {
                self.execute_RST();
                16
            }
            [0xD7] => {
                self.execute_RST();
                16
            }
            [0xDF] => {
                self.execute_RST();
                16
            }
            [0xE7] => {
                self.execute_RST();
                16
            }
            [0xEF] => {
                self.execute_RST();
                16
            }
            [0xF7] => {
                self.execute_RST();
                16
            }
            [0xFF] => {
                self.execute_RST();
                16
            }
            [0xC9] => {
                self.execute_RET();
                16
            }
            [0xC0] => {
                let flag_condition = false;
                self.execute_RET_cc(Flag::z, flag_condition);
                20
            }
            [0xC8] => {
                let flag_condition = true;
                self.execute_RET_cc(Flag::z, flag_condition);
                20
            }
            [0xD0] => {
                let flag_condition = false;
                self.execute_RET_cc(Flag::c, flag_condition);
                20
            }
            [0xD8] => {
                let flag_condition = true;
                self.execute_RET_cc(Flag::c, flag_condition);
                20
            }
            // __OPCODES_DECODING_REPLACEMENT_END__
            _ => {
                let formatted_instruction = utils::format_hex(instruction_bytes);
                panic!("Unsupported instruction!: {}", formatted_instruction)
            }
        }
    }

    // __OPCODES_EXECUTION_REPLACEMENT_START__
    fn execute_LD_r_n(&mut self, dst_register: Reg8, immediate: &u8) {
        self[Reg16::PC] += 2;

        self[dst_register] = *immediate;

    }

    fn execute_LD_r1_r2(&mut self, dst_register: Reg8, src_register: Reg8) {
        self[Reg16::PC] += 1;

        self[dst_register] = self[src_register];

    }

    fn execute_LD_r1_Irr2(&mut self, dst_register: Reg8, src_register: Reg16) {
        self[Reg16::PC] += 1;

        self[dst_register] = self.internal_ram[self[src_register] as usize];

    }

    fn execute_LD_Irr1_r2(&mut self, dst_register: Reg16, src_register: Reg8) {
        self[Reg16::PC] += 1;

        self.internal_ram[self[dst_register] as usize] = self[src_register];

    }

    fn execute_LD_IHL_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        self.internal_ram[self[Reg16::HL] as usize] = *immediate;

    }

    fn execute_LD_A_Inn(&mut self, immediate: &u16) {
        self[Reg16::PC] += 3;

        self[Reg8::A] = self.internal_ram[*immediate as usize];

    }

    fn execute_LD_Inn_A(&mut self, immediate: &u16) {
        self[Reg16::PC] += 3;

        self.internal_ram[*immediate as usize] = self[Reg8::A];

    }

    fn execute_LD_A_IC(&mut self) {
        self[Reg16::PC] += 1;

        let address = 0xFF00 + self[Reg8::C] as usize;
        self[Reg8::A] = self.internal_ram[address];

    }

    fn execute_LD_IC_A(&mut self) {
        self[Reg16::PC] += 1;

        let address = 0xFF00 + self[Reg8::C] as usize;
        self.internal_ram[address] = self[Reg8::A];

    }

    fn execute_LDD_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        self[Reg8::A] = self.internal_ram[self[Reg16::HL] as usize];

        let (new_value, _) = self[Reg16::HL].overflowing_sub(1);
        self[Reg16::HL] = new_value;

    }

    fn execute_LDD_IHL_A(&mut self) {
        self[Reg16::PC] += 1;

        self.internal_ram[self[Reg16::HL] as usize] = self[Reg8::A];

        let (new_value, _) = self[Reg16::HL].overflowing_sub(1);
        self[Reg16::HL] = new_value;

    }

    fn execute_LDI_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        self[Reg8::A] = self.internal_ram[self[Reg16::HL] as usize];

        let (new_value, _) = self[Reg16::HL].overflowing_add(1);
        self[Reg16::HL] = new_value;

    }

    fn execute_LDI_IHL_A(&mut self) {
        self[Reg16::PC] += 1;

        self.internal_ram[self[Reg16::HL] as usize] = self[Reg8::A];

        let (new_value, _) = self[Reg16::HL].overflowing_add(1);
        self[Reg16::HL] = new_value;

    }

    fn execute_LDH_In_A(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let address = 0xFF00 + *immediate as usize;
        self.internal_ram[address] = self[Reg8::A];

    }

    fn execute_LDH_A_In(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let address = 0xFF00 + *immediate as usize;
        self[Reg8::A] = self.internal_ram[address];

    }

    fn execute_LD_rr_nn(&mut self, dst_register: Reg16, immediate: &u16) {
        self[Reg16::PC] += 3;

        self[dst_register] = *immediate;

    }

    fn execute_LD_SP_HL(&mut self) {
        self[Reg16::PC] += 1;

        self[Reg16::SP] = self[Reg16::HL];

    }

    fn execute_LDHL_SP_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let operand1 = self[Reg16::SP];
        // Ugly, but required, conversions.
        let operand2 = *immediate as i8 as i16 as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg16::HL] = result;

        self.set_flag(Flag::z, false);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
        let flag_c_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 8);
        self.set_flag(Flag::c, flag_c_value);
    }

    fn execute_LD_Inn_SP(&mut self, immediate: &u16) {
        self[Reg16::PC] += 3;

        self.internal_ram[*immediate as usize] = self[Reg16::SP] as u8;
        self.internal_ram[*immediate as usize + 1] = (self[Reg16::SP] >> 8) as u8;

    }

    fn execute_PUSH_rr(&mut self, dst_register: Reg16) {
        self[Reg16::PC] += 1;

        let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
        self[Reg16::SP] = new_sp;

        let pushed_bytes = self[dst_register].to_le_bytes();
        self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);

    }

    fn execute_POP_rr(&mut self, dst_register: Reg16) {
        self[Reg16::PC] += 1;

        let source_bytes = self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap();
        self[dst_register] = u16::from_le_bytes(source_bytes);

        let (result, _) = self[Reg16::SP].overflowing_add(2);
        self[Reg16::SP] = result;

    }

    fn execute_POP_AF(&mut self) {
        self[Reg16::PC] += 1;

        let source_bytes = self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap();
        self[Reg16::AF] = u16::from_le_bytes(source_bytes) & 0b1111_1111_1111_0000;

        let (result, _) = self[Reg16::SP].overflowing_add(2);
        self[Reg16::SP] = result;

        // self.set_flag(Flag::h, phony);
        // self.set_flag(Flag::z, phony);
        // self.set_flag(Flag::c, phony);
        // self.set_flag(Flag::n, phony);

    }

    fn execute_ADD_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_ADD_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A];
        let operand2 = self.internal_ram[self[Reg16::HL] as usize];

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_ADD_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let operand1 = self[Reg8::A];
        let operand2 = *immediate;

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_ADC_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A] as u16;
        let operand2 = self[dst_register] as u16 + self.get_flag(Flag::c) as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result as u8;

        let carry_set = (result & 0b1_0000_0000) != 0;
        self.set_flag(Flag::c, carry_set);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_ADC_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A] as u16;
        let operand2 = self.internal_ram[self[Reg16::HL] as usize] as u16 + self.get_flag(Flag::c) as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result as u8;

        let carry_set = (result & 0b1_0000_0000) != 0;
        self.set_flag(Flag::c, carry_set);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_ADC_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let operand1 = self[Reg8::A] as u16;
        let operand2 = *immediate as u16 + self.get_flag(Flag::c) as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result as u8;

        let carry_set = (result & 0b1_0000_0000) != 0;
        self.set_flag(Flag::c, carry_set);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_SUB_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);

        self.set_flag(Flag::z, result == 0);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_SUB_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A];
        let operand2 = self.internal_ram[self[Reg16::HL] as usize];

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);

        self.set_flag(Flag::z, result == 0);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_SUB_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let operand1 = self[Reg8::A];
        let operand2 = *immediate;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);

        self.set_flag(Flag::z, result == 0);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_SBC_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A] as u16;
        let operand2 = self[dst_register] as u16 + self.get_flag(Flag::c) as u16;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result as u8;

        self.set_flag(Flag::c, carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, true);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_SBC_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A] as u16;
        let operand2 = self.internal_ram[self[Reg16::HL] as usize] as u16 + self.get_flag(Flag::c) as u16;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result as u8;

        self.set_flag(Flag::c, carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, true);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_SBC_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let operand1 = self[Reg8::A] as u16;
        let operand2 = *immediate as u16 + self.get_flag(Flag::c) as u16;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result as u8;

        self.set_flag(Flag::c, carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, true);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_AND_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let result = self[Reg8::A] & self[dst_register];
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, true);
        self.set_flag(Flag::c, false);
    }

    fn execute_AND_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let result = self[Reg8::A] & self.internal_ram[self[Reg16::HL] as usize];
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, true);
        self.set_flag(Flag::c, false);
    }

    fn execute_AND_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let result = self[Reg8::A] & *immediate;
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, true);
        self.set_flag(Flag::c, false);
    }

    fn execute_OR_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let result = self[Reg8::A] | self[dst_register];
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_OR_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let result = self[Reg8::A] | self.internal_ram[self[Reg16::HL] as usize];
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_OR_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let result = self[Reg8::A] | *immediate;
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_XOR_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let result = self[Reg8::A] ^ self[dst_register];
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_XOR_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let result = self[Reg8::A] ^ self.internal_ram[self[Reg16::HL] as usize];
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_XOR_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let result = self[Reg8::A] ^ *immediate;
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_CP_A_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_sub(operand2);

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);

        self.set_flag(Flag::z, result == 0);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_CP_A_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg8::A];
        let operand2 = self.internal_ram[self[Reg16::HL] as usize];

        let (result, carry) = operand1.overflowing_sub(operand2);

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);

        self.set_flag(Flag::z, result == 0);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_CP_A_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let operand1 = self[Reg8::A];
        let operand2 = *immediate;

        let (result, carry) = operand1.overflowing_sub(operand2);

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);

        self.set_flag(Flag::z, result == 0);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_INC_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_add(operand2);
        self[dst_register] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_INC_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let operand1 = self.internal_ram[self[Reg16::HL] as usize];
        let operand2 = 1;
        let (result, _) = operand1.overflowing_add(operand2);
        self.internal_ram[self[Reg16::HL] as usize] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_DEC_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 1;

        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_sub(operand2);
        self[dst_register] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, true);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_DEC_IHL(&mut self) {
        self[Reg16::PC] += 1;

        let operand1 = self.internal_ram[self[Reg16::HL] as usize];
        let operand2 = 1;
        let (result, _) = operand1.overflowing_sub(operand2);
        self.internal_ram[self[Reg16::HL] as usize] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, true);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_ADD_HL_rr(&mut self, dst_register: Reg16) {
        self[Reg16::PC] += 1;

        let operand1 = self[Reg16::HL];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg16::HL] = result;

        self.set_flag(Flag::c, carry);

        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 12);
        self.set_flag(Flag::h, flag_h_value);
    }

    fn execute_ADD_SP_n(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let operand1 = self[Reg16::SP];
        // Ugly, but required, conversions.
        let operand2 = *immediate as i8 as i16 as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg16::SP] = result;

        self.set_flag(Flag::z, false);
        self.set_flag(Flag::n, false);
        let flag_h_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 4);
        self.set_flag(Flag::h, flag_h_value);
        let flag_c_value = Cpu::compute_carry_flag(operand1 as u16, operand2 as u16, result as u16, 8);
        self.set_flag(Flag::c, flag_c_value);
    }

    fn execute_INC_rr(&mut self, dst_register: Reg16) {
        self[Reg16::PC] += 1;

        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_add(operand2);
        self[dst_register] = result;

    }

    fn execute_DEC_rr(&mut self, dst_register: Reg16) {
        self[Reg16::PC] += 1;

        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_sub(operand2);
        self[dst_register] = result;

    }

    fn execute_SWAP_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        let result = self[dst_register] >> 4 | ((self[dst_register] & 0b0000_1111) << 4);
        self[dst_register] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_SWAP_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let value = self.internal_ram[self[Reg16::HL] as usize];
        let result = value >> 4 | ((value & 0b0000_1111) << 4);
        self.internal_ram[self[Reg16::HL] as usize] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
        self.set_flag(Flag::c, false);
    }

    fn execute_DAA(&mut self) {
        self[Reg16::PC] += 1;

        let mut result = self[Reg8::A] as u16;

        self[Reg8::A] = 0x00;
        self.set_flag(Flag::z, true);

        if self.get_flag(Flag::n) {
            if self.get_flag(Flag::h) {
                result = (result - 0x06) & 0xFF;
            }

            if self.get_flag(Flag::c) {
                result -= 0x60;
            }
        }
        else {
            if self.get_flag(Flag::h) || (result & 0x0F) > 0x09 {
                result += 0x06;
            }

            if self.get_flag(Flag::c) || result > 0x9F {
                result += 0x60;
            }
        }

        if (result & 0xFF) == 0 {
            self.set_flag(Flag::z, true);
        }

        if (result & 0x100) == 0x100 {
            self.set_flag(Flag::c, true);
        }

        self.set_flag(Flag::h, false);
        self[Reg8::A] = result as u8;

    }

    fn execute_CPL(&mut self) {
        self[Reg16::PC] += 1;

        self[Reg8::A] = !self[Reg8::A];

        self.set_flag(Flag::n, true);
        self.set_flag(Flag::h, true);
    }

    fn execute_CCF(&mut self) {
        self[Reg16::PC] += 1;

        let cf_value = self.get_flag(Flag::c);
        self.set_flag(Flag::c, !cf_value);

        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_SCF(&mut self) {
        self[Reg16::PC] += 1;

        self.set_flag(Flag::c, true);

        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_NOP(&mut self) {
        self[Reg16::PC] += 1;


    }

    fn execute_RLCA(&mut self) {
        self[Reg16::PC] += 1;

        self.set_flag(Flag::c, (self[Reg8::A] & 0b1000_0000) != 0);
        let result = self[Reg8::A].rotate_left(1);
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RLA(&mut self) {
        self[Reg16::PC] += 1;

        let new_carry = (self[Reg8::A] & 0b1000_0000) != 0;

        let result = self[Reg8::A].wrapping_shl(1) | self.get_flag(Flag::c) as u8;
        self[Reg8::A] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RRCA(&mut self) {
        self[Reg16::PC] += 1;

        self.set_flag(Flag::c, (self[Reg8::A] & 0b0000_0001) != 0);
        let result = self[Reg8::A].rotate_right(1);
        self[Reg8::A] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RRA(&mut self) {
        self[Reg16::PC] += 1;

        let new_carry = (self[Reg8::A] & 0b0000_0001) != 0;

        let mut result = self[Reg8::A].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self[Reg8::A] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RLC_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        self.set_flag(Flag::c, (self[dst_register] & 0b1000_0000) != 0);
        let result = self[dst_register].rotate_left(1);
        self[dst_register] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RLC_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;

        self.set_flag(Flag::c, (self.internal_ram[address] & 0b1000_0000) != 0);
        let result = self.internal_ram[address].rotate_left(1);

        self.internal_ram[address] = result;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RL_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        let new_carry = (self[dst_register] & 0b1000_0000) != 0;

        let result = self[dst_register].wrapping_shl(1) | self.get_flag(Flag::c) as u8;
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RL_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b1000_0000) != 0;

        let result = self.internal_ram[address].wrapping_shl(1) | self.get_flag(Flag::c) as u8;
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RRC_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        let new_carry = (self[dst_register] & 0b0000_0001) != 0;

        let mut result = self[dst_register].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RRC_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b0000_0001) != 0;

        let mut result = self.internal_ram[address].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RR_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        let new_carry = (self[dst_register] & 0b0000_0001) != 0;

        let mut result = self[dst_register].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_RR_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b0000_0001) != 0;

        let mut result = self.internal_ram[address].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_SLA_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        let new_carry = (self[dst_register] & 0b1000_0000) != 0;

        let result = self[dst_register].wrapping_shl(1);
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_SLA_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b1000_0000) != 0;

        let result = self.internal_ram[address].wrapping_shl(1);
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_SRA_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        let new_carry = (self[dst_register] & 0b0000_0001) != 0;
        let old_msb = self[dst_register] & 0b1000_0000;

        let result = self[dst_register].wrapping_shr(1) | old_msb;
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_SRA_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;

        let new_carry = (self.internal_ram[address] & 0b0000_0001) != 0;
        let old_msb = self.internal_ram[address] & 0b1000_0000;

        let result = self.internal_ram[address].wrapping_shr(1) | old_msb;
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_SRL_r(&mut self, dst_register: Reg8) {
        self[Reg16::PC] += 2;

        let new_carry = (self[dst_register] & 0b1000_0000) != 0;

        let result = self[dst_register].wrapping_shl(1);
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_SRL_IHL(&mut self) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b1000_0000) != 0;

        let result = self.internal_ram[address].wrapping_shl(1);
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, false);
    }

    fn execute_BIT_n_r(&mut self, immediate: &u8, src_register: Reg8) {
        self[Reg16::PC] += 2;

        let bitmask = 1 << *immediate;

        let result = self[src_register] & bitmask;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, true);
    }

    fn execute_BIT_n_IHL(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let bitmask = 1 << *immediate;

        let result = self.internal_ram[address] & bitmask;

        self.set_flag(Flag::z, result == 0);
        self.set_flag(Flag::n, false);
        self.set_flag(Flag::h, true);
    }

    fn execute_SET_n_r(&mut self, immediate: &u8, src_register: Reg8) {
        self[Reg16::PC] += 2;

        let bitmask = 1 << *immediate;

        self[src_register] |= bitmask;

    }

    fn execute_SET_n_IHL(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let bitmask = 1 << *immediate;

        self.internal_ram[address] |= bitmask;

    }

    fn execute_RES_n_r(&mut self, immediate: &u8, src_register: Reg8) {
        self[Reg16::PC] += 2;

        let bitmask = !(1 << *immediate);

        self[src_register] &= bitmask;

    }

    fn execute_RES_n_IHL(&mut self, immediate: &u8) {
        self[Reg16::PC] += 2;

        let address = self[Reg16::HL] as usize;
        let bitmask = !(1 << *immediate);

        self.internal_ram[address] &= bitmask;

    }

    fn execute_JP_nn(&mut self, immediate: &u16) {
        self[Reg16::PC] = *immediate;

    }

    fn execute_JP_cc_nn(&mut self, flag: Flag, flag_condition: bool, immediate: &u16) {
        if self.get_flag(flag) == flag_condition {
          self[Reg16::PC] = *immediate;
        }
        else {
          self[Reg16::PC] += 3;
        }

    }

    fn execute_JP_IHL(&mut self) {
        self[Reg16::PC] = self[Reg16::HL];

    }

    fn execute_JR_n(&mut self, immediate: &u8) {
        let operand1 = self[Reg16::PC];
        let operand2 = *immediate as i8 as i16 as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg16::PC] = result;

    }

    fn execute_JR_cc_n(&mut self, flag: Flag, flag_condition: bool, immediate: &u8) {
        if self.get_flag(flag) == flag_condition {
          let operand1 = self[Reg16::PC];
          let operand2 = *immediate as i8 as i16 as u16;

          let (result, _) = operand1.overflowing_add(operand2);
          self[Reg16::PC] = result;
        }
        else {
          self[Reg16::PC] += 2;
        }

    }

    fn execute_CALL_nn(&mut self, immediate: &u16) {
        let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
        self[Reg16::SP] = new_sp;

        let (stored_address, _) = self[Reg16::PC].overflowing_add(3);
        let pushed_bytes = stored_address.to_le_bytes();
        self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);

        self[Reg16::PC] = *immediate;

    }

    fn execute_CALL_cc_nn(&mut self, flag: Flag, flag_condition: bool, immediate: &u16) {
        if self.get_flag(flag) == flag_condition {
            let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
            self[Reg16::SP] = new_sp;

            let (stored_address, _) = self[Reg16::PC].overflowing_add(3);
            let pushed_bytes = stored_address.to_le_bytes();
            self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);

            self[Reg16::PC] = *immediate;
        } else {
            self[Reg16::PC] += 3;
        }

    }

    fn execute_RST(&mut self) {
        let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
        self[Reg16::SP] = new_sp;

        let pushed_bytes = self[Reg16::PC].to_le_bytes();
        self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);

        let destination_address = match self.internal_ram[self[Reg16::PC] as usize] {
            0xC7 => 0x00,
            0xCF => 0x08,
            0xD7 => 0x10,
            0xDF => 0x18,
            0xE7 => 0x20,
            0xEF => 0x28,
            0xF7 => 0x30,
            0xFF => 0x38,
            _ => panic!(),
        };

        self[Reg16::PC] = destination_address;

    }

    fn execute_RET(&mut self) {
        self[Reg16::PC] = u16::from_le_bytes(self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap());

        let (new_sp, _) = self[Reg16::SP].overflowing_add(2);
        self[Reg16::SP] = new_sp;

    }

    fn execute_RET_cc(&mut self, flag: Flag, flag_condition: bool) {
        if self.get_flag(flag) == flag_condition {
            self[Reg16::PC] = u16::from_le_bytes(self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap());

            let (new_sp, _) = self[Reg16::SP].overflowing_add(2);
            self[Reg16::SP] = new_sp;
        } else {
            self[Reg16::PC] += 1;
        }

    }
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
