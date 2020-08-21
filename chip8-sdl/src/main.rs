#[macro_use]
extern crate maplit;

use clap::{self, App, Arg};

use frontend_sdl::FrontendSdl;
use interfaces::{EventCode, Logger, StdoutLogger};

use std::error::Error;
use std::fs;

fn decode_commandline_arguments() -> (String, bool, bool) {
    let commandline_args = std::env::args().collect::<Vec<String>>();

    let matches = App::new("chip8")
        .arg(Arg::with_name("GAME_ROM").required(true).index(1))
        .arg(
            Arg::with_name("DEBUG")
                .short("d")
                .long("debug")
                .help("Enable debug mode (logs to stdout)"),
        )
        .arg(
            Arg::with_name("MAX_SPEED")
                .short("m")
                .long("max-speed")
                .help("Set the maximum emulation speed (1000x)"),
        )
        .get_matches_from(commandline_args);

    let game_rom_filename = matches.value_of("GAME_ROM").unwrap().to_string();
    let debug_mode = matches.is_present("DEBUG");
    let max_speed = matches.is_present("MAX_SPEED");

    (game_rom_filename, debug_mode, max_speed)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (game_rom_filename, debug_mode, max_speed) = decode_commandline_arguments();

    let game_rom_data = fs::read(game_rom_filename)?;

    let custom_keys_mapping = hashmap! {
         EventCode::KeyNum4 => EventCode::KeyC,
         EventCode::KeyQ => EventCode::KeyNum4,
         EventCode::KeyW => EventCode::KeyNum5,
         EventCode::KeyE => EventCode::KeyNum6,
         EventCode::KeyR => EventCode::KeyD,
         EventCode::KeyA => EventCode::KeyNum7,
         EventCode::KeyS => EventCode::KeyNum8,
         EventCode::KeyD => EventCode::KeyNum9,
         EventCode::KeyF => EventCode::KeyE,
         EventCode::KeyZ => EventCode::KeyA,
         EventCode::KeyX => EventCode::KeyNum0,
         EventCode::KeyC => EventCode::KeyB,
         EventCode::KeyV => EventCode::KeyF,
    };

    let mut sdl_frontend = FrontendSdl::new("CHIP-8!", custom_keys_mapping, Some(60));

    let mut logger: Option<Box<dyn Logger>> = if debug_mode {
        Some(Box::new(StdoutLogger::new()))
    } else {
        None
    };

    let mut chip8 = libchip8::Chip8::new(&mut sdl_frontend, &game_rom_data, &mut logger);

    chip8.run(max_speed);

    Ok(())
}
