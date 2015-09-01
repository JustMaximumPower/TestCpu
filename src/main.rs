#![feature(plugin)]
#![plugin(peg_syntax_ext)]


use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use gramma::programm;

pub enum Statement {
	Label(String), 
	Instruction(String, Vec<String>),
	Data(String)
}

pub struct Prog {
	statments: Vec<Statement>
}

impl Prog {
	pub fn new(statments: Vec<Option<Statement>>) -> Prog {
		let mut tmp: Vec<Statement> = Vec::new();
		
		for i in statments {
			match i {
				Some(v) => tmp.push(v),
				None => {}
			}
		}
		
		Prog{ statments: tmp }
	}
}

peg_file! gramma("gramma.rustpeg");


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
	
	let mut data = String::new();
	
	file.read_to_string(&mut data).unwrap();
	
	println!("{}", data);
	
	let ast = match programm(&data) {
		Ok(ast) => { ast },
		Err(why) => panic!("{}", why)
	};
	
	for statment in ast.statments {
		match statment {
			Statement::Label(l) => println!("Label {}", l),
			Statement::Data(d) => println!("Data {}", d),
			Statement::Instruction(ins, args) => println!("Instruction {}  {}", ins, args.join(", ")),
			//_ => println!("Unkonwn")
		}
	}
}


