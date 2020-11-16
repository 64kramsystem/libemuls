#![allow(unused_macros)]

use crate::cpu::{Cpu, Flag, Reg16, Reg8};
use demonstrate::demonstrate;
use strum::IntoEnumIterator;

fn assert_cpu_execute(
    cpu: &mut Cpu,
    instruction_bytes: &[u8],
    A: Option<u8>,
    F: Option<u8>,
    B: Option<u8>,
    C: Option<u8>,
    D: Option<u8>,
    E: Option<u8>,
    H: Option<u8>,
    L: Option<u8>,
    AF: Option<u16>,
    BC: Option<u16>,
    DE: Option<u16>,
    HL: Option<u16>,
    SP: Option<u16>,
    PC: Option<u16>,
    zf: Option<bool>,
    nf: Option<bool>,
    hf: Option<bool>,
    cf: Option<bool>,
    mem: Option<(u16, &[u8])>,
    cycles_spent: u8,
) {
    let actual_cycles_spent = cpu.execute(&instruction_bytes);

    for (register, value) in Reg8::iter().zip([A, F, B, C, D, E, H, L].iter()) {
        if let Some(value) = value {
            assert_eq!(
                cpu[register], *value,
                "Unexpected `{:?}`: actual=0x{:02X}, expected=0x{:02X}",
                register, cpu[register], *value
            );
        }
    }

    for (register, value) in Reg16::iter().zip([AF, BC, DE, HL, SP, PC].iter()) {
        if let Some(value) = value {
            assert_eq!(
                cpu[register], *value,
                "Unexpected `{:?}`: actual=0x{:02X}, expected=0x{:02X}",
                register, cpu[register], *value
            );
        }
    }

    for (flag, value) in Flag::iter().zip([zf, nf, hf, cf].iter()) {
        if let Some(value) = value {
            assert_eq!(
                cpu.get_flag(flag),
                *value,
                "Unexpected `{:?}`: actual={}, expected={}",
                flag,
                cpu.get_flag(flag) as u8,
                *value as u8
            );
        }
    }

    if let Some((start_address, expected_values)) = mem {
        for i in 0..(expected_values.len()) {
            let address = start_address as usize + i;
            let actual_value = cpu.internal_ram[address];
            let expected_value = expected_values[i];

            assert_eq!(
                actual_value, expected_value,
                "Unexpected mem[0x{:04X}]: actual=0x{:02X}, expected=0x{:02X}",
                address, actual_value, expected_value,
            );
        }
    }

    assert_eq!(actual_cycles_spent, cycles_spent);
}

// The reason for this macro was originally to make the registers/flags visible in the assert_cpu_execute()
// call - the IDE would normally show them, but due to limitations somewhere in the toolset, this
// doesn't happen inside [the used] macros.
//
// While writing the macro, it became evident that making the parameters optional would make the UT
// expectations very neat.
//
// Up to a certain point, the macro implicitly tested the register not specified, however, after 16-bit
// registers were added, and in particular AF, the logic became too tangled, so the macro now tests
// only what specified.
//
macro_rules! assert_cpu_execute {
    (
        $cpu:ident,
        $instruction_bytes:ident,
        $( A => $expected_A:literal , )?
        $( F => $expected_F:literal , )?
        $( B => $expected_B:literal , )?
        $( C => $expected_C:literal , )?
        $( D => $expected_D:literal , )?
        $( E => $expected_E:literal , )?
        $( H => $expected_H:literal , )?
        $( L => $expected_L:literal , )?
        $( AF => $expected_AF:literal , )?
        $( BC => $expected_BC:literal , )?
        $( DE => $expected_DE:literal , )?
        $( HL => $expected_HL:literal , )?
        $( SP => $expected_SP:literal , )?
        $( PC => $expected_PC:literal , )?
        $( zf => $expected_zf:literal , )?
        $( nf => $expected_nf:literal , )?
        $( hf => $expected_hf:literal , )?
        $( cf => $expected_cf:literal , )?
        $( mem[$mem_address:literal] => [$( $mem_value:expr ),+] , )?
        cycles: $cycles:literal
    ) => {
        // Middle ground between a giant assignment, and a platoon of single ones.

        #[allow(unused_mut, unused_assignments)]
        let (mut A, mut F, mut B, mut C, mut D, mut E, mut H, mut L) = (None, None, None, None, None, None, None, None);
        #[allow(unused_mut, unused_assignments)]
        let (mut AF, mut BC, mut DE, mut HL, mut SP, mut PC) = (None, None, None, None, None, None);
        #[allow(unused_mut, unused_assignments)]
        let (mut zf, mut nf, mut hf, mut cf) = (None, None, None, None);
        #[allow(unused_mut, unused_assignments)]
        let mut mem = None::<(u16, &[u8])>;

        $( A = Some($expected_A); )?
        $( F = Some($expected_F); )?
        $( B = Some($expected_B); )?
        $( C = Some($expected_C); )?
        $( D = Some($expected_D); )?
        $( E = Some($expected_E); )?
        $( H = Some($expected_H); )?
        $( L = Some($expected_L); )?
        $( AF = Some($expected_AF); )?
        $( BC = Some($expected_BC); )?
        $( DE = Some($expected_DE); )?
        $( HL = Some($expected_HL); )?
        $( SP = Some($expected_SP); )?
        $( PC = Some($expected_PC); )?
        $( zf = Some($expected_zf); )?
        $( nf = Some($expected_nf); )?
        $( hf = Some($expected_hf); )?
        $( cf = Some($expected_cf); )?
        $(
        let expected_mem_values = &[$( $mem_value ),*][..];
        mem = Some(($mem_address, expected_mem_values));
        )?

        assert_cpu_execute(
            &mut $cpu,
            &$instruction_bytes,
            A, F, B, C, D, E, H, L,
            AF, BC, DE, HL, SP, PC,
            zf, nf, hf, cf,
            mem,
            $cycles,
        )
    };
}

