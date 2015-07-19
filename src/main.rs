use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;


enum Instruction {
	Nop,
	ShortJump(i32),
	LongJump(u32),
}

enum DecodingError {
	ShortRead,
	NotAnInstruction
}

fn btoi32(buffer: &[u8]) -> i32 {
	(((buffer[0] as u32) << 24) + ((buffer[1] as u32) << 16) + ((buffer[2] as u32) << 8) + buffer[3] as u32) as i32
}

fn btoi16(buffer: &[u8]) -> i16 {
	(((buffer[0] as u32) << 8) + buffer[1] as u32) as i16
}


fn decode(buffer: &[u8]) -> Result<Instruction, DecodingError> {
	if buffer.len() < 1 {
		return Err(DecodingError::ShortRead)
	}
	
	match buffer[0] {
		0 => Ok(Instruction::Nop),
		1 => {
			if buffer.len() < 5 {
				return Err(DecodingError::ShortRead);
			} else {
				let result: i32 = btoi32(& buffer[1 .. 5]);
				return Ok(Instruction::ShortJump(result));
			}
		},
		2 => {
			if buffer.len() < 3 {
				return Err(DecodingError::ShortRead);
			} else {
				let result: i32 = btoi16(& buffer[1 .. 3]) as i32;
				return Ok(Instruction::ShortJump(result));
			}
		},
		3 => {
			if buffer.len() < 5 {
				return Err(DecodingError::ShortRead);
			} else {
				let result: u32 = btoi32(& buffer[1 .. 5]) as u32;
				return Ok(Instruction::LongJump(result));
			}
		},
		_ => Err(DecodingError::NotAnInstruction),
	}
}


struct RamPage {
	data: [u8; 65536],
	base: u32,
	flags: u32
}

struct Cpu {
	gp_regs: [u32; 32],
	mode: u32
}


fn main() {
	
	let input = match env::args().nth(1) {
		None => panic!("no Input"),
		
		Some(file) => file
	};
	
	let path = Path::new(&input);
	
	
	let mut file = match File::open(&path) {
		Err(why) => panic!(why),
		Ok(file) => file,
	};
	
	let mut data : Vec<u8> = Vec::new();
	
	file.read_to_end(&mut data).unwrap();
	
	let mut memory : [u8; 65536] = [0; 65536];
	
	for n in 0 .. data.len() {
		memory[n] = data[n];
	}
	
	let mut pc : u32 = 0u32;
	
	loop {
		let inst = match decode(&memory[pc as usize .. 65536]) {
			Err(why) => panic!(why),
			Ok(inst) => inst
		};
		
		match inst {
			Instruction::Nop => {pc += 1},
			Instruction::ShortJump(index) => {pc = (pc as i32 + index) as u32}
			Instruction::LongJump(address) => {pc = address}
			
		}
	}
}
