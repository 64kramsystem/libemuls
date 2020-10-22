module InstructionsCode
  BASE = "base"

  # Code generated is not efficient, in a few ways. This can be optimized, but it's not the scope of
  # this tool.

  INSTRUCTIONS_CODE = {
    "LD r, n" => {
      operation_code: <<~RUST,
        self[dst_register] = *immediate;
      RUST
      testing: ->(register, _) {
        {
          BASE => {
            extra_instruction_bytes: [0x21],
            expectations: "#{register} => 0x21,"
          }
        }
      }
    },
    "LD r1, r2" => {
      operation_code: <<~RUST,
        self[dst_register] = self[src_register];
      RUST
      testing: ->(register1, register2) {
        {
          BASE => {
            presets: "cpu[Reg8::#{register2}] = 0x21;",
            expectations: "#{register1} => 0x21,",
          }
        }
      }
    },
    "LD r1, (rr2)" => {
        operation_code: <<~RUST,
          self[dst_register] = self.internal_ram[self[src_register] as usize];
        RUST
      testing: ->(register1, register2) {
        {
          BASE => {
            presets: <<~RUST,
              cpu.internal_ram[0x0CAF] = 0x21;
              cpu[Reg16::#{register2}] = 0x0CAF;
            RUST
            expectations: "#{register1} => 0x21,",
          }
        }
      }
    },
    "LD (rr1), r2" => {
      operation_code: <<~RUST,
        self.internal_ram[self[dst_register] as usize] = self[src_register];
      RUST
      testing: ->(register1, register2) {
        {
          BASE => {
            # In the cases where r2 is part of r1, an r1 assignment overwrites r2, so that the memory
            # expectation can be kept the same.
            #
            presets: <<~RUST,
              cpu[Reg8::#{register2}] = 0x21;
              cpu[Reg16::#{register1}] = 0x0CAF;

              let expected_value = cpu[Reg8::#{register2}];
            RUST
            expectations: "mem[0x0CAF] => [expected_value],",
          }
        }
      }
    },
    "LD (HL), n" => {
      operation_code: <<~RUST,
        self.internal_ram[self[Reg16::HL] as usize] = *immediate;
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0x21],
            presets: <<~RUST,
              cpu[Reg16::HL] = 0x0CAF;
            RUST
            expectations: "mem[0x0CAF] => [0x21],",
          }
        }
      }
    },
    "LD A, (nn)" => {
      operation_code: <<~RUST,
        self[Reg8::A] = self.internal_ram[*immediate as usize];
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0xAF, 0x0C],
            presets: "cpu.internal_ram[0x0CAF] = 0x21;",
            expectations: "A => 0x21,",
          }
        }
      }
    },
    "LD (nn), A" => {
      operation_code: <<~RUST,
        self.internal_ram[*immediate as usize] = self[Reg8::A];
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0xAF, 0x0C],
            presets: "cpu[Reg8::A] = 0x21;",
            expectations: "mem[0x0CAF] => [0x21],",
          }
        }
      }
    },
    "LD A, (C)" => {
      operation_code: <<~RUST,
        let address = 0xFF00 + self[Reg8::C] as usize;
        self[Reg8::A] = self.internal_ram[address];
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::C] = 0x13;
              cpu.internal_ram[0xFF13] = 0x21;
            RUST
            expectations: "A => 0x21,",
          }
        }
      }
    },
    "LD (C), A" => {
      operation_code: <<~RUST,
        let address = 0xFF00 + self[Reg8::C] as usize;
        self.internal_ram[address] = self[Reg8::A];
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg8::C] = 0x13;
            RUST
            expectations: "mem[0xFF13] => [0x21],",
          }
        }
      }
    },
    "LDD A, (HL)" => {
      operation_code: <<~RUST,
        self[Reg8::A] = self.internal_ram[self[Reg16::HL] as usize];

        let (new_value, _) = self[Reg16::HL].overflowing_sub(1);
        self[Reg16::HL] = new_value;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0x0000;
              cpu.internal_ram[0x0000] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x21,
              HL => 0xFFFF,
            RUST
          }
        }
      }
    },
    "LDD (HL), A" => {
      operation_code: <<~RUST,
        self.internal_ram[self[Reg16::HL] as usize] = self[Reg8::A];

        let (new_value, _) = self[Reg16::HL].overflowing_sub(1);
        self[Reg16::HL] = new_value;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg16::HL] = 0x0000;
            RUST
            expectations: <<~RUST
              HL => 0xFFFF,
              mem[0x0000] => [0x21],
            RUST
          }
        }
      }
    },
    "LDI A, (HL)" => {
      operation_code: <<~RUST,
        self[Reg8::A] = self.internal_ram[self[Reg16::HL] as usize];

        let (new_value, _) = self[Reg16::HL].overflowing_add(1);
        self[Reg16::HL] = new_value;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xFFFF;
              cpu.internal_ram[0xFFFF] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x21,
              HL => 0x0000,
            RUST
          }
        }
      }
    },
    "LDI (HL), A" => {
      operation_code: <<~RUST,
        self.internal_ram[self[Reg16::HL] as usize] = self[Reg8::A];

        let (new_value, _) = self[Reg16::HL].overflowing_add(1);
        self[Reg16::HL] = new_value;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg16::HL] = 0xFFFF;
            RUST
            expectations: <<~RUST
              HL => 0x0000,
              mem[0xFFFF] => [0x21],
            RUST
          }
        }
      }
    },
    "LDH (n), A" => {
      operation_code: <<~RUST,
        let address = 0xFF00 + *immediate as usize;
        self.internal_ram[address] = self[Reg8::A];
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0x13],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
            RUST
            expectations: "mem[0xFF13] => [0x21],",
          }
        }
      }
    },
    "LDH A, (n)" => {
      operation_code: <<~RUST,
        let address = 0xFF00 + *immediate as usize;
        self[Reg8::A] = self.internal_ram[address];
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0x13],
            presets: <<~RUST,
              cpu.internal_ram[0xFF13] = 0x21;
            RUST
            expectations: "A => 0x21,",
          }
        }
      }
    },
    "LD rr, nn" => {
      operation_code: <<~RUST,
        self[dst_register] = *immediate;
      RUST
      testing: ->(register, _) {
        {
          BASE => {
            extra_instruction_bytes: [0xFE, 0xCA],
            expectations: <<~RUST
              #{register} => 0xCAFE,
            RUST
          }
        }
      }
    },
    "LD SP, HL" => {
      operation_code: <<~RUST,
        self[Reg16::SP] = self[Reg16::HL];
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
            RUST
            expectations: <<~RUST
              SP => 0xCAFE,
            RUST
          }
        }
      }
    },
    "LDHL SP, n" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg16::SP];
        // Ugly, but required, conversions.
        let operand2 = *immediate as i8 as i16 as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg16::HL] = result;
      RUST
      testing: ->(_) {
        {
          "#{BASE}: positive immediate" => {
            extra_instruction_bytes: [0x01],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0x2100;
            RUST
            expectations: <<~RUST
              HL => 0x2101,
            RUST
          },
          "#{BASE}: negative immediate" => {
            extra_instruction_bytes: [0xFF],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0x2100;
            RUST
            expectations: <<~RUST
              HL => 0x20FF,
            RUST
          },
          "H" => {
            extra_instruction_bytes: [0x01],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAEF;
            RUST
            expectations: <<~RUST
              HL => 0xCAF0,
              hf => true,
            RUST
          },
          "H: negative immediate" => {
            extra_instruction_bytes: [0xE1],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCA0F;
            RUST
            expectations: <<~RUST
              HL => 0xC9F0,
              hf => true,
            RUST
          },
          "C" => {
            extra_instruction_bytes: [0x10],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAFF;
            RUST
            expectations: <<~RUST
              HL => 0xCB0F,
              cf => true,
            RUST
          },
          "C: negative immediate" => {
            extra_instruction_bytes: [0xE0],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCA2F;
            RUST
            expectations: <<~RUST
              HL => 0xCA0F,
              cf => true,
            RUST
          },
        }
      }
    },
    "LD (nn), SP" => {
      operation_code: <<~RUST,
        self.internal_ram[*immediate as usize] = self[Reg16::SP] as u8;
        self.internal_ram[*immediate as usize + 1] = (self[Reg16::SP] >> 8) as u8;
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0xFE, 0xCA],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xBEEF;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0xEF, 0xBE],
            RUST
          }
        }
      }
    },
    "PUSH rr" => {
      operation_code: <<~RUST,
        let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
        self[Reg16::SP] = new_sp;

        let pushed_bytes = self[dst_register].to_le_bytes();
        self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::#{register}] = 0xBEEF;
              cpu[Reg16::SP] = 0xCAFE;
            RUST
            expectations: <<~RUST
              SP => 0xCAFC,
              mem[0xCAFC] => [0xEF, 0xBE],
            RUST
          },
          "#{BASE}: wraparound" => {
            presets: <<~RUST,
              cpu[Reg16::#{register}] = 0xBEEF;
            RUST
            expectations: <<~RUST
              SP => 0xFFFE,
              mem[0xFFFE] => [0xEF, 0xBE],
            RUST
          },
        }
      }
    },
    "POP rr" => {
      operation_code: <<~RUST,
        let source_bytes = self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap();
        self[dst_register] = u16::from_le_bytes(source_bytes);

        let (result, _) = self[Reg16::SP].overflowing_add(2);
        self[Reg16::SP] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAFE;

              let address = cpu[Reg16::SP] as usize;
              cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);
            RUST
            expectations: <<~RUST
              #{register} => 0xBEEF,
              SP => 0xCB00,
            RUST
          },
          "#{BASE}: wraparound" => {
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xFFFE;

              let address = cpu[Reg16::SP] as usize;
              cpu.internal_ram[address..address + 2].copy_from_slice(&[0xEF, 0xBE]);
            RUST
            expectations: <<~RUST
              #{register} => 0xBEEF,
              SP => 0x0000,
            RUST
          },
        }
      }
    },
    "POP AF" => {
      # A bit ugly - we need to make the generator think that we're actually setting the flags. Since
      # this is the only exception, is not worth adding extra functionality to handle this case.
      #
      operation_code: <<~RUST,
        let source_bytes = self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap();
        self[Reg16::AF] = u16::from_le_bytes(source_bytes) & 0b1111_1111_1111_0000;

        let (result, _) = self[Reg16::SP].overflowing_add(2);
        self[Reg16::SP] = result;

        // self.set_flag(Flag::h, phony);
        // self.set_flag(Flag::z, phony);
        // self.set_flag(Flag::c, phony);
        // self.set_flag(Flag::n, phony);
      RUST
      testing: ->() {
        {
          # This tests all the flags, so per-flag tests are not meaningful.
          #
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAFE;

              let address = cpu[Reg16::SP] as usize;
              cpu.internal_ram[address..address + 2].copy_from_slice(&[0xFF, 0xBE]);
            RUST
            expectations: <<~RUST
              AF => 0xBEF0,
              SP => 0xCB00,
            RUST
          },
          "H" => {skip: true},
          "Z" => {skip: true},
          "C" => {skip: true},
          "N" => {skip: true},
        }
      }
    },
    "ADD A, r" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
      RUST
      testing: ->(register) {
        # Since the base logic is tested in the base test(s), the flag tests are simple.
        #
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg8::#{register}] = 0x30;
            RUST
            expectations: <<~RUST
              A => 0x51,
            RUST
          },
          "#{BASE}: A" => {
            skip: register != "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          'Z' => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0x00;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x18;
              cpu[Reg8::#{register}] = 0x18;
            RUST
            expectations: <<~RUST
              A => 0x30,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x90;
              cpu[Reg8::#{register}] = 0x90;
            RUST
            expectations: <<~RUST
              A => 0x20,
              cf => true,
            RUST
          }
        }
      }
    },
    "ADD A, (HL)" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = self.internal_ram[self[Reg16::HL] as usize];

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x00;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x22;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x0F;
            RUST
            expectations: <<~RUST
              A => 0x31,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0xF0;
            RUST
            expectations: <<~RUST
              A => 0x10,
              cf => true,
            RUST
          }
        }
      }
    },
    "ADD A, n" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = *immediate;

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
      RUST
      testing: ->(register) {
        {
          BASE => {
            extra_instruction_bytes: [0x21],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0x0],
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            extra_instruction_bytes: [0x0F],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x22;
            RUST
            expectations: <<~RUST
              A => 0x31,
              hf => true,
            RUST
          },
          'C' => {
            extra_instruction_bytes: [0xF0],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
            RUST
            expectations: <<~RUST
              A => 0x10,
              cf => true,
            RUST
          }
        }
      }
    },
    # We can't rely on the standard ways of computing the carry (overflowing_add()/API), because the
    # carry addition may have already set it.
    #
    "ADC A, r" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A] as u16;
        let operand2 = self[dst_register] as u16 + self.get_flag(Flag::c) as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result as u8;

        let carry_set = (result & 0b1_0000_0000) != 0;
        self.set_flag(Flag::c, carry_set);
      RUST
      # Since the base logic is tested in the base test(s), the flag tests are simple.
      #
      testing: ->(register) {
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg8::#{register}] = 0x30;
            RUST
            expectations: <<~RUST
              A => 0x51,
            RUST
          },
          "#{BASE}: A" => {
            skip: register != "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          # Using 0xFF also makes sure that the carry added is not accidentally discarded.
          #
          "#{BASE}: carry set" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0xFF;
              cpu[Reg8::#{register}] = 0xFF;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0xFF,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x18;
              cpu[Reg8::#{register}] = 0x18;
            RUST
            expectations: <<~RUST
              A => 0x30,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x90;
              cpu[Reg8::#{register}] = 0x90;
            RUST
            expectations: <<~RUST
              A => 0x20,
              cf => true,
            RUST
          }
        }
      }
    },
    # See `ADC A, r` for reference notes.
    #
    "ADC A, (HL)" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A] as u16;
        let operand2 = self.internal_ram[self[Reg16::HL] as usize] as u16 + self.get_flag(Flag::c) as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result as u8;

        let carry_set = (result & 0b1_0000_0000) != 0;
        self.set_flag(Flag::c, carry_set);
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          "#{BASE}: carry set" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0xFF;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0xFF;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0xFF,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x00;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x22;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x0F;
            RUST
            expectations: <<~RUST
              A => 0x31,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0xF0;
            RUST
            expectations: <<~RUST
              A => 0x10,
              cf => true,
            RUST
          }
        }
      }
    },
    # See `ADC A, r` for reference notes.
    #
    "ADC A, n" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A] as u16;
        let operand2 = *immediate as u16 + self.get_flag(Flag::c) as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg8::A] = result as u8;

        let carry_set = (result & 0b1_0000_0000) != 0;
        self.set_flag(Flag::c, carry_set);
      RUST
      testing: ->(register) {
        {
          BASE => {
            extra_instruction_bytes: [0x21],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          "#{BASE}: carry set" => {
            extra_instruction_bytes: [0xFF],
            presets: <<~RUST,
              cpu[Reg8::A] = 0xFF;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0xFF,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0x0],
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            extra_instruction_bytes: [0x0F],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x22;
            RUST
            expectations: <<~RUST
              A => 0x31,
              hf => true,
            RUST
          },
          'C' => {
            extra_instruction_bytes: [0xF0],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
            RUST
            expectations: <<~RUST
              A => 0x10,
              cf => true,
            RUST
          }
        }
      }
    },
    "SUB A, r" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);
      RUST
      testing: ->(register) {
        # In the `SUB A, A` case, in essence, the only test case is the `Z` one.
        #
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x22;
              cpu[Reg8::#{register}] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x01,
              RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0x0;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
              nf => true,
              RUST
          },
          'H' => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg8::#{register}] = 0x01;
            RUST
            expectations: <<~RUST
              A => 0x1F,
              nf => true,
              hf => true,
            RUST
          },
          'C' => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x70;
              cpu[Reg8::#{register}] = 0x90;
            RUST
            expectations: <<~RUST
              A => 0xE0,
              nf => true,
              cf => true,
            RUST
          }
        }
      }
    },
    "SUB A, (HL)" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = self.internal_ram[self[Reg16::HL] as usize];

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x42;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x21,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x00;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
              nf => true,
              RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x01;
            RUST
            expectations: <<~RUST
              A => 0x1F,
              nf => true,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x70;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x90;
            RUST
            expectations: <<~RUST
              A => 0xE0,
              nf => true,
              cf => true,
            RUST
          }
        }
      }
    },
    "SUB A, n" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = *immediate;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result;

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);
      RUST
      testing: ->(register) {
        {
          BASE => {
            extra_instruction_bytes: [0x21],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x42;
            RUST
            expectations: <<~RUST
              A => 0x21,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0x0],
            expectations: <<~RUST
              A => 0x00,
              zf => true,
              nf => true,
            RUST
          },
          'H' => {
            extra_instruction_bytes: [0x0F],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x12,
              nf => true,
              hf => true,
            RUST
          },
          'C' => {
            extra_instruction_bytes: [0xF0],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x10;
            RUST
            expectations: <<~RUST
              A => 0x20,
              nf => true,
              cf => true,
            RUST
          }
        }
      }
    },
    "SBC A, r" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A] as u16;
        let operand2 = self[dst_register] as u16 + self.get_flag(Flag::c) as u16;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result as u8;

        self.set_flag(Flag::c, carry);
      RUST
      testing: ->(register) {
        # In the `SBC A, A` case, in essence, the only cases are the carry and `Z` ones.
        #
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x30;
              cpu[Reg8::#{register}] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x0F,
            RUST
          },
          "#{BASE}: carry set" => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x30;
              cpu[Reg8::#{register}] = 0x21;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0x0E,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg8::#{register}] = 0x01;
            RUST
            expectations: <<~RUST
              A => 0x1F,
              hf => true,
            RUST
          },
          'C' => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
              cpu[Reg8::#{register}] = 0x30;
            RUST
            expectations: <<~RUST
              A => 0xF1,
              cf => true,
            RUST
          }
        }
      }
    },
    "SBC A, (HL)" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A] as u16;
        let operand2 = self.internal_ram[self[Reg16::HL] as usize] as u16 + self.get_flag(Flag::c) as u16;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result as u8;

        self.set_flag(Flag::c, carry);
      RUST
      testing: ->() {
        # Since the base logic is tested in the base test(s), the flag tests are simple.
        #
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x30;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x0F,
            RUST
          },
          "#{BASE}: carry set" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x30;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x21;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0x0E,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x00;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x01;
            RUST
            expectations: <<~RUST
              A => 0x1F,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0xF0;
            RUST
            expectations: <<~RUST
              A => 0x30,
              cf => true,
            RUST
          }
        }
      }
    },
    "SBC A, n" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A] as u16;
        let operand2 = *immediate as u16 + self.get_flag(Flag::c) as u16;

        let (result, carry) = operand1.overflowing_sub(operand2);
        self[Reg8::A] = result as u8;

        self.set_flag(Flag::c, carry);
      RUST
      testing: ->(register) {
        # Since the base logic is tested in the base test(s), the flag tests are simple.
        #
        {
          BASE => {
            extra_instruction_bytes: [0x21],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x30;
            RUST
            expectations: <<~RUST
              A => 0x0F,
            RUST
          },
          "#{BASE}: carry set" => {
            extra_instruction_bytes: [0x21],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x30;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0x0E,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0x0],
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            extra_instruction_bytes: [0x01],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
            RUST
            expectations: <<~RUST
              A => 0x1F,
              hf => true,
            RUST
          },
          'C' => {
            extra_instruction_bytes: [0xF0],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
            RUST
            expectations: <<~RUST
              A => 0x30,
              cf => true,
            RUST
          }
        }
      }
    },
    "AND A, r" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] & self[dst_register];
        self[Reg8::A] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg8::#{register}] = 0b0101_1111;
            RUST
            expectations: <<~RUST
              A => 0b0000_1001,
            RUST
          },
          "#{BASE}: A" => {
            skip: register != "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b1010_1001,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "AND A, (HL)" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] & self.internal_ram[self[Reg16::HL] as usize];
        self[Reg8::A] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0101_1111;
            RUST
            expectations: <<~RUST
              A => 0b0000_1001,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
            cpu[Reg8::A] = 0b1010_1001;
            cpu[Reg16::HL] = 0xCAFE;
            cpu.internal_ram[0xCAFE] = 0b0101_0110;
          RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "AND A, n" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] & *immediate;
        self[Reg8::A] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            extra_instruction_bytes: [0b0101_1111],
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b0000_1001,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0b0101_0110],
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "OR A, r" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] | self[dst_register];
        self[Reg8::A] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg8::#{register}] = 0b0101_1001;
            RUST
            expectations: <<~RUST
              A => 0b1111_1001,
            RUST
          },
          "#{BASE}: A" => {
            skip: register != "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b1010_1001,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "OR A, (HL)" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] | self.internal_ram[self[Reg16::HL] as usize];
        self[Reg8::A] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0101_1001;
            RUST
            expectations: <<~RUST
              A => 0b1111_1001,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "OR A, n" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] | *immediate;
        self[Reg8::A] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            extra_instruction_bytes: [0b0101_1001],
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b1111_1001,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0b0000_0000],
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "XOR A, r" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] ^ self[dst_register];
        self[Reg8::A] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg8::#{register}] = 0b0101_1001;
            RUST
            expectations: <<~RUST
              A => 0b1111_0000,
            RUST
          },
          # The `A, A` case fits nicely here, even with duplication.
          #
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg8::#{register}] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "XOR A, (HL)" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] ^ self.internal_ram[self[Reg16::HL] as usize];
        self[Reg8::A] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0101_1001;
            RUST
            expectations: <<~RUST
              A => 0b1111_0000,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "XOR A, n" => {
      operation_code: <<~RUST,
        let result = self[Reg8::A] ^ *immediate;
        self[Reg8::A] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            extra_instruction_bytes: [0b0101_1001],
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b1111_0000,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0b1010_1001],
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1010_1001;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "CP A, r" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_sub(operand2);

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);
      RUST
      testing: ->(register) {
        # In the `CP A, A` case, in essence, the only test case is the `Z` one.
        #
        {
          BASE => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x22;
              cpu[Reg8::#{register}] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x22,
              RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0x0;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
              nf => true,
              RUST
          },
          'H' => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg8::#{register}] = 0x01;
            RUST
            expectations: <<~RUST
              A => 0x20,
              nf => true,
              hf => true,
            RUST
          },
          'C' => {
            skip: register == "A",
            presets: <<~RUST,
              cpu[Reg8::A] = 0x70;
              cpu[Reg8::#{register}] = 0x90;
            RUST
            expectations: <<~RUST
              A => 0x70,
              nf => true,
              cf => true,
            RUST
          }
        }
      }
    },
    "CP A, (HL)" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = self.internal_ram[self[Reg16::HL] as usize];

        let (result, carry) = operand1.overflowing_sub(operand2);

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x42;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x00;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
              nf => true,
              RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x20;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x01;
            RUST
            expectations: <<~RUST
              A => 0x20,
              nf => true,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x70;
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0x90;
            RUST
            expectations: <<~RUST
              A => 0x70,
              nf => true,
              cf => true,
            RUST
          }
        }
      }
    },
    "CP A, n" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg8::A];
        let operand2 = *immediate;

        let (result, carry) = operand1.overflowing_sub(operand2);

        self.set_flag(Flag::c, carry);
        self.set_flag(Flag::n, true);
      RUST
      testing: ->(register) {
        {
          BASE => {
            extra_instruction_bytes: [0x21],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x42;
            RUST
            expectations: <<~RUST
              A => 0x42,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [0x0],
            expectations: <<~RUST
              A => 0x00,
              zf => true,
              nf => true,
            RUST
          },
          'H' => {
            extra_instruction_bytes: [0x0F],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x21;
            RUST
            expectations: <<~RUST
              A => 0x21,
              nf => true,
              hf => true,
            RUST
          },
          'C' => {
            extra_instruction_bytes: [0xF0],
            presets: <<~RUST,
              cpu[Reg8::A] = 0x10;
            RUST
            expectations: <<~RUST
              A => 0x10,
              nf => true,
              cf => true,
            RUST
          }
        }
      }
    },
    "INC r" => {
      operation_code: <<~RUST,
        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_add(operand2);
        self[dst_register] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: "cpu[Reg8::#{register}] = 0x21;",
            expectations: <<~RUST
              #{register} => 0x22,
            RUST
          },
          'Z' => {
            presets: "cpu[Reg8::#{register}] = 0xFF;",
            expectations: <<~RUST
              #{register} => 0x00,
              zf => true,
              hf => true,
            RUST
          },
          'H' => {
            presets: "cpu[Reg8::#{register}] = 0x1F;",
            expectations: <<~RUST
              #{register} => 0x20,
              hf => true,
            RUST
          }
        }
      }
    },
    "INC (HL)" => {
      operation_code: <<~RUST,
        let operand1 = self.internal_ram[self[Reg16::HL] as usize];
        let operand2 = 1;
        let (result, _) = operand1.overflowing_add(operand2);
        self.internal_ram[self[Reg16::HL] as usize] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu.internal_ram[0x0CAF] = 0x21;
              cpu[Reg16::HL] = 0x0CAF;
            RUST
            expectations: <<~RUST
              mem[0x0CAF] => [0x22],
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu.internal_ram[0x0CAF] = 0xFF;
              cpu[Reg16::HL] = 0x0CAF;
            RUST
            expectations: <<~RUST
              mem[0x0CAF] => [0x0],
              zf => true,
              hf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu.internal_ram[0x0CAF] = 0x1F;
              cpu[Reg16::HL] = 0x0CAF;
            RUST
            expectations: <<~RUST
              mem[0x0CAF] => [0x20],
              hf => true,
            RUST
          }
        }
      }
    },
    "DEC r" => {
      operation_code: <<~RUST,
        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_sub(operand2);
        self[dst_register] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: "cpu[Reg8::#{register}] = 0x22;",
            expectations: <<~RUST
              #{register} => 0x21,
            RUST
          },
          'Z' => {
            presets: "cpu[Reg8::#{register}] = 0x01;",
            expectations: <<~RUST
              #{register} => 0x00,
              zf => true,
            RUST
          },
          'H' => {
            presets: "cpu[Reg8::#{register}] = 0x20;",
            expectations: <<~RUST
              #{register} => 0x1F,
              hf => true,
            RUST
          }
        }
      }
    },
    "DEC (HL)" => {
      operation_code: <<~RUST,
        let operand1 = self.internal_ram[self[Reg16::HL] as usize];
        let operand2 = 1;
        let (result, _) = operand1.overflowing_sub(operand2);
        self.internal_ram[self[Reg16::HL] as usize] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu.internal_ram[0x0CAF] = 0x22;
              cpu[Reg16::HL] = 0x0CAF;
            RUST
            expectations: <<~RUST
              mem[0x0CAF] => [0x21],
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu.internal_ram[0x0CAF] = 0x01;
              cpu[Reg16::HL] = 0x0CAF;
            RUST
            expectations: <<~RUST
              mem[0x0CAF] => [0x00],
              zf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu.internal_ram[0x0CAF] = 0x20;
              cpu[Reg16::HL] = 0x0CAF;
            RUST
            expectations: <<~RUST
              mem[0x0CAF] => [0x1F],
              hf => true,
            RUST
          }
        }
      }
    },
    "ADD HL, rr" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg16::HL];
        let operand2 = self[dst_register];

        let (result, carry) = operand1.overflowing_add(operand2);
        self[Reg16::HL] = result;

        self.set_flag(Flag::c, carry);
      RUST
      testing: ->(register) {
        # Tests use duplicate values, accounting for `rr` = `HL`.
        #
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0x2121;
              cpu[Reg16::#{register}] = 0x2121;
            RUST
            expectations: <<~RUST
              HL => 0x4242,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xF000;
              cpu[Reg16::#{register}] = 0x1000;
            RUST
            expectations: <<~RUST
              HL => 0x0000,
              zf => true,
            RUST
          },
          'H' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0x1800;
              cpu[Reg16::#{register}] = 0x1800;
            RUST
            expectations: <<~RUST
              HL => 0x3000,
              hf => true,
            RUST
          },
          'C' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0x9000;
              cpu[Reg16::#{register}] = 0x9000;
            RUST
            expectations: <<~RUST
              HL => 0x2000,
              cf => true,
            RUST
          }
        }
      }
    },
    "ADD SP, n" => {
      operation_code: <<~RUST,
        let operand1 = self[Reg16::SP];
        // Ugly, but required, conversions.
        let operand2 = *immediate as i8 as i16 as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg16::SP] = result;
      RUST
      testing: ->(_) {
        {
          "#{BASE}: positive immediate" => {
            extra_instruction_bytes: [0x01],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0x2100;
            RUST
            expectations: <<~RUST
              SP => 0x2101,
            RUST
          },
          "#{BASE}: negative immediate" => {
            extra_instruction_bytes: [0xFF],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0x2100;
            RUST
            expectations: <<~RUST
              SP => 0x20FF,
            RUST
          },
          "H" => {
            extra_instruction_bytes: [0x01],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAEF;
            RUST
            expectations: <<~RUST
              SP => 0xCAF0,
              hf => true,
            RUST
          },
          "H: negative immediate" => {
            extra_instruction_bytes: [0xE1],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCA0F;
            RUST
            expectations: <<~RUST
              SP => 0xC9F0,
              hf => true,
            RUST
          },
          "C" => {
            extra_instruction_bytes: [0x10],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAFF;
            RUST
            expectations: <<~RUST
              SP => 0xCB0F,
              cf => true,
            RUST
          },
          "C: negative immediate" => {
            extra_instruction_bytes: [0xE0],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCA2F;
            RUST
            expectations: <<~RUST
              SP => 0xCA0F,
              cf => true,
            RUST
          },
        }
      }
    },
    "INC rr" => {
      operation_code: <<~RUST,
        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_add(operand2);
        self[dst_register] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: "cpu[Reg16::#{register}] = 0xFFFF;",
            expectations: <<~RUST
              #{register} => 0x0000,
            RUST
          },
        }
      }
    },
    "DEC rr" => {
      operation_code: <<~RUST,
        let operand1 = self[dst_register];
        let operand2 = 1;

        let (result, _) = operand1.overflowing_sub(operand2);
        self[dst_register] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: "cpu[Reg16::#{register}] = 0x0000;",
            expectations: <<~RUST
              #{register} => 0xFFFF,
            RUST
          },
        }
      }
    },
    "SWAP r" => {
      operation_code: <<~RUST,
        let result = self[dst_register] >> 4 | ((self[dst_register] & 0b0000_1111) << 4);
        self[dst_register] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: "cpu[Reg8::#{register}] = 0x21;",
            expectations: <<~RUST
              #{register} => 0x12,
            RUST
          },
          "Z" => {
            presets: "cpu[Reg8::#{register}] = 0x00;",
            expectations: <<~RUST
              #{register} => 0x00,
              zf => true,
            RUST
          },
        }
      }
    },
    "SWAP (HL)" => {
      operation_code: <<~RUST,
        let value = self.internal_ram[self[Reg16::HL] as usize];
        let result = value >> 4 | ((value & 0b0000_1111) << 4);
        self.internal_ram[self[Reg16::HL] as usize] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu.internal_ram[0xCAFE] = 0x21;
              cpu[Reg16::HL] = 0xCAFE;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0x12],
            RUST
          },
          "Z" => {
            presets: <<~RUST,
              cpu.internal_ram[0xCAFE] = 0x00;
              cpu[Reg16::HL] = 0xCAFE;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0x00],
              zf => true,
            RUST
          },
        }
      }
    },
    # Adapted from SameBoy.
    #
    # Currently has just smoke testing; this is the best candidate for adding a large amount of unit
    # testing.
    # There is at least one table with all the valid values, at https://github.com/ruyrybeyro/daatable/blob/master/daaoutput.txt,
    # which could be used to isolate the meaningful UTs.
    #
    "DAA" => {
      operation_code: <<~RUST,
        let mut result = self[Reg8::A] as u16;

        self[Reg8::A] = 0x00;
        self.set_flag(Flag::z, true);

        if self.get_flag(Flag::n) {
            if self.get_flag(Flag::h) {
                result = (result - 0x06) & 0xFF;
            }

            if self.get_flag(Flag::c) {
                result -= 0x60;
            }
        }
        else {
            if self.get_flag(Flag::h) || (result & 0x0F) > 0x09 {
                result += 0x06;
            }

            if self.get_flag(Flag::c) || result > 0x9F {
                result += 0x60;
            }
        }

        if (result & 0xFF) == 0 {
            self.set_flag(Flag::z, true);
        }

        if (result & 0x100) == 0x100 {
            self.set_flag(Flag::c, true);
        }

        self.set_flag(Flag::h, false);
        self[Reg8::A] = result as u8;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x1B;
            RUST
            expectations: <<~RUST
              A => 0x21,
            RUST
          },
          "H" => {skip: true},
          "Z" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0x0;
            RUST
            expectations: <<~RUST
              A => 0x00,
              zf => true,
            RUST
          },
          "C" => {skip: true},
          "N" => {skip: true},
        }
      }
    },
    "CPL" => {
      operation_code: <<~RUST,
        self[Reg8::A] = !self[Reg8::A];
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0xF0;
            RUST
            expectations: <<~RUST
              A => 0x0F,
            RUST
          },
        }
      }
    },
    "CCF" => {
      operation_code: <<~RUST,
        let cf_value = self.get_flag(Flag::c);
        self.set_flag(Flag::c, !cf_value);
      RUST
      testing: ->() {
        {
          BASE => { skip: true },
          "C: -> true" => {
            presets: <<~RUST,
              cpu.set_flag(Flag::c, false);
            RUST
            expectations: <<~RUST
              cf => true,
            RUST
          },
          "C: -> false" => {
            presets: <<~RUST,
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              cf => false,
            RUST
          },
        }
      }
    },
    # The expectations are added automatically by the test generator.
    #
    "SCF" => {
      operation_code: <<~RUST,
        self.set_flag(Flag::c, true);
      RUST
      testing: ->() {
        {
          "#{BASE}: C -> true" => {
            presets: <<~RUST,
              cpu.set_flag(Flag::c, false);
            RUST
          },
          "#{BASE}: C -> false" => {
            presets: <<~RUST,
              cpu.set_flag(Flag::c, true);
            RUST
          },
        }
      }
    },
    "NOP" => {
      operation_code: "",
      testing: -> {
        {
          BASE => {
          }
        }
      }
    },
    "RLCA" => {
      operation_code: <<~RUST,
        self.set_flag(Flag::c, (self[Reg8::A] & 0b1000_0000) != 0);
        let result = self[Reg8::A].rotate_left(1);
        self[Reg8::A] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              A => 0b1111_0000,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              A => 0b1110_0001,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RLA" => {
      operation_code: <<~RUST,
        let new_carry = (self[Reg8::A] & 0b1000_0000) != 0;

        let result = self[Reg8::A].wrapping_shl(1) | self.get_flag(Flag::c) as u8;
        self[Reg8::A] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              A => 0b1111_0000,
              cf => false,
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0111_1000;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0b1111_0001,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              A => 0b1110_0000,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RRCA" => {
      operation_code: <<~RUST,
        self.set_flag(Flag::c, (self[Reg8::A] & 0b0000_0001) != 0);
        let result = self[Reg8::A].rotate_right(1);
        self[Reg8::A] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              A => 0b0000_1111,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              A => 0b01000_0111,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RRA" => {
      operation_code: <<~RUST,
        let new_carry = (self[Reg8::A] & 0b0000_0001) != 0;

        let mut result = self[Reg8::A].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self[Reg8::A] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              A => 0b0000_1111,
              cf => false,
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0001_1110;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              A => 0b1000_1111,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              A => 0b0000_0111,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::A] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              A => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RLC r" => {
      operation_code: <<~RUST,
        self.set_flag(Flag::c, (self[dst_register] & 0b1000_0000) != 0);
        let result = self[dst_register].rotate_left(1);
        self[dst_register] = result;
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1111_0000,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1110_0001,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RLC (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;

        self.set_flag(Flag::c, (self.internal_ram[address] & 0b1000_0000) != 0);
        let result = self.internal_ram[address].rotate_left(1);

        self.internal_ram[address] = result;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1111_0000],
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1110_0001],
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0000],
              zf => true,
            RUST
          },
        }
      }
    },
    "RL r" => {
      operation_code: <<~RUST,
        let new_carry = (self[dst_register] & 0b1000_0000) != 0;

        let result = self[dst_register].wrapping_shl(1) | self.get_flag(Flag::c) as u8;
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->(register) {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1111_0000,
              cf => false,
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0111_1000;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              #{register} => 0b1111_0001,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1110_0000,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RL (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b1000_0000) != 0;

        let result = self.internal_ram[address].wrapping_shl(1) | self.get_flag(Flag::c) as u8;
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              cf => false,
              mem[0xCAFE] => [0b1111_0000],
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0111_1000;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              cf => false,
              mem[0xCAFE] => [0b1111_0001],
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              cf => true,
              mem[0xCAFE] => [0b1110_0000],
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              zf => true,
              mem[0xCAFE] => [0b0000_0000],
            RUST
          },
        }
      }
    },
    "RRC r" => {
      operation_code: <<~RUST,
        let new_carry = (self[dst_register] & 0b0000_0001) != 0;

        let mut result = self[dst_register].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->(register) {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_1111,
              cf => false,
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0001_1110;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              #{register} => 0b1000_1111,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0111,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RRC (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b0000_0001) != 0;

        let mut result = self.internal_ram[address].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_1111],
              cf => false,
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0001_1110;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1000_1111],
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0111],
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0000],
              zf => true,
            RUST
          },
        }
      }
    },
    "RR r" => {
      operation_code: <<~RUST,
        let new_carry = (self[dst_register] & 0b0000_0001) != 0;

        let mut result = self[dst_register].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->(register) {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_1111,
              cf => false,
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0001_1110;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              #{register} => 0b1000_1111,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0111,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "RR (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b0000_0001) != 0;

        let mut result = self.internal_ram[address].wrapping_shr(1);
        if self.get_flag(Flag::c) {
          result |= 0b1000_0000;
        }
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          "#{BASE}: carry was not set" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_1111],
              cf => false,
            RUST
          },
          "#{BASE}: carry was set" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0001_1110;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1000_1111],
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0111],
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0000],
              zf => true,
            RUST
          },
        }
      }
    },
    "SLA r" => {
      operation_code: <<~RUST,
        let new_carry = (self[dst_register] & 0b1000_0000) != 0;

        let result = self[dst_register].wrapping_shl(1);
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0111_1000;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              #{register} => 0b1111_0000,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1110_0000,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "SLA (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b1000_0000) != 0;

        let result = self.internal_ram[address].wrapping_shl(1);
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0111_1000;
              cpu.set_flag(Flag::c, true);
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1111_0000],
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1110_0000],
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0000],
              zf => true,
            RUST
          },
        }
      }
    },
    "SRA r" => {
      operation_code: <<~RUST,
        let new_carry = (self[dst_register] & 0b0000_0001) != 0;
        let old_msb = self[dst_register] & 0b1000_0000;

        let result = self[dst_register].wrapping_shr(1) | old_msb;
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->(register) {
        {
          "#{BASE}: MSB=0" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_1111,
              cf => false,
            RUST
          },
          "#{BASE}: MSB=1" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1001_1110;
            RUST
            expectations: <<~RUST
              #{register} => 0b1100_1111,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0111,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "SRA (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;

        let new_carry = (self.internal_ram[address] & 0b0000_0001) != 0;
        let old_msb = self.internal_ram[address] & 0b1000_0000;

        let result = self.internal_ram[address].wrapping_shr(1) | old_msb;
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          "#{BASE}: MSB=0" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0001_1110;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_1111],
              cf => false,
            RUST
          },
          "#{BASE}: MSB=1" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1001_1110;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1100_1111],
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_1111;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0111],
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0000],
              zf => true,
            RUST
          },
        }
      }
    },
    "SRL r" => {
      operation_code: <<~RUST,
        let new_carry = (self[dst_register] & 0b1000_0000) != 0;

        let result = self[dst_register].wrapping_shl(1);
        self[dst_register] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->(register) {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1111_0000,
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1110_0000,
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b0000_0000,
              zf => true,
            RUST
          },
        }
      }
    },
    "SRL (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let new_carry = (self.internal_ram[address] & 0b1000_0000) != 0;

        let result = self.internal_ram[address].wrapping_shl(1);
        self.internal_ram[address] = result;

        self.set_flag(Flag::c, new_carry);
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0111_1000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1111_0000],
              cf => false,
            RUST
          },
          "C" => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1110_0000],
              cf => true,
            RUST
          },
          'Z' => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b0000_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b0000_0000],
              zf => true,
            RUST
          },
        }
      }
    },
    "BIT n, r" => {
      operation_code: <<~RUST,
        let bitmask = 1 << *immediate;

        let result = self[src_register] & bitmask;
      RUST
      testing: ->(_, register) {
        {
          BASE => {
            extra_instruction_bytes: [4],
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              zf => false,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [3],
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              zf => true,
            RUST
          },
        }
      }
    },
    "BIT n, (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let bitmask = 1 << *immediate;

        let result = self.internal_ram[address] & bitmask;
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [4],
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              zf => false,
            RUST
          },
          'Z' => {
            extra_instruction_bytes: [3],
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              zf => true,
            RUST
          },
        }
      }
    },
    "SET n, r" => {
      operation_code: <<~RUST,
        let bitmask = 1 << *immediate;

        self[src_register] |= bitmask;
      RUST
      testing: ->(_, register) {
        {
          BASE => {
            extra_instruction_bytes: [3],
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1111_1000,
            RUST
          },
        }
      }
    },
    "SET n, (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let bitmask = 1 << *immediate;

        self.internal_ram[address] |= bitmask;
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [3],
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1111_1000],
            RUST
          },
        }
      }
    },
    "RES n, r" => {
      operation_code: <<~RUST,
        let bitmask = !(1 << *immediate);

        self[src_register] &= bitmask;
      RUST
      testing: ->(_, register) {
        {
          BASE => {
            extra_instruction_bytes: [4],
            presets: <<~RUST,
              cpu[Reg8::#{register}] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              #{register} => 0b1110_0000,
            RUST
          },
        }
      }
    },
    "RES n, (HL)" => {
      operation_code: <<~RUST,
        let address = self[Reg16::HL] as usize;
        let bitmask = !(1 << *immediate);

        self.internal_ram[address] &= bitmask;
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [4],
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
              cpu.internal_ram[0xCAFE] = 0b1111_0000;
            RUST
            expectations: <<~RUST
              mem[0xCAFE] => [0b1110_0000],
            RUST
          },
        }
      }
    },
    "JP nn" => {
      operation_code: <<~RUST,
        self[Reg16::PC] = *immediate;
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0xEF, 0xBE],
            expectations: <<~RUST
              PC => 0xBEEF,
            RUST
          },
        }
      }
    },
    "JP cc, nn" => {
      operation_code: <<~RUST,
        if self.get_flag(flag) == flag_condition {
          self[Reg16::PC] = *immediate;
        }
        else {
          self[Reg16::PC] += 3;
        }
      RUST
      testing: ->(flag, flag_value, condition_matching) {
        {
          "absolute jump" => {
            extra_instruction_bytes: [0xEF, 0xBE],
            presets: <<~RUST,
              cpu.set_flag(Flag::#{flag}, #{flag_value});
            RUST
            expectations: ("PC => 0xBEEF," if condition_matching),
          }
        }
      }
    },
    "JP (HL)" => {
      operation_code: <<~RUST,
        self[Reg16::PC] = self[Reg16::HL];
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::HL] = 0xCAFE;
            RUST
            expectations: <<~RUST
              PC => 0xCAFE,
            RUST
          },
        }
      }
    },
    "JR n" => {
      # See `LDHL, SP` for the logic.
      #
      operation_code: <<~RUST,
        let operand1 = self[Reg16::PC];
        let operand2 = *immediate as i8 as i16 as u16;

        let (result, _) = operand1.overflowing_add(operand2);
        self[Reg16::PC] = result;
      RUST
      testing: ->(_) {
        {
          "#{BASE}: positive" => {
            extra_instruction_bytes: [0x10],
            expectations: <<~RUST
              PC => 0x0031,
            RUST
          },
          "#{BASE}: negative" => {
            extra_instruction_bytes: [0xF0],
            expectations: <<~RUST
              PC => 0x0011,
            RUST
          },
          "#{BASE}: overflow (positive)" => {
            extra_instruction_bytes: [0x20],
            presets: <<~RUST,
              cpu[Reg16::PC] = 0xFFEF;
            RUST
            expectations: <<~RUST
              PC => 0x000F,
            RUST
          },
        }
      }
    },
    "JR cc, n" => {
      # See `LDHL, SP` for the logic.
      #
      operation_code: <<~RUST,
        if self.get_flag(flag) == flag_condition {
          let operand1 = self[Reg16::PC];
          let operand2 = *immediate as i8 as i16 as u16;

          let (result, _) = operand1.overflowing_add(operand2);
          self[Reg16::PC] = result;
        }
        else {
          self[Reg16::PC] += 2;
        }
      RUST
      testing: ->(flag, flag_value, condition_matching) {
        {
          "positive jump" => {
            extra_instruction_bytes: [0x10],
            presets: <<~RUST,
              cpu.set_flag(Flag::#{flag}, #{flag_value});
            RUST
            expectations: ("PC => 0x0031," if condition_matching),
          },
          "negative jump" => {
            extra_instruction_bytes: [0xF0],
            presets: <<~RUST,
              cpu.set_flag(Flag::#{flag}, #{flag_value});
            RUST
            expectations: ("PC => 0x0011," if condition_matching),
          },
          "positive jump, with overflow" => {
            extra_instruction_bytes: [0x1F],
            presets: <<~RUST,
              cpu.set_flag(Flag::#{flag}, #{flag_value});
              cpu[Reg16::PC] = 0xFFF0;
            RUST
            expectations: ("PC => 0x#{condition_matching ? "000F" : "FFF2"},"),
          },
        }
      }
    },
    "CALL nn" => {
      operation_code: <<~RUST,
        let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
        self[Reg16::SP] = new_sp;

        let (stored_address, _) = self[Reg16::PC].overflowing_add(3);
        let pushed_bytes = stored_address.to_le_bytes();
        self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);

        self[Reg16::PC] = *immediate;
      RUST
      testing: ->(_) {
        {
          BASE => {
            extra_instruction_bytes: [0x21, 0x30],
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAFE;
            RUST
            expectations: <<~RUST
              SP => 0xCAFC,
              PC => 0x3021,
              mem[0xCAFC] => [0x24, 0x00],
            RUST
          },
          # Disabled until it's clear what's the behavior.
          #
          # "#{BASE}: PC wraparound" => {
          #   extra_instruction_bytes: [0x21, 0x30],
          #   presets: <<~RUST,
          #     cpu[Reg16::PC] = 0xFFFF;
          #     cpu[Reg16::SP] = 0xCAFE;
          #   RUST
          #   expectations: <<~RUST
          #     SP => 0xCAFC,
          #     PC => 0x3021,
          #     mem[0xCAFC] => [0x02, 0x00],
          #   RUST
          # },
          # "#{BASE}: stack wraparound" => {
          #   extra_instruction_bytes: [0x21, 0x30],
          #   expectations: <<~RUST
          #     SP => 0xFFFE,
          #     PC => 0x3021,
          #     mem[0xFFFE] => [0x24, 0x00],
          #   RUST
          # },
        }
      }
    },
    "CALL cc, nn" => {
      operation_code: <<~RUST,
        if self.get_flag(flag) == flag_condition {
            let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
            self[Reg16::SP] = new_sp;

            let (stored_address, _) = self[Reg16::PC].overflowing_add(3);
            let pushed_bytes = stored_address.to_le_bytes();
            self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);

            self[Reg16::PC] = *immediate;
        } else {
            self[Reg16::PC] += 3;
        }
      RUST
      testing: ->(flag, flag_value, condition_matching) {
        {
          "no wraparounds" => {
            extra_instruction_bytes: [0x21, 0x30],
            presets: <<~RUST,
              cpu.set_flag(Flag::#{flag}, #{flag_value});
              cpu[Reg16::SP] = 0xCAFE;
            RUST
            expectations: (<<~RUST if condition_matching)
              SP => 0xCAFC,
              PC => 0x3021,
              mem[0xCAFC] => [0x24, 0x00],
            RUST
          },
          # Disabled until it's clear what's the behavior.
          #
          # "PC wraparound" => {
          #   extra_instruction_bytes: [0x21, 0x30],
          #   presets: <<~RUST,
          #     cpu.set_flag(Flag::#{flag}, #{flag_value});
          #     cpu[Reg16::PC] = 0xFFFF;
          #     cpu[Reg16::SP] = 0xCAFE;
          #   RUST
          #   expectations: (<<~RUST if condition_matching)
          #     SP => 0xCAFC,
          #     PC => 0x3021,
          #     mem[0xCAFC] => [0x02, 0x00],
          #   RUST
          # },
          # "stack wraparound" => {
          #   extra_instruction_bytes: [0x21, 0x30],
          #   presets: <<~RUST,
          #     cpu.set_flag(Flag::#{flag}, #{flag_value});
          #   RUST
          #   expectations: (<<~RUST if condition_matching)
          #     SP => 0xFFFE,
          #     PC => 0x3021,
          #     mem[0xFFFE] => [0x24, 0x00],
          #   RUST
          # },
        }
      }
    },
    # Currently unclear what happens with invalid values.
    #
    # This is a somewhat ugly, because both the execution code and the execution code are oblivious
    # of the opcode.
    #
    "RST" => {
      operation_code: <<~RUST,
        let (new_sp, _) = self[Reg16::SP].overflowing_sub(2);
        self[Reg16::SP] = new_sp;

        let pushed_bytes = self[Reg16::PC].to_le_bytes();
        self.internal_ram[new_sp as usize..new_sp as usize + 2].copy_from_slice(&pushed_bytes);

        let destination_address = match self.internal_ram[self[Reg16::PC] as usize] {
            0xC7 => 0x00,
            0xCF => 0x08,
            0xD7 => 0x10,
            0xDF => 0x18,
            0xE7 => 0x20,
            0xEF => 0x28,
            0xF7 => 0x30,
            0xFF => 0x38,
            _ => panic!(),
        };

        self[Reg16::PC] = destination_address;
      RUST
      testing: ->() {
        # Hehe. Workaround the fact that address is hardcoded. Requires the instruction opcodes to be
        # stored/retrieved in the same order.
        #
        $opcode_addresses_iterator ||= [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38].to_enum

        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAFE;
            RUST
            expectations: <<~RUST
              SP => 0xCAFC,
              PC => 0x#{"%02X" % $opcode_addresses_iterator.next},
              mem[0xCAFC] => [0x21, 0x00],
            RUST
          },
        }
      }
    },
    "RET" => {
      operation_code: <<~RUST,
        self[Reg16::PC] = u16::from_le_bytes(self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap());

        let (new_sp, _) = self[Reg16::SP].overflowing_add(2);
        self[Reg16::SP] = new_sp;
      RUST
      testing: ->() {
        {
          BASE => {
            presets: <<~RUST,
              cpu[Reg16::SP] = 0xCAFE;
              cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);
            RUST
            expectations: <<~RUST
              SP => 0xCB00,
              PC => 0x2130,
            RUST
          },
        }
      }
    },
    "RET cc" => {
      operation_code: <<~RUST,
        if self.get_flag(flag) == flag_condition {
            self[Reg16::PC] = u16::from_le_bytes(self.internal_ram[self[Reg16::SP] as usize..self[Reg16::SP] as usize + 2].try_into().unwrap());

            let (new_sp, _) = self[Reg16::SP].overflowing_add(2);
            self[Reg16::SP] = new_sp;
        } else {
            self[Reg16::PC] += 1;
        }
      RUST
      testing: ->(flag, flag_value, condition_matching) {
        {
          "no wraparounds" => {
            presets: <<~RUST,
              cpu.set_flag(Flag::#{flag}, #{flag_value});
              cpu[Reg16::SP] = 0xCAFE;
              cpu.internal_ram[0xCAFE..=0xCAFF].copy_from_slice(&[0x30, 0x21]);
            RUST
            expectations: (<<~RUST if condition_matching)
              SP => 0xCB00,
              PC => 0x2130,
            RUST
          },
          # Disabled until it's clear what's the behavior.
          #
          # "PC wraparound" => {
          # "stack wraparound" => {
        }
      }
    },
  }
end
