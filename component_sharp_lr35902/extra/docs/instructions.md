## Considerations

- the maximum amount of explicit and implied operands are three (two registers, one immediate)
- there is never more than one immediate

## List

Conventions:

- `r`: 8-bit register
- `rr`: 16-bit register
- `n`: 8-bit immediate
- `nn`: 16-bit immediate
- `(<op>)`: indirect reference

### 65. 8-Bit Loads

LD r, n
LD r1, r2
LD r1, (r2)
LD (r1), r2
LD (HL), n
LD A, (nn)
LD (nn), A
LD A, (C)
LD (C), A
LDD A, (HL)
LDD (HL), A
LDI A, (HL)
LDI (HL), A
LDH (n), A
LDH A, (n)

### 76. 16-Bit Loads

LD rr, nn
LD SP, HL
LDHL SP, n
LD (nn), SP
PUSH rr
POP rr

### 80. 8-Bit ALU

ADD A, r
ADD A, (HL)
ADD A, n
ADC A, r
ADC A, (HL)
ADC A, n
SUB A, r
SUB A, (HL)
SUB A, n
SBC A, r
SBC A, (HL)
SBC A, n
AND A, r
AND A, (HL)
AND A, n
OR A, r
OR A, (HL)
OR A, n
XOR A, r
XOR A, (HL)
XOR A, n
CP A, r
CP A, (HL)
CP A, n
INC r
INC (HL)
DEC r
DEC (HL)

### 90. 16-Bit Arithmetic

ADD HL, rr
ADD SP, n
INC rr
DEC rr

### 94. Miscellaneous

SWAP r
SWAP (HL)
DAA
CPL
CCF
SCF
NOP
HALT
STOP
DI
EI

### 99. Rotates & Shifts

RLCA
RLA
RRCA
RRA
RLC r
RLC (HL)
RL r
RL (HL)
RRC r
RRC (HL)
RR r
RR (HL)
SLA r
SLA (HL)
SRA r
SRA (HL)
SRL r
SRL (HL)

### 108. Bit Opcodes

BIT n, r
BIT n, (HL)
SET n, r
SET n, (HL)
RES n, r
RES n, (HL)

### 111. Jumps

JP nn
JP cc, nn     # cc = NZ/Z/NC/C
JP (HL)
JR n
JR cc, n

### 114. Calls

CALL nn
CALL cc, nn

### 116. Restarts

RST n

### 117. Returns

RET
RET cc, nn
RETI
