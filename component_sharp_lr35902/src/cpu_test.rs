use crate::cpu::Cpu;
use demonstrate::demonstrate;

fn assert_cpu_execute(
    cpu: &mut Cpu,
    instruction_bytes: &[u8],
    pre_A: u8,
    pre_B: u8,
    pre_C: u8,
    pre_D: u8,
    pre_E: u8,
    pre_H: u8,
    pre_L: u8,
    pre_SP: u16,
    pre_PC: u16,
    pre_zf: bool,
    pre_nf: bool,
    pre_hf: bool,
    pre_cf: bool,
    post_A: u8,
    post_B: u8,
    post_C: u8,
    post_D: u8,
    post_E: u8,
    post_H: u8,
    post_L: u8,
    post_SP: u16,
    post_PC: u16,
    post_zf: bool,
    post_nf: bool,
    post_hf: bool,
    post_cf: bool,
    cycles_spent: u8,
) {
    cpu.zf = pre_zf;
    cpu.nf = pre_nf;
    cpu.hf = pre_hf;
    cpu.cf = pre_cf;

    cpu.A = pre_A;
    cpu.B = pre_B;
    cpu.C = pre_C;
    cpu.D = pre_D;
    cpu.E = pre_E;
    cpu.H = pre_H;
    cpu.L = pre_L;
    cpu.SP = pre_SP;
    cpu.PC = pre_PC;

    let actual_cycles_spent = cpu.execute(&instruction_bytes);

    assert_eq!(cpu.A, post_A);
    assert_eq!(cpu.B, post_B);
    assert_eq!(cpu.C, post_C);
    assert_eq!(cpu.D, post_D);
    assert_eq!(cpu.E, post_E);
    assert_eq!(cpu.H, post_H);
    assert_eq!(cpu.L, post_L);
    assert_eq!(cpu.SP, post_SP);
    assert_eq!(cpu.PC, post_PC);

    assert_eq!(cpu.zf, post_zf);
    assert_eq!(cpu.nf, post_nf);
    assert_eq!(cpu.hf, post_hf);
    assert_eq!(cpu.cf, post_cf);

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
        $( A: $A_pre_value:literal => $A_post_value:literal , )?
        $( B: $B_pre_value:literal => $B_post_value:literal , )?
        $( C: $C_pre_value:literal => $C_post_value:literal , )?
        $( D: $D_pre_value:literal => $D_post_value:literal , )?
        $( E: $E_pre_value:literal => $E_post_value:literal , )?
        $( H: $H_pre_value:literal => $H_post_value:literal , )?
        $( L: $L_pre_value:literal => $L_post_value:literal , )?
        $( SP: $SP_pre_value:literal => $SP_post_value:literal , )?
        $( PC: $PC_pre_value:literal => $PC_post_value:literal , )?
        $( zf: $zf_pre_value:literal => $zf_post_value:literal , )?
        $( nf: $nf_pre_value:literal => $nf_post_value:literal , )?
        $( hf: $hf_pre_value:literal => $hf_post_value:literal , )?
        $( cf: $cf_pre_value:literal => $cf_post_value:literal , )?
        cycles: $cycles:literal
) => {
        // Alternatives to this have been evaluated here: https://users.rust-lang.org/t/any-way-to-cleanly-set-a-default-value-for-a-pseudo-named-parameter-in-a-macro/48682/6
        // A simple, interesting, alternative is to pass the variables to an adhoc struct with
        // default(), however, it's not a radical improvement.
        //
        let pre_A = 0x21 $( - 0x21 + $A_pre_value )?;
        let post_A = 0x21 $( - 0x21 + $A_post_value )?;

        let pre_B = 0x21 $( - 0x21 + $B_pre_value )?;
        let post_B = 0x21 $( - 0x21 + $B_post_value )?;

        let pre_C = 0x21 $( - 0x21 + $C_pre_value )?;
        let post_C = 0x21 $( - 0x21 + $C_post_value )?;

        let pre_D = 0x21 $( - 0x21 + $D_pre_value )?;
        let post_D = 0x21 $( - 0x21 + $D_post_value )?;

        let pre_E = 0x21 $( - 0x21 + $E_pre_value )?;
        let post_E = 0x21 $( - 0x21 + $E_post_value )?;

        let pre_H = 0x21 $( - 0x21 + $H_pre_value )?;
        let post_H = 0x21 $( - 0x21 + $H_post_value )?;

        let pre_L = 0x21 $( - 0x21 + $L_pre_value )?;
        let post_L = 0x21 $( - 0x21 + $L_post_value )?;

        let pre_SP = 0x21 $( - 0x21 + $SP_pre_value )?;
        let post_SP = 0x21 $( - 0x21 + $SP_post_value )?;

        let pre_PC = 0x21 $( - 0x21 + $PC_pre_value )?;
        let post_PC = 0x21 $( - 0x21 + $PC_post_value )?;

        let pre_zf = if (0 $( + $zf_pre_value )?) == 0 { false } else { true };
        let post_zf = if (0 $( + $zf_post_value )?) == 0 { false } else { true };

        let pre_nf = if (0 $( + $nf_pre_value )?) == 0 { false } else { true };
        let post_nf = if (0 $( + $nf_post_value )?) == 0 { false } else { true };

        let pre_hf = if (0 $( + $hf_pre_value )?) == 0 { false } else { true };
        let post_hf = if (0 $( + $hf_post_value )?) == 0 { false } else { true };

        let pre_cf = if (0 $( + $cf_pre_value )?) == 0 { false } else { true };
        let post_cf = if (0 $( + $cf_post_value )?) == 0 { false } else { true };

        assert_cpu_execute(
            &mut $cpu,
            &$instruction_bytes,
            pre_A, pre_B, pre_C, pre_D, pre_E, pre_H, pre_L, pre_SP, pre_PC,
            pre_zf, pre_nf, pre_hf, pre_cf,
            post_A, post_B, post_C, post_D, post_E, post_H, post_L, post_SP, post_PC,
            post_zf, post_nf, post_hf, post_cf,
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
            // __TESTS_REPLACEMENT_POINT__
        }
    }
}
