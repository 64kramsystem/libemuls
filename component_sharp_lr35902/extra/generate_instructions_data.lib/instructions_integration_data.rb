module InstructionsIntegrationData
  INTEGRATION_DATA = {
    "LD r, n" => {
      opcodes: [
        0x06,
        0x0E,
        0x16,
        0x1E,
        0x26,
        0x2E,
        0x3E,
      ],
    },
    "LD r1, r2" => {
      opcodes: [
        0x78,
        0x79,
        0x7A,
        0x7B,
        0x7C,
        0x7D,
        0x41,
        0x42,
        0x43,
        0x44,
        0x45,
        0x48,
        0x4A,
        0x4B,
        0x4C,
        0x4D,
        0x50,
        0x51,
        0x53,
        0x54,
        0x55,
        0x58,
        0x59,
        0x5A,
        0x5C,
        0x5D,
        0x60,
        0x61,
        0x62,
        0x63,
        0x65,
        0x68,
        0x69,
        0x6A,
        0x6B,
        0x6C,
        0x47,
        0x4F,
        0x57,
        0x5F,
        0x67,
        0x6F,
        0x7F,
        0x40,
        0x49,
        0x52,
        0x5B,
        0x64,
        0x6D,
      ],
    },
    "LD r1, (rr2)" => {
      opcodes: [
        0x46,
        0x4E,
        0x56,
        0x5E,
        0x7E,
        0x0A,
        0x1A,
        0x66,
        0x6E,
      ],
    },
    "LD (rr1), r2" => {
      opcodes: [
        0x70,
        0x71,
        0x72,
        0x73,
        0x74,
        0x75,
        0x02,
        0x12,
        0x77,
      ],
    },
    "LD (HL), n" => {
      opcodes: [
        0x36,
      ],
    },
    "LD A, (nn)" => {
      opcodes: [
        0xFA,
      ],
    },
    "LD (nn), A" => {
      opcodes: [
        0xEA,
      ],
    },
    "LD A, (C)" => {
      opcodes: [
        0xF2,
      ],
    },
    "LD (C), A" => {
      opcodes: [
        0xE2,
      ],
    },
    "LDD A, (HL)" => {
      opcodes: [
        0x3A,
      ],
    },
    "LDD (HL), A" => {
      opcodes: [
        0x32,
      ],
    },
    "LDI A, (HL)" => {
      opcodes: [
        0x2A,
      ],
    },
    "LDI (HL), A" => {
      opcodes: [
        0x22,
      ],
    },
    "LDH (n), A" => {
      opcodes: [
        0xE0,
      ],
    },
    "LDH A, (n)" => {
      opcodes: [
        0xF0,
      ],
    },
    "LD rr, nn" => {
      opcodes: [
        0x01,
        0x11,
        0x21,
        0x31,
      ],
    },
    "LD SP, HL" => {
      opcodes: [
        0xF9,
      ],
    },
    "LDHL SP, n" => {
      opcodes: [
        0xF8,
      ],
      transform_data: ->(data) do
        data['operands'].shift
        data['operands'].push({"name" => "d8"})
      end,
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "LD (nn), SP" => {
      opcodes: [
        0x08,
      ],
    },
    "PUSH rr" => {
      opcodes: [
        0xF5,
        0xC5,
        0xD5,
        0xE5,
      ],
    },
    "POP rr" => {
      opcodes: [
        0xC1,
        0xD1,
        0xE1,
      ],
    },
    # This belongs to the previous instruction, however, it has a different flags profile, so it's
    # better to separate it.
    #
    "POP AF" => {
      opcodes: [
        0xF1,
      ],
    },
    "ADD A, r" => {
      opcodes: [
        0x87,
        0x80,
        0x81,
        0x82,
        0x83,
        0x84,
        0x85,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "ADD A, (HL)" => {
      opcodes: [
        0x86,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "ADD A, n" => {
      opcodes: [
        0xC6,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "ADC A, r" => {
      opcodes: [
        0x8F,
        0x88,
        0x89,
        0x8A,
        0x8B,
        0x8C,
        0x8D,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "ADC A, (HL)" => {
      opcodes: [
        0x8E,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "ADC A, n" => {
      opcodes: [
        0xCE,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "SUB A, r" => {
      opcodes: [
        0x97,
        0x90,
        0x91,
        0x92,
        0x93,
        0x94,
        0x95,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
        # Make the flags consistent across the instruction. While defining the outcome as fixed to `1`
        # for `SUB A, A`, this makes that flag outcome specific; by changing it this way, we make the
        # outcome generic, and consistent across the instruction.
        # The same has been applied to `XOR A, r` and `CP A, r`.
        #
        data['flags']['Z'] = 'Z'
      end,
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "SUB A, (HL)" => {
      opcodes: [
        0x96,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end,
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "SUB A, n" => {
      opcodes: [
        0xD6,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end,
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "SBC A, r" => {
      opcodes: [
        0x9F,
        0x98,
        0x99,
        0x9A,
        0x9B,
        0x9C,
        0x9D,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "SBC A, (HL)" => {
      opcodes: [
        0x9E,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "SBC A, n" => {
      opcodes: [
        0xDE,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "AND A, r" => {
      opcodes: [
        0xA7,
        0xA0,
        0xA1,
        0xA2,
        0xA3,
        0xA4,
        0xA5,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "AND A, (HL)" => {
      opcodes: [
        0xA6,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "AND A, n" => {
      opcodes: [
        0xE6,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "OR A, r" => {
      opcodes: [
        0xB7,
        0xB0,
        0xB1,
        0xB2,
        0xB3,
        0xB4,
        0xB5,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "OR A, (HL)" => {
      opcodes: [
        0xB6,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "OR A, n" => {
      opcodes: [
        0xF6,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "XOR A, r" => {
      opcodes: [
        0xAF,
        0xA8,
        0xA9,
        0xAA,
        0xAB,
        0xAC,
        0xAD,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
        data['flags']['Z'] = 'Z'
      end
    },
    "XOR A, (HL)" => {
      opcodes: [
        0xAE,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "XOR A, n" => {
      opcodes: [
        0xEE,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end
    },
    "CP A, r" => {
      opcodes: [
        0xBF,
        0xB8,
        0xB9,
        0xBA,
        0xBB,
        0xBC,
        0xBD,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
        data['flags']['Z'] = 'Z'
      end,
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "CP A, (HL)" => {
      opcodes: [
        0xBE,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end,
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "CP A, n" => {
      opcodes: [
        0xFE,
      ],
      transform_data: ->(data) do
        data['operands'].unshift({"name" => "A"})
      end,
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "INC r" => {
      opcodes: [
        0x3C,
        0x04,
        0x0C,
        0x14,
        0x1C,
        0x24,
        0x2C,
      ],
      carry_flag_positions: {
        "H" => 4
      }
    },
    "INC (HL)" => {
      opcodes: [
        0x34,
      ],
      carry_flag_positions: {
        "H" => 4
      }
    },
    "DEC r" => {
      opcodes: [
        0x3D,
        0x05,
        0x0D,
        0x15,
        0x1D,
        0x25,
        0x2D,
      ],
      carry_flag_positions: {
        "H" => 4,
      }
    },
    "DEC (HL)" => {
      opcodes: [
        0x35,
      ],
      carry_flag_positions: {
        "H" => 4
      }
    },
    "ADD HL, rr" => {
      opcodes: [
        0x09,
        0x19,
        0x29,
        0x39,
      ],
      carry_flag_positions: {
        "H" => 12,
        "C" => 16
      }
    },
    "ADD SP, n" => {
      opcodes: [
        0xE8,
      ],
      carry_flag_positions: {
        "H" => 4,
        "C" => 8
      }
    },
    "INC rr" => {
      opcodes: [
        0x03,
        0x13,
        0x23,
        0x33,
      ],
    },
    "DEC rr" => {
      opcodes: [
        0x0B,
        0x1B,
        0x2B,
        0x3B,
      ],
    },
    "SWAP r" => {
      prefix: 0xCB,
      opcodes: [
        0x37,
        0x30,
        0x31,
        0x32,
        0x33,
        0x34,
        0x35,
      ],
    },
    "SWAP (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x36,
      ],
    },
    "DAA" => {
      opcodes: [
        0x27,
      ],
    },
    "CPL" => {
      opcodes: [
        0x2F,
      ],
    },
    "CCF" => {
      opcodes: [
        0x3F,
      ],
    },
    "SCF" => {
      opcodes: [
        0x37,
      ],
    },
    "NOP" => {
      opcodes: [
        0x00
      ],
    },
    "HALT" => {
      opcodes: [
        0x76,
      ],
    },
    "STOP" => {
      opcodes: [
        0x10,
      ],
    },
    "DI" => {
      opcodes: [
        0xF3,
      ],
    },
    "EI" => {
      opcodes: [
        0xFB,
      ],
    },
    "RLCA" => {
      opcodes: [
        0x07,
      ],
      # Bug fix: See https://git.io/JTWwx.
      #
      transform_data: ->(data) do
        data['flags']['Z'] = 'Z'
      end,
    },
    "RLA" => {
      opcodes: [
        0x17,
      ],
      # Bug fix: See https://git.io/JTWwx.
      #
      transform_data: ->(data) do
        data['flags']['Z'] = 'Z'
      end,
    },
    "RRCA" => {
      opcodes: [
        0x0F,
      ],
      # Bug fix: See https://git.io/JTWwx.
      #
      transform_data: ->(data) do
        data['flags']['Z'] = 'Z'
      end,
    },
    "RRA" => {
      opcodes: [
        0x1F,
      ],
      # Bug fix: See https://git.io/JTWwx.
      #
      transform_data: ->(data) do
        data['flags']['Z'] = 'Z'
      end,
    },
    "RLC r" => {
      prefix: 0xCB,
      opcodes: [
        0x07,
        0x00,
        0x01,
        0x02,
        0x03,
        0x04,
        0x05,
      ],
    },
    "RLC (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x06,
      ],
    },
    "RL r" => {
      prefix: 0xCB,
      opcodes: [
        0x17,
        0x10,
        0x11,
        0x12,
        0x13,
        0x14,
        0x15,
      ],
    },
    "RL (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x16,
      ],
    },
    "RRC r" => {
      prefix: 0xCB,
      opcodes: [
        0x0F,
        0x08,
        0x09,
        0x0A,
        0x0B,
        0x0C,
        0x0D,
      ],
    },
    "RRC (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x0E,
      ],
    },
    "RR r" => {
      prefix: 0xCB,
      opcodes: [
        0x1F,
        0x18,
        0x19,
        0x1A,
        0x1B,
        0x1C,
        0x1D,
      ],
    },
    "RR (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x1E,
      ],
    },
    "SLA r" => {
      prefix: 0xCB,
      opcodes: [
        0x27,
        0x20,
        0x21,
        0x22,
        0x23,
        0x24,
        0x25,
      ],
    },
    "SLA (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x26,
      ],
    },
    "SRA r" => {
      prefix: 0xCB,
      opcodes: [
        0x2F,
        0x28,
        0x29,
        0x2A,
        0x2B,
        0x2C,
        0x2D,
      ],
    },
    "SRA (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x2E,
      ],
    },
    "SRL r" => {
      prefix: 0xCB,
      opcodes: [
        0x3F,
        0x38,
        0x39,
        0x3A,
        0x3B,
        0x3C,
        0x3D,
      ],
    },
    "SRL (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x3E,
      ],
    },
    "BIT n, r" => {
      prefix: 0xCB,
      opcodes: [
        0x47,
        0x40,
        0x41,
        0x42,
        0x43,
        0x44,
        0x45,
      ],
    },
    "BIT n, (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x46,
      ],
    },
    "SET n, r" => {
      prefix: 0xCB,
      opcodes: [
        0xC7,
        0xC0,
        0xC1,
        0xC2,
        0xC3,
        0xC4,
        0xC5,
      ],
    },
    "SET n, (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0xC6,
      ],
    },
    "RES n, r" => {
      prefix: 0xCB,
      opcodes: [
        0x87,
        0x80,
        0x81,
        0x82,
        0x83,
        0x84,
        0x85,
      ],
    },
    "RES n, (HL)" => {
      prefix: 0xCB,
      opcodes: [
        0x86,
      ],
    },
    "JP nn" => {
      opcodes: [
        0xC3,
      ],
    },
    "JP cc, nn" => {
      opcodes: [
        0xC2,
        0xCA,
        0xD2,
        0xDA,
      ],
    },
    "JP (HL)" => {
      opcodes: [
        0xE9,
      ],
    },
    "JR n" => {
      opcodes: [
        0x18,
      ],
    },
    "JR cc, n" => {
      opcodes: [
        0x20,
        0x28,
        0x30,
        0x38,
      ],
    },
    "CALL nn" => {
      opcodes: [
        0xCD,
      ],
    },
    "CALL cc, nn" => {
      opcodes: [
        0xC4,
        0xCC,
        0xD4,
        0xDC,
      ],
    },
    "RST n" => {
      opcodes: [
        0xC7,
        0xCF,
        0xD7,
        0xDF,
        0xE7,
        0xEF,
        0xF7,
        0xFF,
      ],
    },
    "RET" => {
      opcodes: [
        0xC9,
      ],
    },
    "RET cc" => {
      opcodes: [
        0xC0,
        0xC8,
        0xD0,
        0xD8,
      ],
    },
    "RETI" => {
      opcodes: [
        0xD9,
      ],
    },
  }.freeze
end
