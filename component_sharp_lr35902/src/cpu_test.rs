#![allow(unused_macros)]

use crate::cpu::{Cpu, Flag, Register16, Register8};
use demonstrate::demonstrate;

fn assert_cpu_execute(
    cpu: &mut Cpu,
    instruction_bytes: &[u8],
    A: u8,
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    H: u8,
    L: u8,
    SP: u16,
    PC: u16,
    zf: bool,
    nf: bool,
    hf: bool,
    cf: bool,
    mem: Option<(u16, u8)>,
    cycles_spent: u8,
) {
    let actual_cycles_spent = cpu.execute(&instruction_bytes);

    assert_eq!(
        cpu[Register8::A],
        A,
        "Unexpected `A`: actual={}, expected={}",
        cpu[Register8::A],
        A
    );
    assert_eq!(
        cpu[Register8::B],
        B,
        "Unexpected `B`: actual={}, expected={}",
        cpu[Register8::B],
        B
    );
    assert_eq!(
        cpu[Register8::C],
        C,
        "Unexpected `C`: actual={}, expected={}",
        cpu[Register8::C],
        C
    );
    assert_eq!(
        cpu[Register8::D],
        D,
        "Unexpected `D`: actual={}, expected={}",
        cpu[Register8::D],
        D
    );
    assert_eq!(
        cpu[Register8::E],
        E,
        "Unexpected `E`: actual={}, expected={}",
        cpu[Register8::E],
        E
    );
    assert_eq!(
        cpu[Register8::H],
        H,
        "Unexpected `H`: actual={}, expected={}",
        cpu[Register8::H],
        H
    );
    assert_eq!(
        cpu[Register8::L],
        L,
        "Unexpected `L`: actual={}, expected={}",
        cpu[Register8::L],
        L
    );
    assert_eq!(
        cpu[Register16::SP],
        SP,
        "Unexpected `SP`: actual={}, expected={}",
        cpu[Register16::SP],
        SP
    );
    assert_eq!(
        cpu[Register16::PC],
        PC,
        "Unexpected `PC`: actual={}, expected={}",
        cpu[Register16::PC],
        PC
    );

    assert_eq!(
        cpu[Flag::z],
        zf,
        "Unexpected `zf`: actual={}, expected={}",
        cpu[Flag::z] as u8,
        zf as u8
    );
    assert_eq!(
        cpu[Flag::n],
        nf,
        "Unexpected nf: actual={}, expected={}",
        cpu[Flag::n] as u8,
        nf as u8
    );
    assert_eq!(
        cpu[Flag::h],
        hf,
        "Unexpected `hf`: actual={}, expected={}",
        cpu[Flag::h] as u8,
        hf as u8
    );
    assert_eq!(
        cpu[Flag::c],
        cf,
        "Unexpected `cf`: actual={}, expected={}",
        cpu[Flag::c] as u8,
        cf as u8
    );

    if let Some((mem_address, mem_value)) = mem {
        let actual_value = cpu.internal_ram[mem_address as usize];
        assert_eq!(
            actual_value, mem_value,
            "Unexpected mem[{}]: actual={}, expected={}",
            mem_address, actual_value, mem_value,
        );
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
macro_rules! assert_cpu_execute {
    (
        $cpu:ident,
        $instruction_bytes:ident,
        $( A => $expected_A:literal , )?
        $( B => $expected_B:literal , )?
        $( C => $expected_C:literal , )?
        $( D => $expected_D:literal , )?
        $( E => $expected_E:literal , )?
        $( H => $expected_H:literal , )?
        $( L => $expected_L:literal , )?
        $( BC => $expected_BC:literal , )?
        $( DE => $expected_DE:literal , )?
        $( HL => $expected_HL:literal , )?
        $( SP => $expected_SP:literal , )?
        $( PC => $expected_PC:literal , )?
        $( zf => $expected_zf:literal , )?
        $( nf => $expected_nf:literal , )?
        $( hf => $expected_hf:literal , )?
        $( cf => $expected_cf:literal , )?
        $( mem[$mem_address:literal] => $mem_value:expr, )?
        cycles: $cycles:literal
) => {
        let current_A = $cpu[Reg8::A];
        let current_B = $cpu[Reg8::B];
        let current_C = $cpu[Reg8::C];
        let current_D = $cpu[Reg8::D];
        let current_E = $cpu[Reg8::E];
        let current_H = $cpu[Reg8::H];
        let current_L = $cpu[Reg8::L];
        let current_SP = $cpu[Reg16::SP];
        let current_PC = $cpu[Reg16::PC];
        let current_zf = $cpu[Flag::z];
        let current_nf = $cpu[Flag::n];
        let current_hf = $cpu[Flag::h];
        let current_cf = $cpu[Flag::c];

        // Alternatives to this have been evaluated here: https://users.rust-lang.org/t/any-way-to-cleanly-set-a-default-value-for-a-pseudo-named-parameter-in-a-macro/48682/6
        // A simple, interesting, alternative is to pass the variables to an adhoc struct with
        // default(), however, it's not a radical improvement.
        //
        let A = current_A $( - current_A + $expected_A )?;

        #[allow(unused_mut, unused_assignments)]
        let mut B = current_B $( - current_B + $expected_B )?;
        #[allow(unused_mut, unused_assignments)]
        let mut C = current_C $( - current_C + $expected_C )?;
        $(
        B = ($expected_BC >> 8) as u8;
        C = ($expected_BC & 0b1111_1111) as u8;
        )?

        #[allow(unused_mut, unused_assignments)]
        let mut D = current_D $( - current_D + $expected_D )?;
        #[allow(unused_mut, unused_assignments)]
        let mut E = current_E $( - current_E + $expected_E )?;

        $(
        D = ($expected_DE >> 8) as u8;
        E = ($expected_DE & 0b1111_1111) as u8;
        )?

        #[allow(unused_mut, unused_assignments)]
        let mut H = current_H $( - current_H + $expected_H )?;
        #[allow(unused_mut, unused_assignments)]
        let mut L = current_L $( - current_L + $expected_L )?;

        $(
        H = ($expected_HL >> 8) as u8;
        L = ($expected_HL & 0b1111_1111) as u8;
        )?

        let SP = current_SP $( - current_SP + $expected_SP )?;
        let PC = current_PC $( - current_PC + $expected_PC )?;
        let zf = current_zf $( ^ current_zf | (if $expected_zf == 0 { false } else { true }) )?;
        let nf = current_nf $( ^ current_nf | (if $expected_nf == 0 { false } else { true }) )?;
        let hf = current_hf $( ^ current_hf | (if $expected_hf == 0 { false } else { true }) )?;
        let cf = current_cf $( ^ current_cf | (if $expected_cf == 0 { false } else { true }) )?;

        // The numerical workaround doesn't work here, unless we use some array silliness.
        //
        #[allow(unused_variables)]
        let mem = None::<(u16, u8)>;
        $( let mem = Some(($mem_address, $mem_value)); )?

        assert_cpu_execute(
            &mut $cpu,
            &$instruction_bytes,
            A, B, C, D, E, H, L, SP, PC,
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

            assert_eq!(cpu[Register8::A], 0);
            assert_eq!(cpu[Register8::B], 0);
            assert_eq!(cpu[Register8::C], 0);
            assert_eq!(cpu[Register8::D], 0);
            assert_eq!(cpu[Register8::E], 0);
            assert_eq!(cpu[Register8::H], 0);
            assert_eq!(cpu[Register8::L], 0);
            assert_eq!(cpu[Register16::SP], 0);
            assert_eq!(cpu[Register16::PC], 0);

            assert_eq!(cpu[Flag::z], false);
            assert_eq!(cpu[Flag::n], false);
            assert_eq!(cpu[Flag::h], false);
            assert_eq!(cpu[Flag::c], false);
        }

        context "executes" {
            // __TESTS_REPLACEMENT_START__
            // __TESTS_REPLACEMENT_END__
          }
    }
}
