#![feature(plugin)]
#![plugin(peg_syntax_ext)]


use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::collections::HashMap;

use gramma::programm;

#[derive(Clone)]
pub enum Statement {
	Label(String), 
	Instruction(String, Vec<Argument>),
	Data(String)
}

#[derive(Clone)]
pub enum Argument {
	Ident(String),
	Number(String)
}

#[derive(Clone)]
enum BackRev {
	Absolute(String, u32),
	Relative(String, u32),
	Relative16(String, u32)
}


pub struct Prog {
	statments: Vec<Statement>,
	labels: HashMap<String, usize>,
	program: Vec<u8>,
	back_rev: Vec<BackRev>
}

impl Prog {
	//constuctor
	pub fn new(statments: Vec<Option<Statement>>) -> Prog {
		let mut tmp: Vec<Statement> = Vec::new();
		
		for i in statments {
			match i {
				Some(v) => tmp.push(v),
				None => {}
			}
		}
		
		Prog { 
			statments: tmp, 
			labels: HashMap::new(), 
			program: Vec::new(), 
			back_rev: Vec::new()
		}
	}
	
	// first pass of compile 
	pub fn first_pass(&mut self) {
		for statment in self.statments.clone() {
			match statment {
				Statement::Label(l) => {
					if self.labels.contains_key(&l) {
						panic!("Duplicate lable {}", l);
					}
					
					let pos = self.program.len();
					self.labels.insert(l.clone(), pos);
					println!("Label {} at 0x{:X} ", l, pos)
				},
				
				Statement::Data(d) => {
					let value = Prog::parse_number(&d) as u8;
					let target = self.program.len() as u32;
					self.push_value8(value, target);
					println!("Data {} as 0x{:X}", d, target);
				},
				
				Statement::Instruction(ins, args) => {
					println!("Instruction {} ", ins);
					match ins.as_ref() {
						"nop" => {
							self.program.push(0x0u8);
							if !args.is_empty() {
								panic!("nop expects no arguments");	
							}
						},
						
						"jmp" => {
							self.program.push(0x3u8);
							if args.len() != 1 {
								panic!("jmp expects 1 argument");	
							}
							let target = self.program.len() as u32;
							self.push_address(&args[0], target, true);
						},
						
						_ => {
							panic!("unkown instruction {}", ins);
						}
					}
				}
			}
		}
	}
	
	pub fn second_pass(&mut self) {
		for rev in self.back_rev.clone() {
			match rev {
				BackRev::Absolute(label, target_address) => {
					self.push_address(&Argument::Ident(label), target_address, false);
				},
				BackRev::Relative(label, target_address) =>{ panic!("not implemeted") },
				BackRev::Relative16(label, target_address) => { panic!("not implemeted") }
			}
		}
	}
	
	fn push_address(&mut self, arg: &Argument, target_address: u32, may_fail: bool) {
		match arg.clone() {
			Argument::Ident(x) => {
				if self.labels.contains_key(&x) {
					let address = self.labels.get(&x).unwrap().clone();
					self.push_value32(address as u32, target_address);
				}
				else if may_fail
				{
					let pos = self.program.len() as u32;
					self.push_value32(0u32, target_address);
					let rev = BackRev::Absolute(x, pos);
					self.back_rev.push(rev);
				}
				else
				{
					panic!("Missing lable: {}", x);
				}
			},
			Argument::Number(x) => {
				self.push_value32(Prog::parse_number(&x) as u32, target_address);
			}
		}
	}
	
	fn parse_number(str_number: &String) -> u64 {
		if str_number.find("0x").is_some() {
			return u64::from_str_radix(&str_number[2..], 16).unwrap();
		} else {
			return str_number.parse().unwrap();
		}
	}
	
	fn push_value8(&mut self, value : u8, target_address : u32) {
		while (self.program.len() as u32) <= target_address {
			self.program.push(0);
		}
		self.program[target_address as usize] = value;
	}
	
	fn push_value16(&mut self, value : u16, offset : u32) {
		self.push_value8((value >> 8) as u8, offset);
		self.push_value8(value as u8, offset + 1);
	}
	
	fn push_value32(&mut self, value : u32, offset : u32) {
		self.push_value16((value >> 16) as u16, offset);
		self.push_value16(value as u16, offset + 2);
	}
}



peg_file! gramma("gramma.rustpeg");



fn main() {
	let input = match env::args().nth(1) {
		None => panic!("no Input"),
		Some(file) => file
	};
	
	let output = match env::args().nth(2) {
		None => panic!("no Outout"),
		Some(file) => file
	};
	
	
	let mut file = match File::open(&Path::new(&input)) {
		Err(why) => panic!(why),
		Ok(file) => file,
	};
	
	let mut data = String::new();
	
	file.read_to_string(&mut data).unwrap();
	
	let mut ast = match programm(&data) {
		Ok(ast) => { ast },
		Err(why) => panic!("{}", why)
	};
	
	ast.first_pass();
	
	ast.second_pass();
	
	
	file = match File::create(&Path::new(&output)) {
		Err(why) => panic!(why),
		Ok(file) => file,
	};
	
	match file.write_all(&ast.program) {
		Err(why) => panic!(why),
		Ok(()) => {}
	}
}

