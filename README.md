# libemuls

`libemuls` is a framework, written in Rust, for writing retro gaming system emulators.

Although it provides binaries for emulating systems, it's not intended for end-users, instead, for developers interested in writing emulators.

## Architecture

The architecture is founded on separation of concerns, which is expressed in two areas:

- in terms of tiers: by separating the frontend from the backend, it makes it simple to write different frontends (e.g. SDL, WASM...)
- in terms of components: each hardware component is encapsulated in a library, so that an emulator can be written by putting together libraries; for example, a Commodore 64 can be emulated by wiring together separate libraries for the MOS 6510, the VIC-II, and the SID 8580 (of course, it's not implied that "wiring together" is a simple task).

### Software support

Since the project's target is not gaming in itself, compatibility with games has relatively little importance. A base number of programs (games) will be supported for each platform, but the extra time required to debug obscure, undocumented behaviors, will be rather spent for researching/developing a new library.

Compatibility improvement contributions are always welcome, nonetheless.

## Software engineering considerations

### Clarity

Since this is essentially an educative project, it's founded on clarity in every aspect, from the documentation, to the testing, down to the SCM metadata (history).

Due to `libchip8` being primarily an exploration, it doesn't have any automated tests, however, all the future libraries will be specified and verified through test suites.

### Next developments

From a technical perspective, the system emulators will be designed as distributed systems; as a requirement, libraries will (need to) be able to run asynchronously, each running in its own thread, communicating with each other, and synchronizing with a central clock.

## Current support, and running an emulator

Currently, a basic CHIP-8 emulator with SDL frontend is provided.

It can be run from the project root, with `cargo run --bin chip8-sdl -- /path/to/rom`; help is provided via `cargo run --bin chip8-sdl -- --help`.

## Current components

The project is composed of the following packages:

- `interfaces`: Frontend interfaces (traits)
- `frontend-sdl`: SDL frontend implementation
- `chip8-sdl`: CHIP-8 library
- `libchip8`: CHIP-8 emulator, composed from the CHIP-8 library, and the SDL frontend
