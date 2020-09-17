#![allow(unused_macros)]

use crate::cpu::Cpu;
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

    assert_eq!(cpu.A, A, "Unexpected `A`: actual={}, expected={}", cpu.A, A);
    assert_eq!(cpu.B, B, "Unexpected `B`: actual={}, expected={}", cpu.B, B);
    assert_eq!(cpu.C, C, "Unexpected `C`: actual={}, expected={}", cpu.C, C);
    assert_eq!(cpu.D, D, "Unexpected `D`: actual={}, expected={}", cpu.D, D);
    assert_eq!(cpu.E, E, "Unexpected `E`: actual={}, expected={}", cpu.E, E);
    assert_eq!(cpu.H, H, "Unexpected `H`: actual={}, expected={}", cpu.H, H);
    assert_eq!(cpu.L, L, "Unexpected `L`: actual={}, expected={}", cpu.L, L);
    assert_eq!(
        cpu.SP, SP,
        "Unexpected `SP`: actual={}, expected={}",
        cpu.SP, SP
    );
    assert_eq!(
        cpu.PC, PC,
        "Unexpected `PC`: actual={}, expected={}",
        cpu.PC, PC
    );

    assert_eq!(
        cpu.zf, zf,
        "Unexpected `zf`: actual={}, expected={}",
        cpu.zf as u8, zf as u8
    );
    assert_eq!(
        cpu.nf, nf,
        "Unexpected nf: actual={}, expected={}",
        cpu.nf as u8, nf as u8
    );
    assert_eq!(
        cpu.hf, hf,
        "Unexpected `hf`: actual={}, expected={}",
        cpu.hf as u8, hf as u8
    );
    assert_eq!(
        cpu.cf, cf,
        "Unexpected `cf`: actual={}, expected={}",
        cpu.cf as u8, cf as u8
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
        $( SP => $expected_SP:literal , )?
        $( PC => $expected_PC:literal , )?
        $( zf => $expected_zf:literal , )?
        $( nf => $expected_nf:literal , )?
        $( hf => $expected_hf:literal , )?
        $( cf => $expected_cf:literal , )?
        $( mem[$mem_address:literal] => $mem_value:expr, )?
        cycles: $cycles:literal
) => {
        let current_A = $cpu.A;
        let current_B = $cpu.B;
        let current_C = $cpu.C;
        let current_D = $cpu.D;
        let current_E = $cpu.E;
        let current_H = $cpu.H;
        let current_L = $cpu.L;
        let current_SP = $cpu.SP;
        let current_PC = $cpu.PC;
        let current_zf = $cpu.zf;
        let current_nf = $cpu.nf;
        let current_hf = $cpu.hf;
        let current_cf = $cpu.cf;

        // Alternatives to this have been evaluated here: https://users.rust-lang.org/t/any-way-to-cleanly-set-a-default-value-for-a-pseudo-named-parameter-in-a-macro/48682/6
        // A simple, interesting, alternative is to pass the variables to an adhoc struct with
        // default(), however, it's not a radical improvement.
        //
        let A = current_A $( - current_A + $expected_A )?;
        let B = current_B $( - current_B + $expected_B )?;
        let C = current_C $( - current_C + $expected_C )?;
        let D = current_D $( - current_D + $expected_D )?;
        let E = current_E $( - current_E + $expected_E )?;
        let H = current_H $( - current_H + $expected_H )?;
        let L = current_L $( - current_L + $expected_L )?;
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

            assert_eq!(cpu.A, 0);
            assert_eq!(cpu.B, 0);
            assert_eq!(cpu.C, 0);
            assert_eq!(cpu.D, 0);
            assert_eq!(cpu.E, 0);
            assert_eq!(cpu.H, 0);
            assert_eq!(cpu.L, 0);
            assert_eq!(cpu.SP, 0);
            assert_eq!(cpu.PC, 0);

            assert_eq!(cpu.zf, false);
            assert_eq!(cpu.nf, false);
            assert_eq!(cpu.hf, false);
            assert_eq!(cpu.cf, false);
        }

        context "executes" {
            // __TESTS_REPLACEMENT_START__
            // __TESTS_REPLACEMENT_END__
          }
    }
}
