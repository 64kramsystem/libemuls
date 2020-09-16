- [Reference](#reference)
- [Translation structure/sample](#translation-structuresample)
  - [Source document](#source-document)
  - [cpu.rs](#cpurs)
  - [cpu_test.rs](#cpu_testrs)

## Reference

Source: https://gbdev.io/gb-opcodes/Opcodes.json

## Translation structure/sample

### Source document

The document doesn't provide any explicit field to distinguish the instruction family, for example, `INC n` and `INC nn`.
The `immediate` field is wrong, both in the semantic and the data (see https://git.io/JU8Jx and https://git.io/JU8JY).

```json
    "0x0C": {
      "mnemonic": "INC",
      "bytes": 1,
      "cycles": [
        4
      ],
      "operands": [
        {
          "name": "C",
          "immediate": true
        }
      ],
      "immediate": true,
      "flags": {
        "Z": "Z",
        "N": "0",
        "H": "H",
        "C": "-"
      }
    },

    "0x06": {
      "mnemonic": "LD",
      "bytes": 2,
      "cycles": [
        8
      ],
      "operands": [
        {
          "name": "B",
          "immediate": true
        },
        {
          "name": "d8",
          "bytes": 1,
          "immediate": true
        }
      ],
      "immediate": true,
      "flags": {
        "Z": "-",
        "N": "-",
        "H": "-",
        "C": "-"
      }
    },
```

### cpu.rs

`cpu_decoding_code`:

```rust
            [0x06, value @ _] => {
                Self::execute_LD_nn_n(&mut self.PC, &mut self.B, *value);
                8
            }
            [0x46] => {
                let address = Self::compose_address(self.H, self.L);
                Self::execute_LD_r1_Ir2(&mut self.PC, &mut self.B, self.internal_ram[address]);
                8
            }
            [0x34] => {
                let address = Self::compose_address(self.H, self.L);
                Self::execute_INC_IHL(&mut self.PC, &mut self.internal_ram[address], &mut self.zf, &mut self.nf, &mut self.hf);
                12
            }
```

`cpu_execution_code`:

```rust
    fn execute_LD_nn_n(PC: &mut u16, operand: &mut u8, value: u8) {
        *PC += 2;

        *operand = value;
    }

    fn execute_LD_r1_Ir2(PC: &mut u16, operand: &mut u8, value: u8) {
        *PC += 1;

        *operand = value;
    }
    fn execute_INC_IHL(PC: &mut u16, operand: &mut u8, zf: &mut bool, nf: &mut bool, hf: &mut bool) {
        *PC += 1;

        let (new_value, carry) = operand.overflowing_add(1);
        *operand = new_value;

        if *operand & 0b0000_1111 == 0b000_0000 {
            *hf = true;
        }
        if carry {
            *zf = true;
        }
        *nf = false;
    }
```

### cpu_test.rs

```rust
            context "LD B, d8 (0x06)" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x06, 0x21];

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        B: 0x00 => 0x21,
                        PC: 0x21 => 0x23,
                        cycles: 8
                    );
                }
            }

            context "INC A (0x3C)" {
                it "without conditional flag modifications" {
                    let instruction_bytes = [0x3C];

                    cpu.A = 0x21;
                    cpu.nf = true;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A: 0x21 => 0x22,
                        PC: 0x21 => 0x22,
                        nf: 1 => 0,
                        cycles: 4
                    );
                }

                it "with flag Z modified" {
                    let instruction_bytes = [0x3C];

                    cpu.A = 0xFF;

                    assert_cpu_execute!(
                        cpu,
                        instruction_bytes,
                        A: 0xFF => 0x00,
                        PC: 0x21 => 0x22,
                        zf: 0 => 1,
                        cycles: 4
                    );
                }
                // AND SO ON
```
