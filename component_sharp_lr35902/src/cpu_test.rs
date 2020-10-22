#![allow(unused_macros)]

use crate::cpu::{Cpu, Flag, Reg16, Reg8};
use demonstrate::demonstrate;
use strum::IntoEnumIterator;

fn assert_cpu_execute(
    cpu: &mut Cpu,
    instruction_bytes: &[u8],
    A: Option<u8>,
    F: Option<u8>,
    B: Option<u8>,
    C: Option<u8>,
    D: Option<u8>,
    E: Option<u8>,
    H: Option<u8>,
    L: Option<u8>,
    AF: Option<u16>,
    BC: Option<u16>,
    DE: Option<u16>,
    HL: Option<u16>,
    SP: Option<u16>,
    PC: Option<u16>,
    zf: Option<bool>,
    nf: Option<bool>,
    hf: Option<bool>,
    cf: Option<bool>,
    mem: Option<(u16, &[u8])>,
    cycles_spent: u8,
) {
    let actual_cycles_spent = cpu.execute(&instruction_bytes);

    for (register, value) in Reg8::iter().zip([A, F, B, C, D, E, H, L].iter()) {
        if let Some(value) = value {
            assert_eq!(
                cpu[register], *value,
                "Unexpected `{:?}`: actual=0x{:02X}, expected=0x{:02X}",
                register, cpu[register], *value
            );
        }
    }

    for (register, value) in Reg16::iter().zip([AF, BC, DE, HL, SP, PC].iter()) {
        if let Some(value) = value {
            assert_eq!(
                cpu[register], *value,
                "Unexpected `{:?}`: actual=0x{:02X}, expected=0x{:02X}",
                register, cpu[register], *value
            );
        }
    }

    for (flag, value) in Flag::iter().zip([zf, nf, hf, cf].iter()) {
        if let Some(value) = value {
            assert_eq!(
                cpu.get_flag(flag),
                *value,
                "Unexpected `{:?}`: actual={}, expected={}",
                flag,
                cpu.get_flag(flag) as u8,
                *value as u8
            );
        }
    }

    if let Some((start_address, expected_values)) = mem {
        for i in 0..(expected_values.len()) {
            let address = start_address as usize + i;
            let actual_value = cpu.internal_ram[address];
            let expected_value = expected_values[i];

            assert_eq!(
                actual_value, expected_value,
                "Unexpected mem[0x{:04X}]: actual=0x{:02X}, expected=0x{:02X}",
                address, actual_value, expected_value,
            );
        }
    }

    assert_eq!(actual_cycles_spent, cycles_spent);
}

// The reason for this macro was originally to make the registers/flags visible in the assert_cpu_execute()
// call - the IDE would normally show them, but due to limitations somewhere in the toolset, this
// doesn't happen inside [the used] macros.
//
// While writing the macro, it became evident that making the parameters optional would make the UT
// expectations very neat.
//
// Up to a certain point, the macro implicitly tested the register not specified, however, after 16-bit
// registers were added, and in particular AF, the logic became too tangled, so the macro now tests
// only what specified.
//
macro_rules! assert_cpu_execute {
    (
        $cpu:ident,
        $instruction_bytes:ident,
        $( A => $expected_A:literal , )?
        $( F => $expected_F:literal , )?
        $( B => $expected_B:literal , )?
        $( C => $expected_C:literal , )?
        $( D => $expected_D:literal , )?
        $( E => $expected_E:literal , )?
        $( H => $expected_H:literal , )?
        $( L => $expected_L:literal , )?
        $( AF => $expected_AF:literal , )?
        $( BC => $expected_BC:literal , )?
        $( DE => $expected_DE:literal , )?
        $( HL => $expected_HL:literal , )?
        $( SP => $expected_SP:literal , )?
        $( PC => $expected_PC:literal , )?
        $( zf => $expected_zf:literal , )?
        $( nf => $expected_nf:literal , )?
        $( hf => $expected_hf:literal , )?
        $( cf => $expected_cf:literal , )?
        $( mem[$mem_address:literal] => [$( $mem_value:expr ),+] , )?
        cycles: $cycles:literal
    ) => {
        // Middle ground between a giant assignment, and a platoon of single ones.

        #[allow(unused_mut, unused_assignments)]
        let (mut A, mut F, mut B, mut C, mut D, mut E, mut H, mut L) = (None, None, None, None, None, None, None, None);
        #[allow(unused_mut, unused_assignments)]
        let (mut AF, mut BC, mut DE, mut HL, mut SP, mut PC) = (None, None, None, None, None, None);
        #[allow(unused_mut, unused_assignments)]
        let (mut zf, mut nf, mut hf, mut cf) = (None, None, None, None);
        #[allow(unused_mut, unused_assignments)]
        let mut mem = None::<(u16, &[u8])>;

        $( A = Some($expected_A); )?
        $( F = Some($expected_F); )?
        $( B = Some($expected_B); )?
        $( C = Some($expected_C); )?
        $( D = Some($expected_D); )?
        $( E = Some($expected_E); )?
        $( H = Some($expected_H); )?
        $( L = Some($expected_L); )?
        $( AF = Some($expected_AF); )?
        $( BC = Some($expected_BC); )?
        $( DE = Some($expected_DE); )?
        $( HL = Some($expected_HL); )?
        $( SP = Some($expected_SP); )?
        $( PC = Some($expected_PC); )?
        $( zf = Some($expected_zf); )?
        $( nf = Some($expected_nf); )?
        $( hf = Some($expected_hf); )?
        $( cf = Some($expected_cf); )?
        $(
        let expected_mem_values = &[$( $mem_value ),*][..];
        mem = Some(($mem_address, expected_mem_values));
        )?

        assert_cpu_execute(
            &mut $cpu,
            &$instruction_bytes,
            A, F, B, C, D, E, H, L,
            AF, BC, DE, HL, SP, PC,
            zf, nf, hf, cf,
            mem,
            $cycles,
        )
    };
}

demonstrate! {
    describe "CPU" {
        use super::*;

        before {
          // (Current) issue with declarative testing frameworks; see https://git.io/JUlar.
          //
          #[allow(unused_mut)]
          let mut cpu = Cpu::new();
        }

        // Can't really test random, but it's good practice to just make sure it's not been initialized
        // with zeros.
        // This test will fail for near-impossibly unlucky runs (or lucky, depending on the perspective).
        //
        it "initializes" {
            let internal_ram_sum: u32 = cpu.internal_ram.to_vec().iter().map(|&x| x as u32).sum();

            assert_ne!(internal_ram_sum, 0);

            assert_eq!(cpu[Reg8::A], 0);
            assert_eq!(cpu[Reg16::BC], 0);
            assert_eq!(cpu[Reg16::DE], 0);
            assert_eq!(cpu[Reg16::HL], 0);
            assert_eq!(cpu[Reg16::SP], 0);
            assert_eq!(cpu[Reg16::PC], 0);

            assert_eq!(cpu.get_flag(Flag::z), false);
            assert_eq!(cpu.get_flag(Flag::n), false);
            assert_eq!(cpu.get_flag(Flag::h), false);
            assert_eq!(cpu.get_flag(Flag::c), false);
        }

        context "executes" {
            // __TESTS_REPLACEMENT_START__
            // __TESTS_REPLACEMENT_END__
          }
    }
}
