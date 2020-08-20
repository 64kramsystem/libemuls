// For clarity, any register reference is upper case.
#![allow(non_snake_case)]

use interfaces::{EventCode, IoFrontend, Logger};
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

const DEFAULT_CLOCK_SPEED: u32 = 500; // Herz
const DEFAULT_TIMERS_SPEED: u32 = 60; // Herz

const MAX_SPEED_FACTOR: u32 = 1000; // Arbitrary; any very high value will do.

const STANDARD_SCREEN_WIDTH: usize = 64;
const STANDARD_SCREEN_HEIGHT: usize = 32;
const HIRES_SCREEN_WIDTH: usize = 128;
const HIRES_SCREEN_HEIGHT: usize = 64;

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

pub struct Chip8<'a, T: IoFrontend> {
    ram: [Byte; RAM_SIZE],
    screen: Vec<bool>,
    stack: [usize; 16], // Simplification (exactly: word); see location constants comment.

    V: [Byte; 16],

    // Simplification of the address registers (exactly: word); see location constants comment.
    //
    I: usize,
    PC: usize,
    SP: usize,

    delay_timer: Byte,
    sound_timer: Byte,

    // True/false for key pressed/released.
    //
    keys_status: [bool; 16],

    io_frontend: &'a mut T,
    logger: &'a mut Box<dyn Logger>,

    screen_width: usize,
    screen_height: usize,
}

impl<'a, T: IoFrontend> Chip8<'a, T> {
    // Simplification: load the data on instantiation, as there is practically no initialization
    // stage (BIOS/firmware).
    //
    pub fn new(
        io_frontend: &'a mut T,
        logger: &'a mut Box<dyn Logger>,
        game_rom: &[Byte],
    ) -> Chip8<'a, T> {
        if game_rom.len() > RAM_SIZE - PROGRAMS_LOCATION {
            panic!(
                "Rom too big!: {} bytes ({} allowed)",
                game_rom.len(),
                RAM_SIZE - PROGRAMS_LOCATION
            );
        }

        let mut chip8 = Chip8 {
            ram: [0; RAM_SIZE],
            screen: vec![],
            stack: [0; 16],

            V: [0; 16],
            I: 0,
            PC: PROGRAMS_LOCATION,
            SP: 0,

            delay_timer: 0,
            sound_timer: 0,

            keys_status: [false; 16],

            io_frontend,
            logger,

            screen_width: STANDARD_SCREEN_WIDTH,
            screen_height: STANDARD_SCREEN_HEIGHT,
        };

        chip8.ram[FONTS_LOCATION..FONTS_LOCATION + FONTSET.len()].copy_from_slice(&FONTSET);

        chip8.ram[PROGRAMS_LOCATION..PROGRAMS_LOCATION + game_rom.len()].copy_from_slice(game_rom);

        chip8.setup_graphics();

