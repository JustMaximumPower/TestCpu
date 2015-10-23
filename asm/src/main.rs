#![feature(plugin)]
#![plugin(peg_syntax_ext)]

extern crate regex;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::collections::HashMap;
use regex::Regex;

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
	back_rev: Vec<BackRev>,
	is_second_pass: bool
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
			back_rev: Vec::new(),
			is_second_pass: false
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
					self.push_instruction(ins, args);
				}
			}
		}
	}
	
	pub fn second_pass(&mut self) {
		self.is_second_pass = true;
		for rev in self.back_rev.clone() {
			match rev {
				BackRev::Absolute(label, target_address) => {
					self.push_address(&Argument::Ident(label), target_address);
				},
				BackRev::Relative(_, _) =>{ panic!("not implemeted") },
				BackRev::Relative16(_, _) => { panic!("not implemeted") }
			}
		}
	}
	
	
	fn push_instruction(&mut self, ins: String, args: Vec<Argument>) {
		println!("Instruction {} ", ins);
		match ins.as_ref() {
			"nop" => {
				self.program.push(0x0u8);
				if !args.is_empty() {
					panic!("nop expects no arguments");	
				}
			},
			
			"jmp" => {
				self.program.push(0x03u8);
				if args.len() != 1 {
					panic!("jmp expects 1 argument");	
				}
				let target = self.program.len() as u32;
				self.push_address(&args[0], target);
			},
			
			"store" | "load" => {
				self.program.push(if ins == "store" { 0x0Au8 } else { 0x0Bu8 });
				if args.len() < 2 || args.len() > 3 {
					panic!("{} expects 2 or 3 argument", ins);	
				}
				
				let word_size = if args.len() == 2 { 4 } else {
					match args[0].clone() {
						Argument::Ident(_) => {
							panic!("Wordsize needs to be a number");
						},
						Argument::Number(num) => {
							Prog::parse_number(&num)
						}
					}
				};
				
				let reg = Prog::get_register_code(args[args.len() - 2].clone(), true);
				
				let argument = match word_size {
						1 => { 0 },
						2 => { 1 << 6 },
						4 => { 2 << 6 },
						_ => { panic!("Illegal word size {}", word_size); }
				} | reg;
				
				self.program.push(argument);
				let target = self.program.len() as u32;
				self.push_address(&args[args.len() - 1], target);
			},
			
			"mv" => {
				if args.len() != 3 {
					panic!("{} expects 3 argument", ins);	
				}
				self.program.push(0x0C);
				
				let size = match args[0].clone() {
					Argument::Ident(_) => {
						panic!("Size needs to be a number");
					},
					Argument::Number(num) => {
						Prog::parse_number(&num) as u8
					}
				};
				
				self.program.push(size);
				
				let target = self.program.len() as u32;
				self.push_address(&args[1], target);
				let target = self.program.len() as u32;
				self.push_address(&args[2], target);
			},
			
			_ => {
				panic!("unkown instruction {}", ins);
			}
		}
	}
	
	fn push_address(&mut self, arg: &Argument, target_address: u32) {
		match arg.clone() {
			Argument::Ident(x) => {
				if self.labels.contains_key(&x) {
					let address = self.labels.get(&x).unwrap().clone();
					self.push_value32(address as u32, target_address);
				}
				else if !self.is_second_pass
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
	
	fn parse_number(str_number: &str) -> u64 {
		if str_number.find("0x").is_some() {
			return u64::from_str_radix(&str_number[2..], 16).unwrap();
		} else {
			return str_number.parse().unwrap();
		}
	}
	
	fn get_register_code(reg: Argument, allow_spectial: bool) -> u8 {
		let str_reg = match reg {
			Argument::Ident(x) => { x },
			Argument::Number(_) => {
				panic!("Register needs to be a ident");
			}
		};
		
		let re = Regex::new(r"r(\d+)").unwrap();
		match re.captures(&str_reg) {
			Some(caps) => {
				Prog::parse_number(caps.at(1).unwrap()) as u8
			},
			None => {
				if allow_spectial {
					match str_reg.as_ref() {
						"pc" => { 0x20 },
						_ => { 
							panic!("unknown spectial registe {}", str_reg);
						}
					}
				}
				else
				{
					panic!("can't use spectial register");
				}
			}
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

