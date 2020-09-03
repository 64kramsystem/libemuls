use crate::cpu::Cpu;
use ruspec::ruspec;

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