        chip8
    }

    pub fn run(&mut self, max_speed: bool) {
        let (mut clock_speed, mut timers_speed) = (DEFAULT_CLOCK_SPEED, DEFAULT_TIMERS_SPEED);

        // Simplest possible approach; it's not worth messing with the timers logic itself, as it's
        // sensitive.
        //
        if max_speed {
            clock_speed *= MAX_SPEED_FACTOR;
            timers_speed *= MAX_SPEED_FACTOR;
        }

        let cycle_time_slice = Duration::new(0, 1_000_000_000 / clock_speed);
        let timers_time_slice = Duration::new(0, 1_000_000_000 / timers_speed);

        // There's no standard, so this is the only reasonable (simple) approach.
        //
        let beep_speed_factor = clock_speed as f32 / DEFAULT_CLOCK_SPEED as f32;

        let mut last_cycle_time = Instant::now();
        let mut next_timers_time = last_cycle_time;

        // This is an optimization; the simplest approach is to pass the reference down the call stack.
        //
        let mut draw_screen = false;

        let mut emulation_running = true;

        while emulation_running {
            self.emulate_cycle(&mut draw_screen, &mut emulation_running);

            if draw_screen {
                self.draw_graphics();
                draw_screen = false;
            }

            self.set_keys(&mut emulation_running);

            // If there are no delays, use a fixed loop time (start time + N * cycle_time_slice).
            // If there is a delay, expand the current loop (time), and delay the timers' next tick.
            //
            // The code would be more expressive if it was possible to set `delay = current_time - next_cycle_time`,
            // but it panics when the result is negative!
            //
            let next_cycle_time = last_cycle_time + cycle_time_slice;

            // This check doesn't need to account delays, because it uses the next cycle time, which,
            // at this step, is not recalculated.
            //
            if last_cycle_time <= next_timers_time && next_timers_time < next_cycle_time {
                self.update_timers(beep_speed_factor);
                next_timers_time += timers_time_slice;
            }

            let current_time = Instant::now();

            if current_time < next_cycle_time {
                thread::sleep(next_cycle_time - current_time);
                last_cycle_time = next_cycle_time;
            } else {
                last_cycle_time = current_time;
                next_timers_time += current_time - next_cycle_time;
            }
        }
    }

    fn setup_graphics(&mut self) {
        self.screen = vec![false; self.screen_width * self.screen_height];
        self.io_frontend
            .init(self.screen_width as u32, self.screen_height as u32);
    }

    fn emulate_cycle(&mut self, draw_screen: &mut bool, emulation_running: &mut bool) {
        // The decode/execute stages are conventionally split. In this system there is not real need
        // for this, so, for simplicity, they're merged. A separate-stages design would likely have
        // a function pointer and the operands as intermediate values.
        //
        let instruction = self.cycle_fetch();

        self.cycle_decode_execute(instruction, draw_screen, emulation_running);
    }

    fn draw_graphics(&mut self) {
        let pixels = self
            .screen
            .iter()
            .map(|pixel_on| {
                if *pixel_on {
                    (255, 255, 255)
                } else {
                    (0, 0, 0)
                }
            })
            .collect::<Vec<(u8, u8, u8)>>();

        self.io_frontend.update_screen(&pixels);
    }

    // Return true if a quit event has been received.
    //
    fn set_keys(&mut self, emulation_running: &mut bool) {
        while let Some((keycode, key_pressed)) = self.io_frontend.read_event(false) {
            let key_index = match keycode {
                EventCode::KeyNum0 => 0,
                EventCode::KeyNum1 => 1,
                EventCode::KeyNum2 => 2,
                EventCode::KeyNum3 => 3,
                EventCode::KeyNum4 => 4,
                EventCode::KeyNum5 => 5,
                EventCode::KeyNum6 => 6,
                EventCode::KeyNum7 => 7,
                EventCode::KeyNum8 => 8,
                EventCode::KeyNum9 => 9,
                EventCode::KeyA => 10,
                EventCode::KeyB => 11,
                EventCode::KeyC => 12,
                EventCode::KeyD => 13,
                EventCode::KeyE => 14,
                EventCode::KeyF => 15,
                EventCode::Quit => {
                    *emulation_running = false;
                    return;
                }
                _ => continue,
            };

            self.keys_status[key_index] = key_pressed;
        }
    }

    fn update_timers(&mut self, beep_speed_factor: f32) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                self.io_frontend.beep(beep_speed_factor);
            }
            self.sound_timer -= 1;
        }
    }

    // CYCLE MAIN STAGES ///////////////////////////////////////////////////////////////////////////

    fn cycle_fetch(&self) -> Word {
        let instruction_hi_byte = self.ram[self.PC] as Word;
        let instruction_lo_byte = self.ram[self.PC + 1] as Word;
        (instruction_hi_byte << 8) + instruction_lo_byte
    }

    fn cycle_decode_execute(
        &mut self,
        instruction: Word,
        draw_screen: &mut bool,
        emulation_running: &mut bool,
    ) {
        // When used alone, nibble1 and/or nibble2 are always Vx/Vy; nibble0 and nibble3
        // are never used alone.
        //
        let nibble0 = (instruction >> 12) as usize;
        let Vx = ((instruction & 0x0F00) >> 8) as usize;
        let Vy = ((instruction & 0x00F0) >> 4) as usize;
        let nibble3 = (instruction & 0x000F) as usize;

        match (nibble0, Vx, Vy, nibble3) {
            // Some instructions are in the 0x0NNN range (machine code routine call), and need to be
            // placed before it, therefore, out of order.
            //
            (0, 0, 0xD, _) => panic!("Unsupported instruction: 00DN (XO-CHIP)"),
            (0, 0, 0xE, 0) => {
                self.execute_clear_screen(draw_screen);
            }
            (0, 0, 0xE, 0xE) => {
                self.execute_return_from_subroutine();
            }
            (0, 0, 0xF, 0xC) => {
                panic!("Unsupported instruction: 00FC (Super-CHIP 1.1)");
            }
            (0, 0, 0xF, 0xF) => {
                self.execute_set_hires_mode();
            }
            (0, _, _, _) => panic!(
                "Call machine code routine instruction or extension not implemented: {:04X}",
                instruction
            ),
            (1, _, _, _) => {
                let address = (instruction & 0x0FFF) as usize;
                self.execute_goto(address);
            }
            (2, _, _, _) => {
                let address = (instruction & 0x0FFF) as usize;
                self.execute_call_subroutine(address);
            }
            (3, _, _, _) => {
                let n = (instruction & 0x00FF) as Byte;
                self.execute_skip_next_instruction_if_Vx_equals_n(Vx, n);
            }
            (4, _, _, _) => {
                let n = (instruction & 0x00FF) as Byte;
                self.execute_skip_next_instruction_if_Vx_not_equals_n(Vx, n);
            }
            (5, _, _, 0) => {
                self.execute_skip_next_instruction_if_Vx_equals_Vy(Vx, Vy);
            }
            (5, _, _, 2) => {
                panic!("Unsupported instruction: 5XY2 (XO-CHIP)");
            }
            (5, _, _, 3) => {
                panic!("Unsupported instruction: 5XY3 (XO-CHIP)");
            }
            (6, _, _, _) => {
                let n = (instruction & 0x00FF) as Byte;
                self.execute_set_Vx_to_n(Vx, n);
            }
            (7, _, _, _) => {
                let n = (instruction & 0x00FF) as Byte;
                self.execute_add_n_to_Vx(Vx, n);
            }
            (8, _, _, 0) => {
                self.execute_set_Vx_to_Vy(Vx, Vy);
            }
            (8, _, _, 1) => {
                self.execute_set_Vx_to_Vx_or_Vy(Vx, Vy);
            }
            (8, _, _, 2) => {
                self.execute_set_Vx_to_Vx_and_Vy(Vx, Vy);
            }
            (8, _, _, 3) => {
                self.execute_set_Vx_to_Vx_xor_Vy(Vx, Vy);
            }
            (8, _, _, 4) => {
                self.execute_add_Vy_to_Vx(Vx, Vy);
            }
            (8, _, _, 5) => {
                self.execute_subtract_Vy_from_Vx(Vx, Vy);
            }
            (8, _, _, 6) => {
                // Vy is ignored
                self.execute_shift_right_Vx(Vx);
            }
            (8, _, _, 7) => {
                self.execute_set_Vx_to_Vy_minus_Vx(Vx, Vy);
            }
            (8, _, _, 0xE) => {
                let Vx = ((instruction & 0x0F00) >> 8) as usize;
                // Vy is ignored
                self.execute_shift_left_Vx(Vx);
            }
            (9, _, _, 0) => {
                self.execute_skip_next_instruction_if_Vx_not_equals_Vy(Vx, Vy);
            }
            (0xA, _, _, _) => {
                let value = (instruction & 0x0FFF) as usize;
                self.execute_set_I(value);
            }
            (0xB, _, _, _) => {
                let address = (instruction & 0x0FFF) as usize;
                self.execute_goto_plus_V0(address);
            }
            (0xC, _, _, _) => {
                let n = (instruction & 0x00FF) as Byte;
                self.execute_set_Vx_to_masked_random(Vx, n);
            }
            (0xD, _, _, _) => {
                let lines = (instruction & 0x00F) as usize;
                self.execute_draw_sprite(Vx, Vy, lines, draw_screen);
            }
            (0xE, _, 9, 0xE) => {
                self.execute_skip_next_instruction_if_Vx_key_pressed(Vx);
            }
            (0xE, _, 0xA, 1) => {
                self.execute_skip_next_instruction_if_Vx_key_not_pressed(Vx);
            }
            (0xF, 0, 0, 0) => {
                panic!("Unsupported instruction: F000 (XO-CHIP)");
            }
            (0xF, _, 0, 1) => {
                panic!("Unsupported instruction: FN01 (XO-CHIP)");
            }
            (0xF, _, 0, 2) => {
                panic!("Unsupported instruction: F002 (XO-CHIP)");
            }
            (0xF, _, 0, 7) => {
                self.execute_set_Vx_to_delay_timer(Vx);
            }
            (0xF, _, 0, 0xA) => {
                self.execute_wait_keypress(Vx, emulation_running);
            }
            (0xF, _, 1, 5) => {
                self.execute_set_delay_timer_to_Vx(Vx);
            }
            (0xF, _, 1, 8) => {
                self.execute_set_sound_timer_to_Vx(Vx);
            }
            (0xF, _, 1, 0xE) => {
                self.execute_add_Vx_to_I(Vx);
            }
            (0xF, _, 2, 0x9) => {
                self.execute_set_I_to_Vx_sprite_address(Vx);
            }
            (0xF, _, 3, 3) => {
                self.execute_store_Vx_bcd_representation(Vx);
            }
            (0xF, _, 5, 5) => {
                self.execute_dump_registers_to_memory(Vx);
            }
            (0xF, _, 6, 5) => {
                self.execute_load_registers_from_memory(Vx);
            }
            _ => panic!("Invalid/unsupported instruction: {:04X}", instruction),
        }
    }

    // OPCODE EXECUTION ////////////////////////////////////////////////////////////////////////////

    fn execute_clear_screen(&mut self, draw_screen: &mut bool) {
        self.logger.log(format!("[{:X}] CLS", self.PC));

        self.screen = vec![false; self.screen_width * self.screen_height];
        self.PC += 2;

        *draw_screen = true;
    }

    fn execute_return_from_subroutine(&mut self) {
        self.logger.log(format!("[{:X}] RET", self.PC));

        self.SP -= 1;
        self.PC = self.stack[self.SP];
    }

    fn execute_set_hires_mode(&mut self) {
        self.logger.log(format!("[{:X}] HIRES", self.PC));

        self.screen_width = HIRES_SCREEN_WIDTH;
        self.screen_height = HIRES_SCREEN_HEIGHT;
        self.setup_graphics();
        self.PC += 2;
    }

    fn execute_goto(&mut self, address: usize) {
        self.logger
            .log(format!("[{:X}] JP 0x{:X}", self.PC, address));

        self.PC = address;
    }

    fn execute_call_subroutine(&mut self, address: usize) {
        self.logger
            .log(format!("[{:X}] CALL 0x{:X}", self.PC, address));

        self.stack[self.SP] = self.PC + 2;
        self.SP += 1;
        self.PC = address;
    }

    fn execute_skip_next_instruction_if_Vx_equals_n(&mut self, Vx: usize, n: Byte) {
        self.logger
            .log(format!("[{:X}] SE V{}, 0x{:X}", self.PC, Vx, n));

        if self.V[Vx] == n {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_skip_next_instruction_if_Vx_not_equals_n(&mut self, Vx: usize, n: Byte) {
        self.logger
            .log(format!("[{:X}] SNE V{}, 0x{:X}", self.PC, Vx, n));

        if self.V[Vx] != n {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_skip_next_instruction_if_Vx_equals_Vy(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] SE V{}, V{}", self.PC, Vx, Vy));

        if self.V[Vx] == self.V[Vy] {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_set_Vx_to_n(&mut self, Vx: usize, n: Byte) {
        self.logger
            .log(format!("[{:X}] LD V{}, 0x{:X}", self.PC, Vx, n));

        self.V[Vx] = n;
        self.PC += 2;
    }

    fn execute_add_n_to_Vx(&mut self, Vx: usize, n: Byte) {
        self.logger
            .log(format!("[{:X}] ADD V{}, 0x{:X}", self.PC, Vx, n));

        let (addition_result, _) = self.V[Vx].overflowing_add(n);
        self.V[Vx] = addition_result;
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vy(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] LD V{}, V{}", self.PC, Vx, Vy));

        self.V[Vx] = self.V[Vy];
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vx_or_Vy(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] OR V{}, V{}", self.PC, Vx, Vy));

        self.V[Vx] |= self.V[Vy];
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vx_and_Vy(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] AND V{}, V{}", self.PC, Vx, Vy));

        self.V[Vx] &= self.V[Vy];
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vx_xor_Vy(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] XOR V{}, V{}", self.PC, Vx, Vy));

        self.V[Vx] ^= self.V[Vy];
        self.PC += 2;
    }

    fn execute_add_Vy_to_Vx(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] ADD V{}, V{}", self.PC, Vx, Vy));

        let (addition_result, carry) = self.V[Vx].overflowing_add(self.V[Vy]);
        self.V[Vx] = addition_result;
        self.V[15] = carry as Byte;
        self.PC += 2;
    }

    fn execute_subtract_Vy_from_Vx(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] SUB V{}, V{}", self.PC, Vx, Vy));

        let (subtraction_result, carry) = self.V[Vx].overflowing_sub(self.V[Vy]);
        self.V[Vx] = subtraction_result;
        self.V[15] = (!carry) as Byte;
        self.PC += 2;
    }

    fn execute_shift_right_Vx(&mut self, Vx: usize) {
        self.logger
            .log(format!("[{:X}] SHR V{} {{, Vy}}", self.PC, Vx));

        self.V[15] = self.V[Vx] & 1;
        self.V[Vx] >>= 1;
        self.PC += 2;
    }

    fn execute_set_Vx_to_Vy_minus_Vx(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] SUBN V{}, V{}", self.PC, Vx, Vy));

        let (subtraction_result, carry) = self.V[Vy].overflowing_sub(self.V[Vx]);
        self.V[Vx] = subtraction_result;
        self.V[15] = (!carry) as Byte;
        self.PC += 2;
    }

    fn execute_shift_left_Vx(&mut self, Vx: usize) {
        self.logger
            .log(format!("[{:X}] SHL V{} {{, Vy}}", self.PC, Vx));

        self.V[15] = self.V[Vx] >> 7;
        self.V[Vx] <<= 1;
        self.PC += 2;
    }

    fn execute_skip_next_instruction_if_Vx_not_equals_Vy(&mut self, Vx: usize, Vy: usize) {
        self.logger
            .log(format!("[{:X}] SNE V{}, V{}", self.PC, Vx, Vy));

        if self.V[Vx] != self.V[Vy] {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_set_I(&mut self, value: usize) {
        self.logger
            .log(format!("[{:X}] LD I, 0x{:X}", self.PC, value));
        self.I = value;
        self.PC += 2;
    }

    fn execute_goto_plus_V0(&mut self, address: usize) {
        self.logger
            .log(format!("[{:X}] JP V0, 0x{:X}", self.PC, address));

        self.PC = address + self.V[0] as usize;
    }

    fn execute_set_Vx_to_masked_random(&mut self, Vx: usize, n: Byte) {
        self.logger
            .log(format!("[{:X}] RND V{}, 0x{:X}", self.PC, Vx, n));

        self.V[Vx] = rand::random::<Byte>() & n;
        self.PC += 2;
    }

    fn execute_draw_sprite(&mut self, Vx: usize, Vy: usize, lines: usize, draw_screen: &mut bool) {
        self.logger.log(format!(
            "[{:X}] DRW V{}, V{}, {}; I={:X}, x={}, y={}",
            self.PC, Vx, Vy, lines, self.I, self.V[Vx], self.V[Vy],
        ));

        let top_x = self.V[Vx] as usize;
        let top_y = self.V[Vy] as usize;

        // lines = 0 is a Superchip feature; see https://chip8.fandom.com/wiki/Instruction_Draw.
        //
        let (bytes_per_line, lines) = if lines == 0 { (2, 16) } else { (1, lines) };

        let mut sprite_collided: Byte = 0;

        for y_shift in 0..lines {
            let pixel_y = (top_y + y_shift) % self.screen_height;

            for sprite_line_index in 0..bytes_per_line {
                for x_shift in 0..8 {
                    // Sprites wrap around; see http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#Dxyn.
                    //
                    let pixel_x = (top_x + x_shift + 8 * sprite_line_index) % self.screen_width;

                    let pixel_source_byte =
                        self.ram[self.I + bytes_per_line * y_shift + sprite_line_index];

                    let pixel_value = (pixel_source_byte << x_shift) & 0b1000_0000;

                    if pixel_value != 0 {
                        let pixel_screen_index = self.screen_width * pixel_y + pixel_x;

                        if self.screen[pixel_screen_index] {
                            sprite_collided = 1;
                        }
                        self.screen[pixel_screen_index] ^= true;
                    }
                }
            }
        }

        self.V[15] = sprite_collided;
        self.PC += 2;

        *draw_screen = true;
    }

    fn execute_skip_next_instruction_if_Vx_key_pressed(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] SKP V{}", self.PC, Vx));

        let keyIndex = self.V[Vx] as usize;

        if self.keys_status[keyIndex] {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_skip_next_instruction_if_Vx_key_not_pressed(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] SKNP V{}", self.PC, Vx));

        let keyIndex = self.V[Vx] as usize;

        if !self.keys_status[keyIndex] {
            self.PC += 4;
        } else {
            self.PC += 2;
        }
    }

    fn execute_set_Vx_to_delay_timer(&mut self, Vx: usize) {
        self.logger.log(format!(
            "[{:X}] LD V{}, DT; DT = {}",
            self.PC, Vx, self.delay_timer
        ));

        self.V[Vx] = self.delay_timer;
        self.PC += 2;
    }

    fn execute_wait_keypress(&mut self, Vx: usize, emulation_running: &mut bool) {
        self.logger.log(format!("[{:X}] LD V{}, K", self.PC, Vx));

        loop {
            if let Some((key_code, key_pressed)) = self.io_frontend.read_event(true) {
                let key_index = match key_code {
                    EventCode::KeyNum0 => 0,
                    EventCode::KeyNum1 => 1,
                    EventCode::KeyNum2 => 2,
                    EventCode::KeyNum3 => 3,
                    EventCode::KeyNum4 => 4,
                    EventCode::KeyNum5 => 5,
                    EventCode::KeyNum6 => 6,
                    EventCode::KeyNum7 => 7,
                    EventCode::KeyNum8 => 8,
                    EventCode::KeyNum9 => 9,
                    EventCode::KeyA => 10,
                    EventCode::KeyB => 11,
                    EventCode::KeyC => 12,
                    EventCode::KeyD => 13,
                    EventCode::KeyE => 14,
                    EventCode::KeyF => 15,
                    EventCode::Quit => {
                        *emulation_running = false;
                        return;
                    }
                    _ => continue,
                };

                // Don't forget to register key released events!
                //
                self.keys_status[key_index] = key_pressed;

                if key_pressed {
                    self.V[Vx] = key_index as Byte;
                    self.PC += 2;
                    return;
                }
            };
        }
    }

    fn execute_set_delay_timer_to_Vx(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] LD DT, V{}", self.PC, Vx));

        self.delay_timer = self.V[Vx];
        self.PC += 2;
    }

    fn execute_set_sound_timer_to_Vx(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] LD ST, V{}", self.PC, Vx));

        self.sound_timer = self.V[Vx];
        self.PC += 2;
    }

    fn execute_add_Vx_to_I(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] ADD I, V{}", self.PC, Vx));

        self.I += self.V[Vx] as usize;
        self.PC += 2;
    }

    fn execute_set_I_to_Vx_sprite_address(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] LD F, V{}", self.PC, Vx));

        self.I = (self.V[Vx] * 5) as usize;
        self.PC += 2;
    }

    fn execute_store_Vx_bcd_representation(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] LD B, V{}", self.PC, Vx));

        let most_significant_digit = self.V[Vx] / 100;
        let middle_digit = (self.V[Vx] % 100) / 10;
        let least_significant_digit = self.V[Vx] % 10;
        self.ram[self.I] = most_significant_digit;
        self.ram[self.I + 1] = middle_digit;
        self.ram[self.I + 2] = least_significant_digit;
        self.PC += 2;
    }

    fn execute_dump_registers_to_memory(&mut self, Vx: usize) {
        self.logger.log(format!("[{:X}] LD I, V{}", self.PC, Vx));

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
        self.logger.log(format!("[{:X}] LD V{}, I", self.PC, Vx));

        for i in 0..=Vx {
            self.V[i] = self.ram[self.I + i];
        }
        self.PC += 2;
    }
}
