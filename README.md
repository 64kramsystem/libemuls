# libemuls

`libemuls` is my personal experimental project in the retrogaming emulators field.

Since the purpose it to (at least currently) be a dumping ground for my experiments, it's of no value to end users; some ideas may be of general interest to developers.

Subjects I've explored until now:

- CHIP-8 emulation
  - completed the unextended instructions set, a few extensions implemented
  - functioning emulator, with an SDL interface
- Rust programming
- Generic emulation interfaces, with strong components separation
- Benchmarking different multithreading architectures for high-performance systems, including lockless implementations
- Automated generation of CPU instructions fetch/decoding/execution, starting from instructions metadata

My next subject in line is implementing a WASM interface.

The [GitHub project wiki](../../wiki) has additional information.
