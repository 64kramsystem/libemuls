# libemuls

`libemuls` is a framework, written in Rust, for writing retro gaming system emulators.

Although it provides binaries for emulating systems, it's not intended for end-users, instead, for developers interested in writing emulators.

Table of contents:

- [libemuls](#libemuls)
  - [Architecture](#architecture)
    - [Software support](#software-support)
  - [Software engineering considerations](#software-engineering-considerations)
    - [Clarity](#clarity)
    - [Next developments](#next-developments)
  - [Current support, and running an emulator](#current-support-and-running-an-emulator)
  - [Current packages](#current-packages)
    - [Packages naming](#packages-naming)
  - [Documentation](#documentation)

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

Due to `system-chip_8` being primarily an exploration, it doesn't have any automated tests, however, all the future libraries will be specified and verified through test suites.

### Next developments

From a technical perspective, the system emulators will be designed as distributed systems; as a requirement, libraries will (need to) be able to run asynchronously, each running in its own thread, communicating with each other, and synchronizing with a central clock.

## Current support, and running an emulator

Currently, a basic CHIP-8 emulator with SDL frontend is provided.

It can be run from the project root, with `cargo run --bin emu-chip_8-sdl -- /path/to/rom`; help is provided via `cargo run --bin emu-chip_8-sdl -- --help`.

In order to quickly run a test game:

```sh
cargo run --bin emu-chip_8-sdl -- <(curl -L 'https://github.com/JohnEarnest/chip8Archive/blob/master/roms/flightrunner.ch8?raw=true')
```

Keys are `W`, `A`, `S`, `D`.

## Current packages

The project is currently composed of the following packages:

- `emu-chip_8-sdl`: SDL CHIP-8 full emulator
- `system-chip_8`: CHIP-8 (single-component) system
- `frontend-sdl`: SDL frontend implementation
- `interfaces-frontend`: Frontend interfaces

### Packages naming

- full emulators: `emu-<system>-<frontend>`, e.g. `emu-chip_8-sdl`
- systems: `system-<name>`, e.g. `system-commodore_64`
- components: `component-<name>`, e.g. `system-chip_8`
- periperals: `peripheral-<name>`, e.g. `peripheral-tv_pal`
- frontends: `frontend-<name>`, e.g. `frontend-sdl`
- interfaes: `interface-<package_type>`, e.g. `interfaces-frontend`

## Documentation

The code comments and documentation are intended to be expressive and thorough; the specifications are defined through the test suites.

The [GitHub project wiki](../../wiki) includes additional information.
