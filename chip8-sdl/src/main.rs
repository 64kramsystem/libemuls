use clap::{self, App, Arg};

use frontend_sdl::FrontendSdl;
use interfaces::{Logger, StdoutLogger};

use std::collections::HashMap;
use std::error::Error;
use std::fs;

fn decode_commandline_arguments() -> (String, bool) {
    let commandline_args = std::env::args().collect::<Vec<String>>();

    let matches = App::new("chip8")
        .arg(Arg::with_name("GAME_ROM").required(true).index(1))
        .arg(
            Arg::with_name("DEBUG")
                .short("d")
                .long("debug")
                .help("Enable debug mode (logs to stdout)"),
        )
        .get_matches_from(commandline_args);

    let game_rom_filename = matches.value_of("GAME_ROM").unwrap().to_string();
    let debug_mode = matches.is_present("DEBUG");

    (game_rom_filename, debug_mode)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (game_rom_filename, debug_mode) = decode_commandline_arguments();

    let game_rom_data = fs::read(game_rom_filename)?;

    let custom_keys_mapping = HashMap::new();

    let mut sdl_frontend = FrontendSdl::new("CHIP-8!", custom_keys_mapping);

    let mut logger: Option<Box<dyn Logger>> = if debug_mode {
        Some(Box::new(StdoutLogger::new()))
    } else {
        None
    };

    let mut chip8 = libchip8::Chip8::new(&mut sdl_frontend, &game_rom_data, &mut logger);

    chip8.run();

    Ok(())
}
