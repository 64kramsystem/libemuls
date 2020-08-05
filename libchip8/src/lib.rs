type Byte = u8;
type Word = u16;

struct Chip8 {
    ram: [Byte; 4096],
    screen: [Byte; 64 * 32], // Simplification: each byte is a bit.
    stack: [Word; 16],

    v: [Byte; 16],
    i: Word,
    pc: Word,
    sp: Word,

    delay_timer: Byte,
    sound_timer: Byte,

    key: [Byte; 16], // Simplification: each byte is a bit.
}

impl Chip8 {
    // Simplification: instantiate with the data loaded.
    //
    fn new(game_rom: &Vec<Byte>) -> Chip8 {
        Chip8 {
            ram: [0; 4096],
            screen: [0; 64 * 32],
            stack: [0; 16],

            v: [0; 16],
            i: 0,
            pc: 0,
            sp: 0,

            delay_timer: 0,
            sound_timer: 0,

            key: [0; 16],
        }
    }

    fn emulate_cycle(&self) {
        println!("WRITEME: emulate_cycle");

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

    fn cycle_decode_execute(&self, instruction: Word) {
        println!("WRITEME: cycle_decode");
        match instruction {
            _ => self.cycle_execute_foo(0, 1, 2),
        }
    }

    fn cycle_update_timers(&self) {
        println!("WRITEME: cycle_update_timers")
    }

    // OPCODE EXECUTION ////////////////////////////////////////////////////////////////////////////

    fn cycle_execute_foo(&self, x: u8, y: u8, n: u8) {
        println!("WRITEME: cycle_execute_foo; x:{}, y:{}, n:{}", x, y, n);
    }
}

fn setup_graphics() {
    println!("WRITEME: setup_graphics")
}

fn setup_input() {
    println!("WRITEME: setup_input")
}

pub fn emulate(game_rom: Vec<Byte>) {
    setup_graphics();
    setup_input();

    let chip8 = Chip8::new(&game_rom);

    loop {
        chip8.emulate_cycle();

        if chip8.draw_flag() {
            chip8.draw_graphics();
        }

        chip8.set_keys();

        println!("WRITEME: sleep");
    }
}
