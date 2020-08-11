// For clarity, any register reference is upper case.
#![allow(non_snake_case)]

use std::thread;
use std::time::{Duration, Instant};

type Byte = u8;
type Word = u16;

// Simplification: the below are words, however, since they're used in indexing, the required
// casting makes usage very ugly, therefore, they're defined as usize.
//
const RAM_SIZE: usize = 4096;
const FONTS_LOCATION: usize = 0; // There's no reference location, but this is common practice
const PROGRAMS_LOCATION: usize = 0x200;

const CLOCK_SPEED: u32 = 500; // Herz

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

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
    screen: [Byte; SCREEN_WIDTH * SCREEN_HEIGHT], // Simplification (exactly: bit)
    stack: [usize; 16], // Simplification (exactly: word); see location constants comment.

    V: [Byte; 16],

    // Simplification of the address registers (exactly: word); see location constants comment.
    //
    I: usize,
    PC: usize,
    SP: usize,

    delay_timer: Byte,
    sound_timer: Byte,

    key: [Byte; 16], // Simplification (exactly: bit)

    draw_flag: bool,
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
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            stack: [0; 16],

            V: [0; 16],
            I: 0,
            PC: PROGRAMS_LOCATION,
            SP: 0,

            delay_timer: 0,
            sound_timer: 0,

            key: [0; 16],

            draw_flag: false,
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

    fn draw_graphics(&mut self) {
        println!("WRITEME: draw_graphics");
        self.draw_flag = false;
    }

    fn set_keys(&self) {
        println!("WRITEME: set_keys")
    }

    // CYCLE MAIN STAGES ///////////////////////////////////////////////////////////////////////////

    fn cycle_fetch(&self) -> Word {
        let instruction_hi_byte = self.ram[self.PC] as Word;
        let instruction_lo_byte = self.ram[self.PC + 1] as Word;
        (instruction_hi_byte << 8) + instruction_lo_byte
    }

    fn cycle_decode_execute(&mut self, instruction: Word) {
        match instruction {
            // Some instructions are in the 0x0NNN range (machine code routine call), and need to be
            // placed before it, therefore, out of order.
            //
            0x00E0 => {
                self.execute_clear_screen();
            }
            0x00EE => {
                self.execute_return_from_subroutine();
            }
            0x0000..=0x0FFF => panic!(
                "Call machine code routine instruction or extension not implemented: {:04X}",
                instruction
            ),
            0x1000..=0x1FFF => {
                let address = (instruction & 0x0FFF) as usize;
                self.execute_goto(address);
            }
            0x2000..=0x2FFF => {
                let address = (instruction & 0x0FFF) as usize;
                self.execute_call_subroutine(address);
            }
            0x3000..=0x3FFF => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let n = (instruction & 0x00FF) as Byte;
                self.execute_skip_next_instruction_if_Vx_equals_n(Vx, n);
            }
            0x4000..=0x4FFF => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let n = (instruction & 0x00FF) as Byte;
                self.execute_skip_next_instruction_if_Vx_not_equals_n(Vx, n);
            }
            0x5000..=0x5FFF if instruction & 0x000F == 0x0000 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_skip_next_instruction_if_Vx_equals_Vy(Vx, Vy);
            }
            0x6000..=0x6FFF => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let n = (instruction & 0x00FF) as Byte;
                self.execute_set_Vx_to_n(Vx, n);
            }
            0x7000..=0x7FFF => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let n = (instruction & 0x00FF) as Byte;
                self.execute_add_n_to_Vx(Vx, n);
            }
            0x8000..=0x8FF0 if instruction & 0x000F == 0x0000 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_set_Vx_to_Vy(Vx, Vy);
            }
            0x8001..=0x8FF1 if instruction & 0x000F == 0x0001 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_set_Vx_to_Vx_or_Vy(Vx, Vy);
            }
            0x8002..=0x8FF2 if instruction & 0x000F == 0x0002 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_set_Vx_to_Vx_and_Vy(Vx, Vy);
            }
            0x8003..=0x8FF3 if instruction & 0x000F == 0x0003 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_set_Vx_to_Vx_xor_Vy(Vx, Vy);
            }
            0x8004..=0x8FF4 if instruction & 0x000F == 0x0004 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_add_Vy_to_Vx(Vx, Vy);
            }
            0x8005..=0x8FF5 if instruction & 0x000F == 0x0005 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_subtract_Vy_from_Vx(Vx, Vy);
            }
            0x8006..=0x8FF6 if instruction & 0x000F == 0x0006 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                // Vy is ignored
                self.execute_shift_right_Vx(Vx);
            }
            0x8007..=0x8FF7 if instruction & 0x000F == 0x0007 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_set_Vx_to_Vy_minus_Vx(Vx, Vy);
            }
            0x800E..=0x8FFE if instruction & 0x000F == 0x000E => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                // Vy is ignored
                self.execute_shift_left_Vx(Vx);
            }
            0x9000..=0x9FFF if instruction & 0x000F == 0x0000 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                self.execute_skip_next_instruction_if_Vx_not_equals_Vy(Vx, Vy);
            }
            0xA000..=0xAFFF => {
                let value = (instruction & 0x0FFF) as usize;
                self.execute_set_I(value);
            }
            0xB000..=0xBFFF => {
                let address = (instruction & 0x0FFF) as usize;
                self.execute_goto_plus_V0(address);
            }
            0xC000..=0xCFFF => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let n = (instruction & 0x00FF) as Byte;
                self.execute_set_Vx_to_masked_random(Vx, n);
            }
            0xD000..=0xDFFF => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                let Vy = ((instruction & 0x00F0) >> 4) as usize;
                let lines = (instruction & 0x00F) as usize;
                self.execute_draw_sprite(Vx, Vy, lines);
            }
            0xE09E..=0xEF9E if instruction & 0x00FF == 0x009E => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_skip_next_instruction_if_Vx_key_pressed(Vx);
            }
            0xE0A1..=0xEFA1 if instruction & 0x00FF == 0x00A1 => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_skip_next_instruction_if_Vx_key_not_pressed(Vx);
            }
            0xF007..=0xFF07 if instruction & 0x00FF == 0x0007 => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_set_Vx_to_delay_timer(Vx);
            }
            0xF015..=0xFF15 if instruction & 0x00FF == 0x0015 => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_set_delay_timer_to_Vx(Vx);
            }
            0xF018..=0xFF18 if instruction & 0x00FF == 0x0018 => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_set_sound_timer_to_Vx(Vx);
            }
            0xF01E..=0xFF1E if instruction & 0x00FF == 0x001E => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_add_Vx_to_I(Vx);
            }
            0xF029..=0xFF29 if instruction & 0x00FF == 0x0029 => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_set_I_to_Vx_sprite_address(Vx);
            }
            0xF033..=0xFF33 if instruction & 0x00FF == 0x0033 => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_store_Vx_bcd_representation(Vx);
            }
            0xF055..=0xFF55 if instruction & 0x00FF == 0x0055 => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_dump_registers_to_memory(Vx);
            }
            0xF065..=0xFF65 if instruction & 0x00FF == 0x0065 => {
                let Vx: usize = ((instruction & 0x0F00) >> 8) as usize;
                self.execute_load_registers_from_memory(Vx);
            }
            _ => panic!(
                "WRITEME: Invalid/unsupported instruction: {:04X}",
                instruction
            ),
        }
    }

    fn cycle_update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("WRITEME: Beep");
            }
            self.sound_timer -= 1;
        }
    }

    // OPCODE EXECUTION ////////////////////////////////////////////////////////////////////////////

    fn execute_clear_screen(&mut self) {
        self.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.PC += 2;

        self.draw_flag = true;
    }

    fn execute_return_from_subroutine(&mut self) {
        self.SP -= 1;
        self.PC = self.stack[self.SP];
    }

    fn execute_goto(&mut self, address: usize) {
        self.PC = address;
    }

    fn execute_call_subroutine(&mut self, address: usize) {
        self.stack[self.SP] = self.PC + 2;
        self.SP += 1;
        self.PC = address;
    }

    fn execute_skip_next_instruction_if_Vx_equals_n(&mut self, Vx: usize, n: Byte) {
        if self.V[Vx] == n {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_skip_next_instruction_if_Vx_not_equals_n(&mut self, Vx: usize, n: Byte) {
        if self.V[Vx] != n {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_skip_next_instruction_if_Vx_equals_Vy(&mut self, Vx: usize, Vy: usize) {
        if self.V[Vx] == self.V[Vy] {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_set_Vx_to_n(&mut self, Vx: usize, n: Byte) {
        self.V[Vx] = n;
        self.PC += 2;
    }

    fn execute_add_n_to_Vx(&mut self, Vx: usize, n: Byte) {
        let (addition_result, _) = self.V[Vx].overflowing_add(n);
        self.V[Vx] = addition_result;
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vy(&mut self, Vx: usize, Vy: usize) {
        self.V[Vx] = self.V[Vy];
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vx_or_Vy(&mut self, Vx: usize, Vy: usize) {
        self.V[Vx] |= self.V[Vy];
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vx_and_Vy(&mut self, Vx: usize, Vy: usize) {
        self.V[Vx] &= self.V[Vy];
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vx_xor_Vy(&mut self, Vx: usize, Vy: usize) {
        self.V[Vx] ^= self.V[Vy];
        self.PC += 2;
    }

    fn execute_add_Vy_to_Vx(&mut self, Vx: usize, Vy: usize) {
        let (addition_result, carry) = self.V[Vx].overflowing_add(self.V[Vy]);
        self.V[Vx] = addition_result;
        self.V[15] = carry as Byte;
        self.PC += 2;
    }

    fn execute_subtract_Vy_from_Vx(&mut self, Vx: usize, Vy: usize) {
        let (subtraction_result, carry) = self.V[Vx].overflowing_sub(self.V[Vy]);
        self.V[Vx] = subtraction_result;
        self.V[15] = (!carry) as Byte;
        self.PC += 2;
    }

    fn execute_shift_right_Vx(&mut self, Vx: usize) {
        self.V[15] = self.V[Vx] & 1;
        self.V[Vx] >>= 1;
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vy_minus_Vx(&mut self, Vx: usize, Vy: usize) {
        let (subtraction_result, carry) = self.V[Vy].overflowing_sub(self.V[Vx]);
        self.V[Vx] = subtraction_result;
        self.V[15] = (!carry) as Byte;
        self.PC += 2;
    }

    fn execute_shift_left_Vx(&mut self, Vx: usize) {
        self.V[15] = self.V[Vx] >> 7;
        self.V[Vx] <<= 1;
        self.PC += 2;
    }

    fn execute_skip_next_instruction_if_Vx_not_equals_Vy(&mut self, Vx: usize, Vy: usize) {
        if self.V[Vx] != self.V[Vy] {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_set_I(&mut self, value: usize) {
        self.I = value;
        self.PC += 2;
    }

    fn execute_goto_plus_V0(&mut self, address: usize) {
        self.PC = address + self.V[0] as usize;
    }

    fn execute_set_Vx_to_masked_random(&mut self, Vx: usize, n: Byte) {
        self.V[Vx] = rand::random::<Byte>() & n;
        self.PC += 2;
    }

    fn execute_draw_sprite(&mut self, Vx: usize, Vy: usize, lines: usize) {
        let x = self.V[Vx] as usize;
        let y = self.V[Vy] as usize;

        let mut current_position = SCREEN_WIDTH * y + x;
        let mut sprite_collided: Byte = 0;

        for source_sprite_row in self.ram[(self.I)..(self.I + lines)].iter() {
            for sprite_pixel_shift in (0..=7).rev() {
                let pixel_value = (source_sprite_row >> sprite_pixel_shift) & 0b000_0001_u8;

                if pixel_value == 1 {
                    if self.screen[current_position] == 1 {
                        sprite_collided = 1;
                    }
                    self.screen[current_position] ^= 1;
                }

                current_position += 1;
            }

            current_position += SCREEN_WIDTH - 8;
        }

        self.V[15] = sprite_collided;
        self.PC += 2;

        self.draw_flag = true;
    }

    fn execute_skip_next_instruction_if_Vx_key_pressed(&mut self, Vx: usize) {
        let keyIndex = self.V[Vx] as usize;

        if self.key[keyIndex] == 1 {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_skip_next_instruction_if_Vx_key_not_pressed(&mut self, Vx: usize) {
        let keyIndex = self.V[Vx] as usize;

        if self.key[keyIndex] == 0 {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_set_Vx_to_delay_timer(&mut self, Vx: usize) {
        self.V[Vx] = self.delay_timer;
        self.PC += 2;
    }

    fn execute_set_delay_timer_to_Vx(&mut self, Vx: usize) {
        self.delay_timer = self.V[Vx];
        self.PC += 2;
    }

    fn execute_set_sound_timer_to_Vx(&mut self, Vx: usize) {
        self.sound_timer = self.V[Vx];
        self.PC += 2;
    }

    fn execute_add_Vx_to_I(&mut self, Vx: usize) {
        self.I += self.V[Vx] as usize;
        self.PC += 2;
    }

    fn execute_set_I_to_Vx_sprite_address(&mut self, Vx: usize) {
        self.I = (self.V[Vx] * 5) as usize;
        self.PC += 2;
    }

    fn execute_store_Vx_bcd_representation(&mut self, Vx: usize) {
        let most_significant_digit = self.V[Vx] / 100;
        let middle_digit = (self.V[Vx] % 100) / 10;
        let least_significant_digit = self.V[Vx] % 10;
        self.ram[self.I] = most_significant_digit;
        self.ram[self.I + 1] = middle_digit;
        self.ram[self.I + 2] = least_significant_digit;
        self.PC += 2;
    }

    fn execute_dump_registers_to_memory(&mut self, Vx: usize) {
        // An amusing, but too verbose, Rust-y approach is
        //
        //   for (address, v) in self.ram.iter_mut().skip(self.I).take(16).zip(self.V.iter()) { /* ... */ }
        //
        // There are other middle grounds, e.g.:
        //
        //   for (i, value) in self.V.iter().enumerate() { /* ... */ }
        //
        for i in 0..=Vx {
            self.ram[self.I + i] = self.V[i];
        }
        self.PC += 2;
    }

    fn execute_load_registers_from_memory(&mut self, Vx: usize) {
        for i in 0..=Vx {
            self.V[i] = self.ram[self.I + i];
        }
        self.PC += 2;
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

    let cycle_time_slice = Duration::new(0, 1_000_000_000 / CLOCK_SPEED);

    let mut last_cycle_time = Instant::now();

    loop {
        chip8.emulate_cycle();

        if chip8.draw_flag {
            chip8.draw_graphics();
        }

        chip8.set_keys();

        // If there are no delays, use a fixed loop time (start time + N * cycle_time_slice).
        // If there is a delay, expand the current loop (time).
        //
        let next_cycle_time = last_cycle_time + cycle_time_slice;
        let current_time = Instant::now();

        if current_time < next_cycle_time {
            thread::sleep(next_cycle_time - current_time);
            last_cycle_time = next_cycle_time;
        } else {
            last_cycle_time = current_time;
        }
    }
}
