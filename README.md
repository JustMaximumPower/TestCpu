# TestCpu
This is a pet project to implement a cpu emulator in rust.

## Instructions
|  | First Byte |   |
|-----|------|-----|
| Nop | 0x00 | - |
| Relative Jump | 0x01 | Adress32 |
| Short Jump | 0x02 | Adress16 |
| Long Jump | 0x03 | Adress32 |
| Store | 0x0A | 2bit Wordsize, 1Bit special Reg, 5Bit Register Index; Adress32 |
| Load | 0x0B | 2bit Wordsize, 1Bit special Reg, 5Bit Register Index; Adress32 |
| Move | 0x0C | 1Byte lenght; Adress32 source; Adress32 destination |
| Copy | 0x0D |  2Bit unused, 1Bit special Reg, 5Bit Source Reg; 2Bit unused, 1Bit special Reg, 5Bit target Reg |
| Add | 0x10| 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
| Sub | 0x11 | 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
| Mul | 0x12 | 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
| Div | 0x13 | 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
| Mod | 0x14 | 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
| Or | 0x15 | 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
| And | 0x16 | 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
| Xor | 0x17 | 1Bit unused, 5Bit Target Reg, 5Bit A Reg, 5Bit B Reg |
