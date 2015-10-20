
pub mod cpu {
	
	///TODO: find out how to match u8 as enum
//	enum InstructionFirstByte {
//		Nop = 0,
//		ShortJump = 1,
//		LongJump = 2,
//	}


	enum ArithmeticMode {
		Add,
		Sub,
		Mul,
		Div,
		Mod,
		Or,
		And,
		Xor
	}


	enum Instruction {
		Nop,
		ShortJump(i32),
		LongJump(u32),
		//(wordsize, Register, Address)
		Store(u8, u8, u32),
		//(wordsize, Register, Address)
		Load(u8, u8, u32),
		//(wordsize, SrcAddress, DestAddress)
		Move(u8, u32, u32),
		
		Copy(u8, u8),
		
		Arithmetic(ArithmeticMode, u8, u8, u8)
	}
	
	enum Error {
		NotAnInstruction,
		IllegalRegister(u8),
		AccessError(u32),
		DivedByZero,
	}
	
	struct RamPage {
		data: [u8; 65536],
		base: u32,
		flags: u32
	}
	
	
	pub struct Cpu {
		pc: u32,
		gp_regs: [u32; 32],
		mode: u32,
		ram: Vec<RamPage>
	}
	
	
	impl Cpu {
		
		pub fn new(pages: usize, program: &Vec<u8>) -> Cpu {
			let mut obj = Cpu{pc: 0, gp_regs: [0u32; 32], 
				mode: 0u32, ram: Vec::new()
			};
			
			for n in 0 .. pages {
				obj.ram.push(RamPage {
					data: [0u8; 65536],
					flags: 0u32,
					base: (n as u32) << 16
				});
			}
			
			for n in 0 .. program.len() {
				if obj.storeu8(program[n], n as u32).is_err() {
					panic!("store failed {}", n);
				}
			}
			
			return obj;
		}
		
		pub fn step(&mut self) {
			
			let result = self.step_impl();
			
			if result.is_err() {
				match result.err().unwrap() {

					Error::NotAnInstruction => {
						panic!("Not an instruction at 0x{:X}", self.pc);
					}
					
					Error::IllegalRegister(reg) => {
						panic!("Illegal register r{:}", reg);
					}
					
					Error::AccessError(address) => {
						panic!("Access error at 0x{:X}", address);
					}
					
					Error::DivedByZero => {
						panic!("divied by zero 0x{:X}", self.pc);
					}
				}
			}
		}
		
