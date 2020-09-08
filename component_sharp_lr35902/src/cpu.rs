use crate::utils;
use rand::RngCore;

// Preliminary, supersimplified implementation.
// Based on a cursory look at the manuals, there is one operation that mass-sets the flags, so it
// makes sense to store them individually.
// The execution is temporary, designed with in mind the testability only.
//
pub struct Cpu {
    // Registers
    //
    pub A: u8,

    pub B: u8,
    pub C: u8,

    pub D: u8,
    pub E: u8,

    pub H: u8,
    pub L: u8,

    pub SP: u16,
    pub PC: u16,

    // Flags
    //
    pub zf: bool,
    pub nf: bool,
    pub hf: bool,
    pub cf: bool,

    // Internal RAM
    //
    pub internal_ram: [u8; 0x2000],
}

impl Cpu {
    pub fn new() -> Self {
        let mut internal_ram = [0; 0x2000];
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
            [0x00] => self.execute_nop(),
            _ => {
                let formatted_instruction = utils::format_hex(instruction_bytes);
                panic!("Unsupported instruction!: {}", formatted_instruction)
            }
        }
    }

    fn execute_nop(&mut self) -> u8 {
        self.PC += 1;

        4
    }

    // HELPERS /////////////////////////////////////////////////////////////////////////////////////

    pub fn HL_location(&self) -> &u8 {
        let HL = ((self.H as usize) << 8) + (self.L as usize);
        &self.internal_ram[HL]
    }
}
