# TestCpu
This is a pet project to implement a cpu emulator in rust.

## Instructions
|  | First Byte |   |
|-----|------|-----|
| Nop | 0x00 | - |
| Relative Jump | 0x01 | Adress32 |
| Short Jump | 0x02 | Adress16 |
| Long Jump | 0x03 | Adress32 |
| Store | 0x0A | 2bit Wordsize, 1Bit unused, 5Bit Register Index; Adress32 |
| Load | 0x0B | 2bit Wordsize, 1Bit unused, 5Bit Register Index; Adress32 |
| Move | 0x0C | 1Byte lenght; Adress32 source; Adress32 destination |
