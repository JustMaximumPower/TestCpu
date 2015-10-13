use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;

mod cpu;


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
	
	
	let mut prozess = cpu::cpu::Cpu::new(1, &data);
	
	loop {
		prozess.step();
	}
}