		fn step_impl(&mut self) -> Result<(), Error> {
			let (read, inst) = match self.decode() {
				Err(_) => panic!("decoding faild"),
				Ok(inst) => inst
			};
			
			self.pc += read as u32;
			
			match inst {
				Instruction::Nop => {}
				
				Instruction::ShortJump(index) => {
					println!("Relative Jump {}", index);
					self.pc = (self.pc as i32 + index) as u32
				}
				
				Instruction::LongJump(address) => {
					println!("Absolute Jump 0x{:X}", address);
					self.pc = address
				}
				
				Instruction::Store(wordsize, register, address) => {
					println!("Store {} bytes to r{} at 0x{:X}", wordsize, register, address);
					
					let reg = self.gp_regs[register as usize];
					for n in 0 .. wordsize {
						let value = (reg >> (8 * (wordsize - n))) as u8;
						if self.storeu8(value, address + n as u32).is_err() {
							panic!("store failed {}", address);
						}
					}
				}
				
				Instruction::Load(wordsize, register, address) => {
					println!("Load {} bytes to r{} from 0x{:X}", wordsize, register, address);
					
					let mut reg = 0u32;					
					for n in 0 .. wordsize {
						let value = match self.fetchu8(address + n as u32) {
							Err(_) => panic!("fatch failed at {}", address),
							Ok(value) => value
						} as u32;
						reg = (value << (8 * (wordsize - n - 1))) | reg;
					}
					self.gp_regs[register as usize] = reg;
				}
				
				Instruction::Move(wordsize, src_address, dest_address) => {
					println!("Move {} bytes from 0x{:X} to 0x{:X}", wordsize, src_address, dest_address);
					for n in 0 .. wordsize {
						let value = match self.fetchu8(src_address + n as u32) {
							Err(_) => panic!("fatch failed at {}", src_address + n as u32),
							Ok(value) => value
						};
						if self.storeu8(value, dest_address + n as u32).is_err() {
							panic!("store failed {}", dest_address + n as u32);
						}
					}
				}
				 
				Instruction::Copy(source_reg, dest_reg) => {
					println!("Copy from r{} to r{}", source_reg, dest_reg);
					
					let value = try!(self.fetch_reg(source_reg));
					try!(self.save_reg(dest_reg, value));
				}
				
				Instruction::Arithmetic(mode, target, src_a, src_b) => {
					let value_a = try!(self.fetch_reg(src_a));
					let value_b = try!(self.fetch_reg(src_b));
					
					let result = match mode {
						ArithmeticMode::Add => { value_a + value_b }
						ArithmeticMode::Sub => { value_a - value_b }
						ArithmeticMode::Mul => { value_a * value_b }
						ArithmeticMode::Div => {
							if value_b == 0 {
								return Err(Error::DivedByZero);
							}
							value_a / value_b 
						}
						ArithmeticMode::Mod => {
							if value_b == 0 {
								return Err(Error::DivedByZero);
							}
							value_a % value_b 
						}
						ArithmeticMode::Or  => { value_a | value_b }
						ArithmeticMode::And => { value_a & value_b }
						ArithmeticMode::Xor => { value_a ^ value_b }
					};
					
					try!(self.save_reg(target, result));
				}
			}
			
			Ok(())
		}
		
		
		fn save_reg(&mut self, reg: u8, value: u32) -> Result<(), Error> {
			match reg {
				0 ... 31 => self.gp_regs[reg as usize] = value,
				0x20 => self.pc = value,
				_ => { return Err(Error::IllegalRegister(reg));}
			}
			Ok(())
		}
		
		
		fn fetch_reg(&self, reg: u8) -> Result<u32, Error> {
			match reg {
				0 ... 31 => Ok(self.gp_regs[reg as usize]),
				0x20 => Ok(self.pc),
				_ => { return Err(Error::IllegalRegister(reg));}
			}
		}
		
		
		fn storeu8(&mut self, value: u8, address: u32) -> Result<(), Error> {
			let base = address >> 16;
			let offset = address & 0xffff;
			if self.ram.len() as u32 <= base {
				return Err(Error::AccessError(address));
			}
			self.ram[base as usize].data[offset as usize] = value;
			Ok(())
		}
		
		
		fn fetchu8(&self, address: u32) -> Result<u8, Error> {
			let base = address >> 16;
			let offset = address & 0xffff;
			if self.ram.len() as u32 <= base {
				return Err(Error::AccessError(address));
			}
			
			Ok(self.ram[base as usize].data[offset as usize])
		}
		
		
		fn fetchu16(&self, address: u32) -> Result<u16, Error> {
			let hiby = try!(self.fetchu8(address)) as u16;
			let lowby = try!(self.fetchu8(address + 1)) as u16;
			
			Ok((hiby << 8) + lowby)
		}
		
		
		fn fetchu32(&self, address: u32) -> Result<u32, Error> {
			let hiby = try!(self.fetchu16(address)) as u32;
			let lowby = try!(self.fetchu16(address + 2)) as u32;
			
			Ok((hiby << 16) + lowby)
		}
		
		
		fn decode(&self) -> Result<(u8, Instruction), Error> {
			let fist_byte = try!(self.fetchu8(self.pc));
			
			match fist_byte {
				0x0 => Ok((1, Instruction::Nop)),
				0x1 => {
					let value = try!(self.fetchu32(self.pc + 1));
					Ok((5, Instruction::ShortJump(value as i32)))
				},
				0x2 => {
					let value = try!(self.fetchu16(self.pc + 1));
					Ok((3, Instruction::ShortJump(value as i32)))
				},
				0x3 => {
					let value = try!(self.fetchu32(self.pc + 1));
					Ok((5, Instruction::LongJump(value)))
				},
				0xA | 0xB => {
					let value = try!(self.fetchu8(self.pc + 1));
					let address = try!(self.fetchu32(self.pc + 2));
					let reg = value & 0x1f;
					let wordsize = 1 << ((value & 0xC0) >> 6);
					
					if fist_byte == 0xA {
						Ok((6, Instruction::Store(wordsize, reg, address)))
					} else {
						Ok((6, Instruction::Load(wordsize, reg, address)))
					}
				},
				0xC => {
					let length = try!(self.fetchu8(self.pc + 1));
					let src_address = try!(self.fetchu32(self.pc + 2));
					let dest_address = try!(self.fetchu32(self.pc + 6));
					
					Ok((10, Instruction::Move(length, src_address, dest_address)))
				},
				
				0xD => {
					let source_reg = try!(self.fetchu8(self.pc + 1));
					let dest_reg = try!(self.fetchu8(self.pc + 1));
					
					Ok((3, Instruction::Copy(source_reg, dest_reg)))
				},
				
				code @ 0x10 ... 0x17 => {					
					let value = try!(self.fetchu16(self.pc + 1));
					
					let mode = match code {
						0x10 => ArithmeticMode::Add,
						0x11 => ArithmeticMode::Sub,
						0x12 =>	ArithmeticMode::Mul,
						0x13 =>	ArithmeticMode::Div,
						0x14 =>	ArithmeticMode::Mod,
						0x15 =>	ArithmeticMode::Or,
						0x16 =>	ArithmeticMode::And,
						0x17 => ArithmeticMode::Xor, 
						_ => unreachable!() //HACK: 
					};
					let target = ((value & 0b0111110000000000) >> 10) as u8;
					let src_a  = ((value & 0b0000001111100000) >> 5) as u8;
					let src_b  = (value & 0b0000000000011111) as u8;
					
					Ok((3, Instruction::Arithmetic(mode, target, src_a, src_b)))
				},
				
				_ => Err(Error::NotAnInstruction)
			}
		}
	}
}