demonstrate! {
    describe "CPU" {
        use super::*;

        before {
          // (Current) issue with declarative testing frameworks; see https://git.io/JUlar.
          //
          #[allow(unused_mut)]
          let mut cpu = Cpu::new();
        }

        // Can't really test random, but it's good practice to just make sure it's not been initialized
        // with zeros.
        // This test will fail for near-impossibly unlucky runs (or lucky, depending on the perspective).
        //
        it "initializes" {
            let internal_ram_sum: u32 = cpu.internal_ram.to_vec().iter().map(|&x| x as u32).sum();

            assert_ne!(internal_ram_sum, 0);

            assert_eq!(cpu[Reg8::A], 0);
            assert_eq!(cpu[Reg16::BC], 0);
            assert_eq!(cpu[Reg16::DE], 0);
            assert_eq!(cpu[Reg16::HL], 0);
            assert_eq!(cpu[Reg16::SP], 0);
            assert_eq!(cpu[Reg16::PC], 0);

            assert_eq!(cpu.get_flag(Flag::z), false);
            assert_eq!(cpu.get_flag(Flag::n), false);
            assert_eq!(cpu.get_flag(Flag::h), false);
            assert_eq!(cpu.get_flag(Flag::c), false);
        }

        context "executes" {
            // __TESTS_REPLACEMENT_START__
            context "LD r, n [0x06: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x06, 0x21];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "LD r, n [0x0E: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x0E, 0x21];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "LD r, n [0x16: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x16, 0x21];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "LD r, n [0x1E: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x1E, 0x21];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "LD r, n [0x26: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x26, 0x21];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "LD r, n [0x2E: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x2E, 0x21];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "LD r, n [0x3E: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x3E, 0x21];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "LD r1, r2 [0x78: A, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x78];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x79: A, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x79];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x7A: A, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x7A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x7B: A, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x7B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x7C: A, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x7C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x7D: A, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x7D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x41: B, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x41];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x42: B, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x42];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x43: B, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x43];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x44: B, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x44];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x45: B, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x45];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x48: C, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x48];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x4A: C, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x4A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x4B: C, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x4B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x4C: C, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x4C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x4D: C, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x4D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x50: D, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x50];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x51: D, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x51];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x53: D, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x53];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x54: D, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x54];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x55: D, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x55];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x58: E, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x58];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x59: E, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x59];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x5A: E, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x5A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x5C: E, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x5C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x5D: E, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x5D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x60: H, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x60];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x61: H, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x61];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x62: H, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x62];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x63: H, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x63];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x65: H, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x65];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x68: L, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x68];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x69: L, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x69];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x6A: L, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x6A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x6B: L, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x6B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x6C: L, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x6C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x47: B, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x47];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x4F: C, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x4F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x57: D, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x57];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x5F: E, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x5F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x67: H, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x67];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x6F: L, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x6F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x7F: A, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x7F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x40: B, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x40];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x49: C, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x49];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x52: D, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x52];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x5B: E, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x5B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x64: H, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x64];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, r2 [0x6D: L, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x6D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "LD r1, (rr2) [0x46: B, HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x46];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x4E: C, HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x4E];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x56: D, HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x56];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x5E: E, HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x5E];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x7E: A, HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x7E];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x0A: A, BC]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x0A];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::BC] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x1A: A, DE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x1A];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::DE] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x66: H, HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x66];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD r1, (rr2) [0x6E: L, HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x6E];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x70: HL, B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x70];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    let expected_value = cpu[Reg8::B];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x71: HL, C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x71];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    let expected_value = cpu[Reg8::C];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x72: HL, D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x72];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    let expected_value = cpu[Reg8::D];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x73: HL, E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x73];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    let expected_value = cpu[Reg8::E];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x74: HL, H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x74];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    let expected_value = cpu[Reg8::H];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x75: HL, L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x75];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    let expected_value = cpu[Reg8::L];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x02: BC, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x02];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg16::BC] = 0x0CAF;

                    let expected_value = cpu[Reg8::A];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x12: DE, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x12];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg16::DE] = 0x0CAF;

                    let expected_value = cpu[Reg8::A];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (rr1), r2 [0x77: HL, A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x77];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    let expected_value = cpu[Reg8::A];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0x0CAF] => [expected_value],
                        cycles: 8
                    );
                }
            }

            context "LD (HL), n [0x36]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x36, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        mem[0x0CAF] => [0x21],
                        cycles: 12
                    );
                }
            }

            context "LD A, (nn) [0xFA]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xFA, 0xAF, 0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x24,
                        cycles: 16
                    );
                }
            }

            context "LD (nn), A [0xEA]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xEA, 0xAF, 0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        mem[0x0CAF] => [0x21],
                        cycles: 16
                    );
                }
            }

            context "LD A, (C) [0xF2]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xF2];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x13;
                    cpu.internal_ram[0xFF13] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LD (C), A [0xE2]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xE2];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::C] = 0x13;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        mem[0xFF13] => [0x21],
                        cycles: 8
                    );
                }
            }

            context "LDD A, (HL) [0x3A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x3A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x0000;
                    cpu.internal_ram[0x0000] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        HL => 0xFFFF,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LDD (HL), A [0x32]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x32];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg16::HL] = 0x0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xFFFF,
                        PC => 0x22,
                        mem[0x0000] => [0x21],
                        cycles: 8
                    );
                }
            }

            context "LDI A, (HL) [0x2A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x2A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xFFFF;
                    cpu.internal_ram[0xFFFF] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        HL => 0x0000,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LDI (HL), A [0x22]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x22];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg16::HL] = 0xFFFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x0000,
                        PC => 0x22,
                        mem[0xFFFF] => [0x21],
                        cycles: 8
                    );
                }
            }

            context "LDH (n), A [0xE0]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xE0, 0x13];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        mem[0xFF13] => [0x21],
                        cycles: 12
                    );
                }
            }

            context "LDH A, (n) [0xF0]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xF0, 0x13];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0xFF13] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x23,
                        cycles: 12
                    );
                }
            }

            context "LD rr, nn [0x01: BC]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x01, 0xFE, 0xCA];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        BC => 0xCAFE,
                        PC => 0x24,
                        cycles: 12
                    );
                }
            }

            context "LD rr, nn [0x11: DE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x11, 0xFE, 0xCA];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        DE => 0xCAFE,
                        PC => 0x24,
                        cycles: 12
                    );
                }
            }

            context "LD rr, nn [0x21: HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x21, 0xFE, 0xCA];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xCAFE,
                        PC => 0x24,
                        cycles: 12
                    );
                }
            }

            context "LD rr, nn [0x31: SP]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x31, 0xFE, 0xCA];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFE,
                        PC => 0x24,
                        cycles: 12
                    );
                }
            }

            context "LD SP, HL [0xF9]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xF9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFE,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "LDHL SP, n [0xF8]" {
                it "without conditional flag modifications: positive immediate" {
                    let instruction_bytes = [0xF8, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0x2100;
                    cpu.set_flag(Flag::z, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x2101,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        cycles: 12
                    );
                }
                it "without conditional flag modifications: negative immediate" {
                    let instruction_bytes = [0xF8, 0xFF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0x2100;
                    cpu.set_flag(Flag::z, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x20FF,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        cycles: 12
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xF8, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xCAF0,
                        PC => 0x23,
                        hf => true,
                        cycles: 12
                    );
                }
                it "with flag H modified: negative immediate" {
                    let instruction_bytes = [0xF8, 0xE1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCA0F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xC9F0,
                        PC => 0x23,
                        hf => true,
                        cycles: 12
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xF8, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xCB0F,
                        PC => 0x23,
                        cf => true,
                        cycles: 12
                    );
                }
                it "with flag C modified: negative immediate" {
                    let instruction_bytes = [0xF8, 0xE0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCA2F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xCA0F,
                        PC => 0x23,
                        cf => true,
                        cycles: 12
                    );
                }
            }

            context "LD (nn), SP [0x08]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x08, 0xFE, 0xCA];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xBEEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        mem[0xCAFE] => [0xEF, 0xBE],
                        cycles: 20
                    );
                }
            }

            context "PUSH rr [0xF5: AF]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xF5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::AF] = 0xBEEF;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x22,
                        mem[0xCAFC] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: wraparound" {
                    let instruction_bytes = [0xF5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::AF] = 0xBEEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xFFFE,
                        PC => 0x22,
                        mem[0xFFFE] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
            }

            context "PUSH rr [0xC5: BC]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xC5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::BC] = 0xBEEF;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x22,
                        mem[0xCAFC] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: wraparound" {
                    let instruction_bytes = [0xC5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::BC] = 0xBEEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xFFFE,
                        PC => 0x22,
                        mem[0xFFFE] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
            }

            context "PUSH rr [0xD5: DE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xD5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::DE] = 0xBEEF;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x22,
                        mem[0xCAFC] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: wraparound" {
                    let instruction_bytes = [0xD5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::DE] = 0xBEEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xFFFE,
                        PC => 0x22,
                        mem[0xFFFE] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
            }

            context "PUSH rr [0xE5: HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xE5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xBEEF;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x22,
                        mem[0xCAFC] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: wraparound" {
                    let instruction_bytes = [0xE5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xBEEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xFFFE,
                        PC => 0x22,
                        mem[0xFFFE] => [0xEF, 0xBE],
                        cycles: 16
                    );
                }
            }

            context "POP rr [0xC1: BC]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xC1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    let address = cpu[Reg16::SP] as usize;
                    cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        BC => 0xBEEF,
                        SP => 0xCB00,
                        PC => 0x22,
                        cycles: 12
                    );
                }
                it "without conditional flag modifications: wraparound" {
                    let instruction_bytes = [0xC1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xFFFE;

                    let address = cpu[Reg16::SP] as usize;
                    cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        BC => 0xBEEF,
                        SP => 0x0000,
                        PC => 0x22,
                        cycles: 12
                    );
                }
            }

            context "POP rr [0xD1: DE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xD1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    let address = cpu[Reg16::SP] as usize;
                    cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        DE => 0xBEEF,
                        SP => 0xCB00,
                        PC => 0x22,
                        cycles: 12
                    );
                }
                it "without conditional flag modifications: wraparound" {
                    let instruction_bytes = [0xD1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xFFFE;

                    let address = cpu[Reg16::SP] as usize;
                    cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        DE => 0xBEEF,
                        SP => 0x0000,
                        PC => 0x22,
                        cycles: 12
                    );
                }
            }

            context "POP rr [0xE1: HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xE1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    let address = cpu[Reg16::SP] as usize;
                    cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xBEEF,
                        SP => 0xCB00,
                        PC => 0x22,
                        cycles: 12
                    );
                }
                it "without conditional flag modifications: wraparound" {
                    let instruction_bytes = [0xE1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xFFFE;

                    let address = cpu[Reg16::SP] as usize;
                    cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xBEEF,
                        SP => 0x0000,
                        PC => 0x22,
                        cycles: 12
                    );
                }
            }

            context "POP AF [0xF1]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xF1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    let address = cpu[Reg16::SP] as usize;
                    cpu.internal_ram[address..address + 2].copy_from_slice(&[0xFF, 0xBE]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        AF => 0xBEF0,
                        SP => 0xCB00,
                        PC => 0x22,
                        cycles: 12
                    );
                }




            }

            context "ADD A, r [0x87: A]" {
                it "without conditional flag modifications: A" {
                    let instruction_bytes = [0x87];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }


                it "with flag H modified" {
                    let instruction_bytes = [0x87];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::A] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x87];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::A] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADD A, r [0x80: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x80];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::B] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x80];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x80];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::B] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x80];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::B] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADD A, r [0x81: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x81];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::C] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x81];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x81];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::C] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x81];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::C] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADD A, r [0x82: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x82];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::D] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x82];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x82];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::D] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x82];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::D] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADD A, r [0x83: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x83];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::E] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x83];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x83];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::E] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x83];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::E] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADD A, r [0x84: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x84];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::H] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x84];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x84];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::H] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x84];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::H] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADD A, r [0x85: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x85];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::L] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x85];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x85];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::L] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x85];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::L] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADD A, (HL) [0x86]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x86];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x22,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x86];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x86];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x0F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x31,
                        PC => 0x22,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x86];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0xF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x10,
                        PC => 0x22,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "ADD A, n [0xC6]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xC6, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x23,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xC6, 0x00];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xC6, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x31,
                        PC => 0x23,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xC6, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x10,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "ADC A, r [0x8F: A]" {
                it "without conditional flag modifications: A" {
                    let instruction_bytes = [0x8F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x8F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg8::A] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x8F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x8F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::A] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x8F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::A] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADC A, r [0x88: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x88];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::B] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x88];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg8::B] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x88];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x88];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::B] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x88];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::B] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADC A, r [0x89: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x89];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::C] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x89];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg8::C] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x89];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x89];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::C] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x89];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::C] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADC A, r [0x8A: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x8A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::D] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x8A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg8::D] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x8A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x8A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::D] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x8A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::D] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADC A, r [0x8B: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x8B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::E] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x8B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg8::E] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x8B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x8B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::E] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x8B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::E] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADC A, r [0x8C: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x8C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::H] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x8C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg8::H] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x8C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x8C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::H] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x8C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::H] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADC A, r [0x8D: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x8D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::L] = 0x30;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x51,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x8D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg8::L] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x8D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x8D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x18;
                    cpu[Reg8::L] = 0x18;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x8D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x90;
                    cpu[Reg8::L] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "ADC A, (HL) [0x8E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x8E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x22,
                        nf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x8E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x22,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x8E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x8E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x0F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x31,
                        PC => 0x22,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x8E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0xF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x10,
                        PC => 0x22,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "ADC A, n [0xCE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCE, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x23,
                        nf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0xCE, 0xFF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xFF,
                        PC => 0x23,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCE, 0x00];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xCE, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x31,
                        PC => 0x23,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCE, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x10,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SUB A, r [0x97: A]" {

                it "with flag Z modified" {
                    let instruction_bytes = [0x97];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }


            }

            context "SUB A, r [0x90: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x90];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::B] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x01,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x90];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x90];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::B] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x90];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::B] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xE0,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SUB A, r [0x91: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x91];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::C] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x01,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x91];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x91];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::C] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x91];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::C] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xE0,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SUB A, r [0x92: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x92];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::D] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x01,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x92];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x92];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::D] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x92];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::D] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xE0,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SUB A, r [0x93: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x93];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::E] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x01,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x93];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x93];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::E] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x93];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::E] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xE0,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SUB A, r [0x94: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x94];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::H] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x01,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x94];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x94];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::H] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x94];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::H] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xE0,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SUB A, r [0x95: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x95];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::L] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x01,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x95];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x95];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::L] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x95];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::L] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xE0,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SUB A, (HL) [0x96]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x96];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x42;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x96];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x96];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x96];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xE0,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SUB A, n [0xD6]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xD6, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x42;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x23,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xD6, 0x00];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x23,
                        zf => true,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xD6, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x12,
                        PC => 0x23,
                        nf => true,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xD6, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x10;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x23,
                        nf => true,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SBC A, r [0x9F: A]" {

                it "with flag Z modified" {
                    let instruction_bytes = [0x9F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }


            }

            context "SBC A, r [0x98: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x98];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::B] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x98];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::B] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x98];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x98];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::B] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x98];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::B] = 0x30;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xF1,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SBC A, r [0x99: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x99];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::C] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x99];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::C] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x99];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x99];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::C] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x99];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::C] = 0x30;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xF1,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SBC A, r [0x9A: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x9A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::D] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x9A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::D] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x9A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x9A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::D] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x9A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::D] = 0x30;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xF1,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SBC A, r [0x9B: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x9B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::E] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x9B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::E] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x9B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x9B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::E] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x9B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::E] = 0x30;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xF1,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SBC A, r [0x9C: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x9C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::H] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x9C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::H] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x9C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x9C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::H] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x9C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::H] = 0x30;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xF1,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SBC A, r [0x9D: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x9D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::L] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x9D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg8::L] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x9D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x9D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::L] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x9D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu[Reg8::L] = 0x30;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0xF1,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "SBC A, (HL) [0x9E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x9E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0x9E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x22,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x9E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x9E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x9E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0xF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x22,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SBC A, n [0xDE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xDE, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x23,
                        nf => true,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry set" {
                    let instruction_bytes = [0xDE, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x30;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0E,
                        PC => 0x23,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xDE, 0x00];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xDE, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x23,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xDE, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x30,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "AND A, r [0xA7: A]" {
                it "without conditional flag modifications: A" {
                    let instruction_bytes = [0xA7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1010_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "AND A, r [0xA0: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::B] = 0b0101_1111;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "AND A, r [0xA1: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::C] = 0b0101_1111;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "AND A, r [0xA2: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA2];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::D] = 0b0101_1111;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA2];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "AND A, r [0xA3: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA3];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::E] = 0b0101_1111;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA3];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "AND A, r [0xA4: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA4];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::H] = 0b0101_1111;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA4];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "AND A, r [0xA5: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::L] = 0b0101_1111;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "AND A, (HL) [0xA6]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA6];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0101_1111;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x22,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA6];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0101_0110;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "AND A, n [0xE6]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xE6, 0x5F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1001,
                        PC => 0x23,
                        nf => false,
                        hf => true,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xE6, 0x56];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "OR A, r [0xB7: A]" {
                it "without conditional flag modifications: A" {
                    let instruction_bytes = [0xB7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1010_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "OR A, r [0xB0: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::B] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "OR A, r [0xB1: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::C] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "OR A, r [0xB2: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB2];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::D] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB2];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "OR A, r [0xB3: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB3];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::E] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB3];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "OR A, r [0xB4: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB4];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::H] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB4];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "OR A, r [0xB5: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::L] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB5];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "OR A, (HL) [0xB6]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB6];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB6];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "OR A, n [0xF6]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xF6, 0x59];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xF6, 0x00];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "XOR A, r [0xAF: A]" {

                it "with flag Z modified" {
                    let instruction_bytes = [0xAF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::A] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "XOR A, r [0xA8: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA8];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::B] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA8];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::B] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "XOR A, r [0xA9: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xA9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::C] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xA9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::C] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "XOR A, r [0xAA: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xAA];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::D] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xAA];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::D] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "XOR A, r [0xAB: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xAB];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::E] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xAB];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::E] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "XOR A, r [0xAC: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xAC];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::H] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xAC];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::H] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "XOR A, r [0xAD: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xAD];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::L] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xAD];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg8::L] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }
            }

            context "XOR A, (HL) [0xAE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xAE];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0101_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xAE];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "XOR A, n [0xEE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xEE, 0x59];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xEE, 0xA9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1010_1001;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "CP A, r [0xBF: A]" {

                it "with flag Z modified" {
                    let instruction_bytes = [0xBF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }


            }

            context "CP A, r [0xB8: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB8];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::B] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x22,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB8];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xB8];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::B] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xB8];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::B] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x70,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "CP A, r [0xB9: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xB9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::C] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x22,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xB9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xB9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::C] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xB9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::C] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x70,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "CP A, r [0xBA: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xBA];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::D] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x22,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xBA];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xBA];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::D] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xBA];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::D] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x70,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "CP A, r [0xBB: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xBB];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::E] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x22,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xBB];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xBB];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::E] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xBB];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::E] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x70,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "CP A, r [0xBC: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xBC];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::H] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x22,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xBC];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xBC];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::H] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xBC];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::H] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x70,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "CP A, r [0xBD: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xBD];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu[Reg8::L] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x22,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xBD];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xBD];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg8::L] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xBD];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg8::L] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x70,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "CP A, (HL) [0xBE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x42;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x21;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x22,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x70;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0x90;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x70,
                        PC => 0x22,
                        nf => true,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "CP A, n [0xFE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xFE, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x42;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x42,
                        PC => 0x23,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xFE, 0x00];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x23,
                        zf => true,
                        nf => true,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xFE, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x23,
                        nf => true,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xFE, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x10;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x10,
                        PC => 0x23,
                        nf => true,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "INC r [0x3C: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x3C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x22,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x3C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x3C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x1F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x20,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "INC r [0x04: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x22,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x00,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x1F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x20,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "INC r [0x0C: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x22,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x00,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x1F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x20,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "INC r [0x14: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x14];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x22,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x14];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x00,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x14];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x1F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x20,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "INC r [0x1C: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x1C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x22,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x1C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x00,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x1C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x1F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x20,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "INC r [0x24: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x24];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x22,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x24];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x00,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x24];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x1F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x20,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "INC r [0x2C: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x2C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x22,
                        PC => 0x22,
                        nf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x2C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x00,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x2C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x1F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x20,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "INC (HL) [0x34]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x34];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x21;
                    cpu[Reg16::HL] = 0x0CAF;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        nf => false,
                        mem[0x0CAF] => [0x22],
                        cycles: 12
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x34];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0xFF;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        zf => true,
                        hf => true,
                        mem[0x0CAF] => [0x0],
                        cycles: 12
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x34];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x1F;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        hf => true,
                        mem[0x0CAF] => [0x20],
                        cycles: 12
                    );
                }
            }

            context "DEC r [0x3D: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x3D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x22;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x3D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x3D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "DEC r [0x05: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x05];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x22;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x05];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x05];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "DEC r [0x0D: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x0D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x22;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x0D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x0D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "DEC r [0x15: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x15];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x22;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x15];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x15];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "DEC r [0x1D: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x1D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x22;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x1D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x1D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "DEC r [0x25: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x25];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x22;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x25];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x25];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "DEC r [0x2D: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x2D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x22;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x21,
                        PC => 0x22,
                        nf => true,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x2D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x01;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x2D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x20;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x1F,
                        PC => 0x22,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "DEC (HL) [0x35]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x35];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x22;
                    cpu[Reg16::HL] = 0x0CAF;
                    cpu.set_flag(Flag::n, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        nf => true,
                        mem[0x0CAF] => [0x21],
                        cycles: 12
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x35];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x01;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        zf => true,
                        mem[0x0CAF] => [0x00],
                        cycles: 12
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x35];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0x0CAF] = 0x20;
                    cpu[Reg16::HL] = 0x0CAF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        hf => true,
                        mem[0x0CAF] => [0x1F],
                        cycles: 12
                    );
                }
            }

            context "ADD HL, rr [0x09: BC]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x09];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x2121;
                    cpu[Reg16::BC] = 0x2121;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x4242,
                        PC => 0x22,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x09];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x1800;
                    cpu[Reg16::BC] = 0x1800;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x3000,
                        PC => 0x22,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x09];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x9000;
                    cpu[Reg16::BC] = 0x9000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x2000,
                        PC => 0x22,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "ADD HL, rr [0x19: DE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x19];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x2121;
                    cpu[Reg16::DE] = 0x2121;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x4242,
                        PC => 0x22,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x19];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x1800;
                    cpu[Reg16::DE] = 0x1800;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x3000,
                        PC => 0x22,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x19];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x9000;
                    cpu[Reg16::DE] = 0x9000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x2000,
                        PC => 0x22,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "ADD HL, rr [0x29: HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x29];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x2121;
                    cpu[Reg16::HL] = 0x2121;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x4242,
                        PC => 0x22,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x29];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x1800;
                    cpu[Reg16::HL] = 0x1800;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x3000,
                        PC => 0x22,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x29];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x9000;
                    cpu[Reg16::HL] = 0x9000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x2000,
                        PC => 0x22,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "ADD HL, rr [0x39: SP]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x39];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x2121;
                    cpu[Reg16::SP] = 0x2121;
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x4242,
                        PC => 0x22,
                        nf => false,
                        cycles: 8
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0x39];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x1800;
                    cpu[Reg16::SP] = 0x1800;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x3000,
                        PC => 0x22,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x39];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x9000;
                    cpu[Reg16::SP] = 0x9000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x2000,
                        PC => 0x22,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "ADD SP, n [0xE8]" {
                it "without conditional flag modifications: positive immediate" {
                    let instruction_bytes = [0xE8, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0x2100;
                    cpu.set_flag(Flag::z, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0x2101,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: negative immediate" {
                    let instruction_bytes = [0xE8, 0xFF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0x2100;
                    cpu.set_flag(Flag::z, true);
                    cpu.set_flag(Flag::n, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0x20FF,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        cycles: 16
                    );
                }

                it "with flag H modified" {
                    let instruction_bytes = [0xE8, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAF0,
                        PC => 0x23,
                        hf => true,
                        cycles: 16
                    );
                }
                it "with flag H modified: negative immediate" {
                    let instruction_bytes = [0xE8, 0xE1];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCA0F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xC9F0,
                        PC => 0x23,
                        hf => true,
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xE8, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCB0F,
                        PC => 0x23,
                        cf => true,
                        cycles: 16
                    );
                }
                it "with flag C modified: negative immediate" {
                    let instruction_bytes = [0xE8, 0xE0];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCA2F;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCA0F,
                        PC => 0x23,
                        cf => true,
                        cycles: 16
                    );
                }
            }

            context "INC rr [0x03: BC]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::BC] = 0xFFFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        BC => 0x0000,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "INC rr [0x13: DE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x13];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::DE] = 0xFFFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        DE => 0x0000,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "INC rr [0x23: HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x23];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xFFFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0x0000,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "INC rr [0x33: SP]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x33];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xFFFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0x0000,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "DEC rr [0x0B: BC]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x0B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::BC] = 0x0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        BC => 0xFFFF,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "DEC rr [0x1B: DE]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x1B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::DE] = 0x0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        DE => 0xFFFF,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "DEC rr [0x2B: HL]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x2B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0x0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        HL => 0xFFFF,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "DEC rr [0x3B: SP]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x3B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0x0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xFFFF,
                        PC => 0x22,
                        cycles: 8
                    );
                }
            }

            context "SWAP r [0xCB 0x37: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x37];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x21;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x12,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x37];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "SWAP r [0xCB 0x30: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x21;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x12,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "SWAP r [0xCB 0x31: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x31];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x21;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x12,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x31];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "SWAP r [0xCB 0x32: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x32];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x21;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x12,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x32];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "SWAP r [0xCB 0x33: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x33];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x21;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x12,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x33];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "SWAP r [0xCB 0x34: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x34];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x21;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x12,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x34];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "SWAP r [0xCB 0x35: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x35];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x21;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x12,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x35];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0x00;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0x00,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "SWAP (HL) [0xCB 0x36]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x36];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0xCAFE] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0x12],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x36];

                    cpu[Reg16::PC] = 0x21;
                    cpu.internal_ram[0xCAFE] = 0x00;
                    cpu[Reg16::HL] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0x00],
                        cycles: 16
                    );
                }
            }

            context "DAA [0x27]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x27];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x1B;
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x21,
                        PC => 0x22,
                        hf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x27];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0x0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x00,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

            }

            context "CPL [0x2F]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x2F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0xF0;
                    cpu.set_flag(Flag::n, false);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0x0F,
                        PC => 0x22,
                        nf => true,
                        hf => true,
                        cycles: 4
                    );
                }
            }

            context "CCF [0x3F]" {

                it "with flag C modified: -> true" {
                    let instruction_bytes = [0x3F];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
                it "with flag C modified: -> false" {
                    let instruction_bytes = [0x3F];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        cf => false,
                        cycles: 4
                    );
                }
            }

            context "SCF [0x37]" {
                it "without conditional flag modifications: C -> true" {
                    let instruction_bytes = [0x37];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => true,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: C -> false" {
                    let instruction_bytes = [0x37];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "NOP [0x00]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x00];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        cycles: 4
                    );
                }
            }

            context "RLCA [0x07]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x07];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x07];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x07];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1110_0001,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "RLA [0x17]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0001,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1110_0000,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "RRCA [0x0F]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1111,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b01000_0111,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "RRA [0x1F]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1111,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1000_1111,
                        PC => 0x22,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x22,
                        zf => true,
                        cycles: 4
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0111,
                        PC => 0x22,
                        cf => true,
                        cycles: 4
                    );
                }
            }

            context "RLC r [0xCB 0x07: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x07];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x07];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x07];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1110_0001,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RLC r [0xCB 0x00: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x00];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x00];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x00];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1110_0001,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RLC r [0xCB 0x01: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x01];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1110_0001,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RLC r [0xCB 0x02: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x02];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x02];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x02];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1110_0001,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RLC r [0xCB 0x03: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1110_0001,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RLC r [0xCB 0x04: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1110_0001,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RLC r [0xCB 0x05: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x05];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x05];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x05];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1110_0001,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RLC (HL) [0xCB 0x06]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x06];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        mem[0xCAFE] => [0b1111_0000],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x06];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0b0000_0000],
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x06];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cf => true,
                        mem[0xCAFE] => [0b1110_0001],
                        cycles: 16
                    );
                }
            }

            context "RL r [0xCB 0x17: A]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x17];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RL r [0xCB 0x10: B]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1111_0001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RL r [0xCB 0x11: C]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x11];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x11];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1111_0001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x11];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x11];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RL r [0xCB 0x12: D]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x12];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x12];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1111_0001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x12];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x12];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RL r [0xCB 0x13: E]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x13];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x13];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1111_0001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x13];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x13];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RL r [0xCB 0x14: H]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x14];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x14];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1111_0001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x14];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x14];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RL r [0xCB 0x15: L]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x15];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x15];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1111_0001,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x15];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x15];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RL (HL) [0xCB 0x16]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x16];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b1111_0000],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x16];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b1111_0001],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x16];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0b0000_0000],
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x16];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cf => true,
                        mem[0xCAFE] => [0b1110_0000],
                        cycles: 16
                    );
                }
            }

            context "RRC r [0xCB 0x0F: A]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x0F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RRC r [0xCB 0x08: B]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x08];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x08];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x08];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x08];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RRC r [0xCB 0x09: C]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x09];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x09];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x09];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x09];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RRC r [0xCB 0x0A: D]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x0A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x0A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x0A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x0A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RRC r [0xCB 0x0B: E]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x0B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x0B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x0B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x0B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RRC r [0xCB 0x0C: H]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x0C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RRC r [0xCB 0x0D: L]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x0D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x0D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x0D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x0D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RRC (HL) [0xCB 0x0E]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x0E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b0000_1111],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x0E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b1000_1111],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x0E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0b0000_0000],
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x0E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cf => true,
                        mem[0xCAFE] => [0b0000_0111],
                        cycles: 16
                    );
                }
            }

            context "RR r [0xCB 0x1F: A]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x1F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RR r [0xCB 0x18: B]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x18];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x18];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x18];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x18];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RR r [0xCB 0x19: C]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x19];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x19];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x19];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x19];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RR r [0xCB 0x1A: D]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x1A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x1A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x1A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x1A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RR r [0xCB 0x1B: E]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x1B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x1B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x1B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x1B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RR r [0xCB 0x1C: H]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x1C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x1C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x1C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x1C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RR r [0xCB 0x1D: L]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x1D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x1D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x1D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x1D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "RR (HL) [0xCB 0x1E]" {
                it "without conditional flag modifications: carry was not set" {
                    let instruction_bytes = [0xCB, 0x1E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b0000_1111],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: carry was set" {
                    let instruction_bytes = [0xCB, 0x1E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0001_1110;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b1000_1111],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x1E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0b0000_0000],
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x1E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cf => true,
                        mem[0xCAFE] => [0b0000_0111],
                        cycles: 16
                    );
                }
            }

            context "SLA r [0xCB 0x27: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x27];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x27];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x27];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SLA r [0xCB 0x20: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x20];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x20];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x20];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SLA r [0xCB 0x21: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x21];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SLA r [0xCB 0x22: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x22];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x22];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x22];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SLA r [0xCB 0x23: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x23];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x23];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x23];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SLA r [0xCB 0x24: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x24];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x24];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x24];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SLA r [0xCB 0x25: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x25];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x25];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x25];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SLA (HL) [0xCB 0x26]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x26];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0111_1000;
                    cpu.set_flag(Flag::c, true);
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b1111_0000],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x26];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0b0000_0000],
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x26];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cf => true,
                        mem[0xCAFE] => [0b1110_0000],
                        cycles: 16
                    );
                }
            }

            context "SRA r [0xCB 0x2F: A]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x2F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x2F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1100_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x2F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x2F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRA r [0xCB 0x28: B]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x28];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x28];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1100_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x28];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x28];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRA r [0xCB 0x29: C]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x29];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x29];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1100_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x29];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x29];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRA r [0xCB 0x2A: D]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x2A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x2A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1100_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x2A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x2A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRA r [0xCB 0x2B: E]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x2B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x2B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1100_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x2B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x2B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRA r [0xCB 0x2C: H]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x2C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x2C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1100_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x2C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x2C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRA r [0xCB 0x2D: L]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x2D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x2D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1100_1111,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x2D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x2D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0111,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRA (HL) [0xCB 0x2E]" {
                it "without conditional flag modifications: MSB=0" {
                    let instruction_bytes = [0xCB, 0x2E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b0000_1111],
                        cycles: 16
                    );
                }
                it "without conditional flag modifications: MSB=1" {
                    let instruction_bytes = [0xCB, 0x2E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1001_1110;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b1100_1111],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x2E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0b0000_0000],
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x2E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_1111;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cf => true,
                        mem[0xCAFE] => [0b0000_0111],
                        cycles: 16
                    );
                }
            }

            context "SRL r [0xCB 0x3F: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x3F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x3F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x3F];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRL r [0xCB 0x38: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x38];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x38];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x38];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRL r [0xCB 0x39: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x39];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x39];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x39];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRL r [0xCB 0x3A: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x3A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x3A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x3A];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRL r [0xCB 0x3B: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x3B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x3B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x3B];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRL r [0xCB 0x3C: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x3C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x3C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x3C];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRL r [0xCB 0x3D: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x3D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1111_0000,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x3D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b0000_0000,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x3D];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1110_0000,
                        PC => 0x23,
                        cf => true,
                        cycles: 8
                    );
                }
            }

            context "SRL (HL) [0xCB 0x3E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x3E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0111_1000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        nf => false,
                        hf => false,
                        cf => false,
                        mem[0xCAFE] => [0b1111_0000],
                        cycles: 16
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x3E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b0000_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        mem[0xCAFE] => [0b0000_0000],
                        cycles: 16
                    );
                }

                it "with flag C modified" {
                    let instruction_bytes = [0xCB, 0x3E];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cf => true,
                        mem[0xCAFE] => [0b1110_0000],
                        cycles: 16
                    );
                }
            }

            context "BIT n, r [0xCB 0x47: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x47, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x47, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "BIT n, r [0xCB 0x40: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x40, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x40, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "BIT n, r [0xCB 0x41: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x41, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x41, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "BIT n, r [0xCB 0x42: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x42, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x42, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "BIT n, r [0xCB 0x43: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x43, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x43, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "BIT n, r [0xCB 0x44: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x44, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x44, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "BIT n, r [0xCB 0x45: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x45, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 8
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x45, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 8
                    );
                }
            }

            context "BIT n, (HL) [0xCB 0x46]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x46, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;
                    cpu.set_flag(Flag::n, true);
                    cpu.set_flag(Flag::h, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => false,
                        nf => false,
                        hf => true,
                        cycles: 12
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0xCB, 0x46, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        zf => true,
                        cycles: 12
                    );
                }
            }

            context "SET n, r [0xCB 0xC7: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC7, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1111_1000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "SET n, r [0xCB 0xC0: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC0, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1111_1000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "SET n, r [0xCB 0xC1: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC1, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1111_1000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "SET n, r [0xCB 0xC2: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC2, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1111_1000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "SET n, r [0xCB 0xC3: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC3, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1111_1000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "SET n, r [0xCB 0xC4: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC4, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1111_1000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "SET n, r [0xCB 0xC5: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC5, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1111_1000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "SET n, (HL) [0xCB 0xC6]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0xC6, 0x03];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        mem[0xCAFE] => [0b1111_1000],
                        cycles: 16
                    );
                }
            }

            context "RES n, r [0xCB 0x87: A]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x87, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::A] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A => 0b1110_0000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "RES n, r [0xCB 0x80: B]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x80, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::B] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B => 0b1110_0000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "RES n, r [0xCB 0x81: C]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x81, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::C] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        C => 0b1110_0000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "RES n, r [0xCB 0x82: D]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x82, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::D] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        D => 0b1110_0000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "RES n, r [0xCB 0x83: E]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x83, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::E] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        E => 0b1110_0000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "RES n, r [0xCB 0x84: H]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x84, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::H] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        H => 0b1110_0000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "RES n, r [0xCB 0x85: L]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x85, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg8::L] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        L => 0b1110_0000,
                        PC => 0x23,
                        cycles: 8
                    );
                }
            }

            context "RES n, (HL) [0xCB 0x86]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCB, 0x86, 0x04];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;
                    cpu.internal_ram[0xCAFE] = 0b1111_0000;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        mem[0xCAFE] => [0b1110_0000],
                        cycles: 16
                    );
                }
            }

            context "JP nn [0xC3]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xC3, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xBEEF,
                        cycles: 16
                    );
                }
            }

            context "JP cc, nn [0xC2: NZ]" {

                it "with jump condition NZ, jump performed: absolute jump" {
                    let instruction_bytes = [0xC2, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xBEEF,
                        cycles: 16
                    );
                }
                it "with jump condition NZ, jump not performed: absolute jump" {
                    let instruction_bytes = [0xC2, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 16
                    );
                }
            }

            context "JP cc, nn [0xCA: Z]" {

                it "with jump condition Z, jump not performed: absolute jump" {
                    let instruction_bytes = [0xCA, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 16
                    );
                }
                it "with jump condition Z, jump performed: absolute jump" {
                    let instruction_bytes = [0xCA, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xBEEF,
                        cycles: 16
                    );
                }
            }

            context "JP cc, nn [0xD2: NC]" {

                it "with jump condition NC, jump performed: absolute jump" {
                    let instruction_bytes = [0xD2, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xBEEF,
                        cycles: 16
                    );
                }
                it "with jump condition NC, jump not performed: absolute jump" {
                    let instruction_bytes = [0xD2, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 16
                    );
                }
            }

            context "JP cc, nn [0xDA: C]" {

                it "with jump condition C, jump not performed: absolute jump" {
                    let instruction_bytes = [0xDA, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 16
                    );
                }
                it "with jump condition C, jump performed: absolute jump" {
                    let instruction_bytes = [0xDA, 0xEF, 0xBE];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xBEEF,
                        cycles: 16
                    );
                }
            }

            context "JP (HL) [0xE9]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xE9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::HL] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xCAFE,
                        cycles: 4
                    );
                }
            }

            context "JR n [0x18]" {
                it "without conditional flag modifications: positive" {
                    let instruction_bytes = [0x18, 0x10];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0031,
                        cycles: 12
                    );
                }
                it "without conditional flag modifications: negative" {
                    let instruction_bytes = [0x18, 0xF0];

                    cpu[Reg16::PC] = 0x21;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0011,
                        cycles: 12
                    );
                }
                it "without conditional flag modifications: overflow (positive)" {
                    let instruction_bytes = [0x18, 0x20];

                    cpu[Reg16::PC] = 0xFFEF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x000F,
                        cycles: 12
                    );
                }
            }

            context "JR cc, n [0x20: NZ]" {

                it "with jump condition NZ, jump performed: positive jump" {
                    let instruction_bytes = [0x20, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0031,
                        cycles: 12
                    );
                }
                it "with jump condition NZ, jump performed: negative jump" {
                    let instruction_bytes = [0x20, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0011,
                        cycles: 12
                    );
                }
                it "with jump condition NZ, jump performed: positive jump, with overflow" {
                    let instruction_bytes = [0x20, 0x1F];

                    cpu.set_flag(Flag::z, false);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x000F,
                        cycles: 12
                    );
                }
                it "with jump condition NZ, jump not performed: positive jump" {
                    let instruction_bytes = [0x20, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition NZ, jump not performed: negative jump" {
                    let instruction_bytes = [0x20, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition NZ, jump not performed: positive jump, with overflow" {
                    let instruction_bytes = [0x20, 0x1F];

                    cpu.set_flag(Flag::z, true);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xFFF2,
                        cycles: 12
                    );
                }
            }

            context "JR cc, n [0x28: Z]" {

                it "with jump condition Z, jump not performed: positive jump" {
                    let instruction_bytes = [0x28, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition Z, jump not performed: negative jump" {
                    let instruction_bytes = [0x28, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition Z, jump not performed: positive jump, with overflow" {
                    let instruction_bytes = [0x28, 0x1F];

                    cpu.set_flag(Flag::z, false);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xFFF2,
                        cycles: 12
                    );
                }
                it "with jump condition Z, jump performed: positive jump" {
                    let instruction_bytes = [0x28, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0031,
                        cycles: 12
                    );
                }
                it "with jump condition Z, jump performed: negative jump" {
                    let instruction_bytes = [0x28, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0011,
                        cycles: 12
                    );
                }
                it "with jump condition Z, jump performed: positive jump, with overflow" {
                    let instruction_bytes = [0x28, 0x1F];

                    cpu.set_flag(Flag::z, true);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x000F,
                        cycles: 12
                    );
                }
            }

            context "JR cc, n [0x30: NC]" {

                it "with jump condition NC, jump performed: positive jump" {
                    let instruction_bytes = [0x30, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0031,
                        cycles: 12
                    );
                }
                it "with jump condition NC, jump performed: negative jump" {
                    let instruction_bytes = [0x30, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0011,
                        cycles: 12
                    );
                }
                it "with jump condition NC, jump performed: positive jump, with overflow" {
                    let instruction_bytes = [0x30, 0x1F];

                    cpu.set_flag(Flag::c, false);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x000F,
                        cycles: 12
                    );
                }
                it "with jump condition NC, jump not performed: positive jump" {
                    let instruction_bytes = [0x30, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition NC, jump not performed: negative jump" {
                    let instruction_bytes = [0x30, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition NC, jump not performed: positive jump, with overflow" {
                    let instruction_bytes = [0x30, 0x1F];

                    cpu.set_flag(Flag::c, true);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xFFF2,
                        cycles: 12
                    );
                }
            }

            context "JR cc, n [0x38: C]" {

                it "with jump condition C, jump not performed: positive jump" {
                    let instruction_bytes = [0x38, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition C, jump not performed: negative jump" {
                    let instruction_bytes = [0x38, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x23,
                        cycles: 12
                    );
                }
                it "with jump condition C, jump not performed: positive jump, with overflow" {
                    let instruction_bytes = [0x38, 0x1F];

                    cpu.set_flag(Flag::c, false);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0xFFF2,
                        cycles: 12
                    );
                }
                it "with jump condition C, jump performed: positive jump" {
                    let instruction_bytes = [0x38, 0x10];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0031,
                        cycles: 12
                    );
                }
                it "with jump condition C, jump performed: negative jump" {
                    let instruction_bytes = [0x38, 0xF0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x0011,
                        cycles: 12
                    );
                }
                it "with jump condition C, jump performed: positive jump, with overflow" {
                    let instruction_bytes = [0x38, 0x1F];

                    cpu.set_flag(Flag::c, true);
                    cpu[Reg16::PC] = 0xFFF0;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x000F,
                        cycles: 12
                    );
                }
            }

            context "CALL nn [0xCD]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCD, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x3021,
                        mem[0xCAFC] => [0x24, 0x00],
                        cycles: 24
                    );
                }
            }

            context "CALL cc, nn [0xC4: NZ]" {

                it "with jump condition NZ, jump performed: no wraparounds" {
                    let instruction_bytes = [0xC4, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x3021,
                        mem[0xCAFC] => [0x24, 0x00],
                        cycles: 24
                    );
                }
                it "with jump condition NZ, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xC4, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 24
                    );
                }
            }

            context "CALL cc, nn [0xCC: Z]" {

                it "with jump condition Z, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xCC, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 24
                    );
                }
                it "with jump condition Z, jump performed: no wraparounds" {
                    let instruction_bytes = [0xCC, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x3021,
                        mem[0xCAFC] => [0x24, 0x00],
                        cycles: 24
                    );
                }
            }

            context "CALL cc, nn [0xD4: NC]" {

                it "with jump condition NC, jump performed: no wraparounds" {
                    let instruction_bytes = [0xD4, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x3021,
                        mem[0xCAFC] => [0x24, 0x00],
                        cycles: 24
                    );
                }
                it "with jump condition NC, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xD4, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 24
                    );
                }
            }

            context "CALL cc, nn [0xDC: C]" {

                it "with jump condition C, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xDC, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x24,
                        cycles: 24
                    );
                }
                it "with jump condition C, jump performed: no wraparounds" {
                    let instruction_bytes = [0xDC, 0x21, 0x30];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x3021,
                        mem[0xCAFC] => [0x24, 0x00],
                        cycles: 24
                    );
                }
            }

            context "RST [0xC7]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xC7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x00,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RST [0xCF]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xCF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x08,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RST [0xD7]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xD7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x10,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RST [0xDF]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xDF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x18,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RST [0xE7]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xE7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x20,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RST [0xEF]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xEF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x28,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RST [0xF7]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xF7];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x30,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RST [0xFF]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xFF];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCAFC,
                        PC => 0x38,
                        mem[0xCAFC] => [0x21, 0x00],
                        cycles: 16
                    );
                }
            }

            context "RET [0xC9]" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0xC9];

                    cpu[Reg16::PC] = 0x21;
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCB00,
                        PC => 0x2130,
                        cycles: 16
                    );
                }
            }

            context "RET cc [0xC0: NZ]" {

                it "with jump condition NZ, jump performed: no wraparounds" {
                    let instruction_bytes = [0xC0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCB00,
                        PC => 0x2130,
                        cycles: 20
                    );
                }
                it "with jump condition NZ, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xC0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        cycles: 20
                    );
                }
            }

            context "RET cc [0xC8: Z]" {

                it "with jump condition Z, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xC8];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, false);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        cycles: 20
                    );
                }
                it "with jump condition Z, jump performed: no wraparounds" {
                    let instruction_bytes = [0xC8];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::z, true);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCB00,
                        PC => 0x2130,
                        cycles: 20
                    );
                }
            }

            context "RET cc [0xD0: NC]" {

                it "with jump condition NC, jump performed: no wraparounds" {
                    let instruction_bytes = [0xD0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCB00,
                        PC => 0x2130,
                        cycles: 20
                    );
                }
                it "with jump condition NC, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xD0];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        cycles: 20
                    );
                }
            }

            context "RET cc [0xD8: C]" {

                it "with jump condition C, jump not performed: no wraparounds" {
                    let instruction_bytes = [0xD8];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, false);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        PC => 0x22,
                        cycles: 20
                    );
                }
                it "with jump condition C, jump performed: no wraparounds" {
                    let instruction_bytes = [0xD8];

                    cpu[Reg16::PC] = 0x21;
                    cpu.set_flag(Flag::c, true);
                    cpu[Reg16::SP] = 0xCAFE;
                    cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        SP => 0xCB00,
                        PC => 0x2130,
                        cycles: 20
                    );
                }
            }
            // __TESTS_REPLACEMENT_END__
          }
    }
}
