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
    fn new() -> Chip8 {
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

    fn load_game(&self, game_rom: &[Byte]) {
        println!("WRITEME: ")
    }

    fn emulate_cycle(&self) {
        println!("WRITEME: emulate_cycle")
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

    let chip8 = Chip8::new();
    chip8.load_game(&game_rom);

    loop {
        chip8.emulate_cycle();

        if chip8.draw_flag() {
            chip8.draw_graphics();
        }

        chip8.set_keys();

        println!("WRITEME: sleep");
    }
}
