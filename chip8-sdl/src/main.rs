use clap::{self, App, Arg};
use std::error::Error;
use std::fs;

fn decode_commandline_arguments() -> String {
    let commandline_args = std::env::args().collect::<Vec<String>>();

    let matches = App::new("chip8")
        .arg(Arg::with_name("GAME_ROM").required(true).index(1))
        .get_matches_from(commandline_args);

    matches.value_of("GAME_ROM").unwrap().to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    let game_rom_filename = decode_commandline_arguments();
    let game_rom_data = fs::read(game_rom_filename)?;

    let mut chip8 = libchip8::Chip8::new(&game_rom_data);

    chip8.run();

    Ok(())
}
