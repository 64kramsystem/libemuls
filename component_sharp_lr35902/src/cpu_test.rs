use crate::cpu::Cpu;
use ruspec::ruspec;

fn test_cpu_execute(
    cpu: &mut Cpu,
    pre_zf: bool,
    pre_nf: bool,
    pre_hf: bool,
    pre_cf: bool,
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
    post_zf: bool,
    post_nf: bool,
    post_hf: bool,
    post_cf: bool,
    cycles_spent: u8,
) {
    cpu.set_flags(pre_zf, pre_nf, pre_hf, pre_cf);

    let actual_cycles_spent = cpu.execute(&instruction_bytes);

    assert_eq!(cpu.A, A);
    assert_eq!(cpu.B, B);
    assert_eq!(cpu.C, C);
    assert_eq!(cpu.D, D);
    assert_eq!(cpu.E, E);
    assert_eq!(cpu.H, H);
    assert_eq!(cpu.L, L);
    assert_eq!(cpu.SP, SP);
    assert_eq!(cpu.PC, PC);

    assert_eq!(cpu.zf, post_zf);
    assert_eq!(cpu.nf, post_nf);
    assert_eq!(cpu.hf, post_hf);
    assert_eq!(cpu.cf, post_cf);

    assert_eq!(actual_cycles_spent, cycles_spent);
}

ruspec! {
    describe "CPU" {
        before { let cpu = Cpu::new(); }

        // Can't really test random, but it's good practice to just make sure it's not been initialized
        // with zeros.
        // This test will fail for near-impossibly unlucky runs (or lucky, depending on the perspective).
        //
        it "initializes" {
            let internal_ram_sum: u32 = cpu.internal_ram.to_vec().iter().map(|&x| x as u32).sum();

            assert_ne!(internal_ram_sum, 0);
        }
    }
}
