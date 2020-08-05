// For clarity, any register reference is upper case.
#![allow(non_snake_case)]

type Byte = u8;
type Word = u16;

// Simplification: the below are words, however, since they're used in indexing, the required
// casting makes usage very ugly, therefore, they're defined as usize.
//
const RAM_SIZE: usize = 4096;
const FONTS_LOCATION: usize = 0; // There's no reference location, but this is common practice
const PROGRAMS_LOCATION: usize = 0x200;

const FONTSET: [Byte; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Chip8 {
    ram: [Byte; RAM_SIZE],
    screen: [Byte; 64 * 32], // Simplification (exactly: bit)
    stack: [Word; 16],

    v: [Byte; 16],
    i: Word,
    pc: Word,
    sp: Word,

    delay_timer: Byte,
    sound_timer: Byte,

    key: [Byte; 16], // Simplification (exactly: bit)
}

impl Chip8 {
    // Simplification: load the data on instantiation, as there is practically no initialization
    // stage (BIOS/firmware).
    //
    pub fn new(game_rom: &[Byte]) -> Chip8 {
        if game_rom.len() > RAM_SIZE - PROGRAMS_LOCATION {
            panic!(
                "Rom too big!: {} bytes ({} allowed)",
                game_rom.len(),
                RAM_SIZE - PROGRAMS_LOCATION
            );
        }

        let mut chip8 = Chip8 {
            ram: [0; RAM_SIZE],
            screen: [0; 64 * 32],
            stack: [0; 16],

            v: [0; 16],
            i: 0,
            pc: PROGRAMS_LOCATION as Word,
            sp: 0,

            delay_timer: 0,
            sound_timer: 0,

            key: [0; 16],
        };

        chip8.ram[FONTS_LOCATION..FONTS_LOCATION + FONTSET.len()].copy_from_slice(&FONTSET);

        chip8.ram[PROGRAMS_LOCATION..PROGRAMS_LOCATION + game_rom.len()].copy_from_slice(game_rom);

        chip8
    }

    fn emulate_cycle(&mut self) {
        // The decode/execute stages are conventionally split. In this system there is not real need
        // for this, so, for simplicity, they're merged. A separate-stages design would likely have
        // a function pointer and the operands as intermediate values.
        //
        let instruction = self.cycle_fetch();

        self.cycle_decode_execute(instruction);

        self.cycle_update_timers();
    }

    fn draw_flag(&self) -> bool {
        println!("WRITEME: draw_flag");
        true
    }

    fn draw_graphics(&self) {
        println!("WRITEME: draw_graphics")
    }

    fn set_keys(&self) {
        println!("WRITEME: set_keys")
    }

    // CYCLE MAIN STAGES ///////////////////////////////////////////////////////////////////////////

    fn cycle_fetch(&self) -> Word {
        let instruction_hi_byte = self.ram[self.pc as usize] as Word;
        let instruction_lo_byte = self.ram[(self.pc + 1) as usize] as Word;
        (instruction_hi_byte << 8) + instruction_lo_byte
    }

    fn cycle_decode_execute(&mut self, instruction: Word) {
        match instruction {
            0xA000..=0xAFFF => {
                let value = instruction & 0b0000_1111_1111_1111;
                self.cycle_execute_set_I(value);
            }
            _ => panic!(
                "WRITEME: Invalid/unsupported instruction: {:04X}",
                instruction
            ),
        }

        self.pc += 2;
    }

    fn cycle_update_timers(&self) {
        println!("WRITEME: cycle_update_timers")
    }

    // OPCODE EXECUTION ////////////////////////////////////////////////////////////////////////////

    // Sets I to the address NNN.
    //
    fn cycle_execute_set_I(&mut self, value: Word) {
        self.i = value;
    }
}

fn setup_graphics() {
    println!("WRITEME: setup_graphics")
}

fn setup_input() {
    println!("WRITEME: setup_input")
}

pub fn emulate(game_rom: &[Byte]) {
    setup_graphics();
    setup_input();

    let mut chip8 = Chip8::new(game_rom);

    loop {
        chip8.emulate_cycle();

        if chip8.draw_flag() {
            chip8.draw_graphics();
        }

        chip8.set_keys();

        println!("WRITEME: sleep");
    }
}
