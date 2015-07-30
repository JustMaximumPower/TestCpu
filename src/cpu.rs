
pub mod cpu {
	
	///TODO: find out how to match u8 as enum
//	enum InstructionFirstByte {
//		Nop = 0,
//		ShortJump = 1,
//		LongJump = 2,
//	}


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
	}
	
	
	enum DecodingError {
		ShortRead,
		NotAnInstruction
	}
	
	
	enum RamError {
		BusError
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
			let (read, inst) = match self.decode() {
				Err(_) => panic!("decoding faild"),
				Ok(inst) => inst
			};
			
			self.pc += read as u32;
			
			match inst {
				Instruction::Nop => {}
				
				Instruction::ShortJump(index) => {
					self.pc = (self.pc as i32 + index) as u32
				}
				
				Instruction::LongJump(address) => {
					self.pc = address
				}
				
				Instruction::Store(wordsize, register, address) => {
					let reg = self.gp_regs[register as usize];
					for n in 0 .. wordsize {
						let value = (reg >> (8 * (wordsize - n))) as u8;
						if self.storeu8(value, address + n as u32).is_err() {
							panic!("store failed {}", address);
						}
					}
				}
				
				Instruction::Load(wordsize, register, address) => {
					
					let mut reg = 0u32;					
					for n in 0 .. wordsize {
						
						let value = match self.fetchu8(address + n as u32) {
							Err(_) => panic!("fatch failed at {}", address),
							Ok(value) => value
						};
						
						reg = reg << (8 * (wordsize - n)) + value;
					}
					self.gp_regs[register as usize] = reg;
				}
				
				Instruction::Move(wordsize, src_address, dest_address) => {
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
			}
		}
		
		
		fn storeu8(&mut self, value: u8, address: u32) -> Result<(), RamError> {
			let base = address >> 16;
			let offset = address & 0xffff;
			if self.ram.len() as u32 <= base {
				return Err(RamError::BusError);
			}
			self.ram[base as usize].data[offset as usize] = value;
			Ok(())
		}
		
		
		fn fetchu8(&self, address: u32) -> Result<u8, RamError> {
			let base = address >> 16;
			let offset = address & 0xffff;
			if self.ram.len() as u32 <= base {
				return Err(RamError::BusError);
			}
			Ok(self.ram[base as usize].data[offset as usize])
		}
		
		
		fn fetchu16(&self, address: u32) -> Result<u16, RamError> {
			let hiby = try!(self.fetchu8(address)) as u16;
			let lowby = try!(self.fetchu8(address + 1)) as u16;
			Ok((hiby << 8) + lowby)
		}
		
		
		fn fetchu32(&self, address: u32) -> Result<u32, RamError> {
			let hiby = try!(self.fetchu16(address)) as u32;
			let lowby = try!(self.fetchu16(address + 2)) as u32;
			Ok((hiby << 16) + lowby)
		}
		
		
		fn decode(&self) -> Result<(u8, Instruction), DecodingError> {
			
			let fist_byte =  match self.fetchu8(self.pc) {
				Err(_) => return Err(DecodingError::ShortRead),
				Ok(data) => data
			};
			
			match fist_byte {
				0x0 => Ok((1, Instruction::Nop)),
				0x1 => {
					match self.fetchu32(self.pc + 1) {
						Err(_) => Err(DecodingError::ShortRead),
						Ok(result) => Ok((5, Instruction::ShortJump(result as i32)))
					}
				},
				0x2 => {
					match self.fetchu16(self.pc + 1) {
						Err(_) => Err(DecodingError::ShortRead),
						Ok(result) => Ok((3, Instruction::ShortJump(result as i32)))
					}
				},
				0x3 => {
					match self.fetchu32(self.pc + 1) {
						Err(_) => Err(DecodingError::ShortRead),
						Ok(result) => Ok((5, Instruction::LongJump(result)))
					}
				},
				0xA => {
					let value = match self.fetchu8(self.pc + 1) {
						Err(_) => return Err(DecodingError::ShortRead),
						Ok(result) => result
					};
					let address = match self.fetchu32(self.pc + 2) {
						Err(_) => return Err(DecodingError::ShortRead),
						Ok(result) => result
					};
					let reg = value & 0x1f;
					let wordsize = 2 << ((value & 0xC0) >> 6);
					
					Ok((6, Instruction::Store(wordsize, reg, address)))
				},
				0xB => {
					let value = match self.fetchu8(self.pc + 1) {
						Err(_) => return Err(DecodingError::ShortRead),
						Ok(result) => result
					};
					let address = match self.fetchu32(self.pc + 2) {
						Err(_) => return Err(DecodingError::ShortRead),
						Ok(result) => result
					};
					let reg = value & 0x1f;
					let wordsize = 2 << ((value & 0xC0) >> 6);
					
					Ok((6, Instruction::Load(wordsize, reg, address)))
				},
				
				_ => Err(DecodingError::NotAnInstruction),
			}
		}
	}
}
