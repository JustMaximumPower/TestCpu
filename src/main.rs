#![feature(plugin)]
#![plugin(peg_syntax_ext)]


use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use gramma::programm;

pub enum Statemant {
	Comment,
	Lable(String), 
	Instruction(String, Vec<String>)
}

pub struct Prog {
	statments: Vec<Statemant>
}

impl Prog {
	pub fn new(statments: Vec<Option<Statemant>>) -> Prog {
		let mut tmp: Vec<Statemant> = Vec::new();
		
		for i in statments {
			match i {
				Some(v) => tmp.push(v),
				None => {}
			}
		}
		
		Prog{ statments: tmp }
	}
}

//peg_file! gramma("gramma.rustpeg");
peg! gramma(r#"

#[pub]
programm -> super::Prog
	= s:Statement* { super::Prog::new(s) }
 
Statement -> Option<super::Statemant>
	= l:Lable { Some(super::Statemant::Lable(l)) }
	/ i:Instruction { Some(i) }
	/ Comment { Some(super::Statemant::Comment) }
	/ WhiteSpace { None }
	
Comment -> ()
	= "/" "/" .*
	
Lable -> String
	= i:Ident ":" { i }
	
Instruction -> super::Statemant
	= i:Ident arg:Ident ** Seperator { super::Statemant::Instruction(i, arg) }

Ident -> String
	= [a-zA-Z_][a-zA-Z_0-9]* { match_str.to_string() }

WhiteSpace -> ()
	= "\s"+
	
Seperator -> ()
	= "\s"* "," "\s"*
"#);

 
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
	
	//match programm(&data) {
	match programm("R:") {
		Ok(p) => {},
		Err(why) => panic!(why)
	}
}


