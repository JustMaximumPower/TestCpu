
pub mod cpu {

	enum Instruction {
		Nop,
		ShortJump(i32),
		LongJump(u32),
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
				obj.storeu8(program[n], n as u32);
			}
			
			return obj;
		}
		
		pub fn step(&mut self) {
			let inst = match self.decode() {
				Err(why) => panic!(why),
				Ok(inst) => inst
			};
			
			match inst {
				Instruction::Nop => {self.pc += 1},
				Instruction::ShortJump(index) => {self.pc = (self.pc as i32 + index) as u32}
				Instruction::LongJump(address) => {self.pc = address}
				
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
		
		fn decode(&self) -> Result<Instruction, DecodingError> {
			let fist_byte =  match self.fetchu8(self.pc) {
				Err(way) => return Err(DecodingError::ShortRead),
				Ok(data) => data
			};
			
			match fist_byte {
				0 => Ok(Instruction::Nop),
				1 => {
					match self.fetchu32(self.pc + 1) {
						Err(why) => Err(DecodingError::ShortRead),
						Ok(result) => Ok(Instruction::ShortJump(result as i32))
					}
				},
				2 => {
					match self.fetchu16(self.pc + 1) {
						Err(why) => Err(DecodingError::ShortRead),
						Ok(result) => Ok(Instruction::ShortJump(result as i32))
					}
				},
				3 => {
					match self.fetchu32(self.pc + 1) {
						Err(why) => Err(DecodingError::ShortRead),
						Ok(result) => Ok(Instruction::LongJump(result))
					}
				},
				_ => Err(DecodingError::NotAnInstruction),
			}
		}
	}
}
