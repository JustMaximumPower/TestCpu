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
}

pub struct Prog {
	statments: Vec<Statemant>
}

impl Prog {
	pub fn new(statments: Vec<Statemant>) -> Prog {
		Prog{statments: statments}
	}
}

//peg_file! gramma("gramma.rustpeg");
peg! gramma(r#"

#[pub]
programm -> super::Prog
	= s:state ** ("\s"*) { super::Prog::new(s) }

state -> super::Statemant
	= comment { super::Statemant::Comment }
	/ l:lable { super::Statemant::Lable(l) }
	
comment -> ()
	= "/" "/" [^\n]* "\n"
	
lable -> String
	= i:Ident ":" { i }

Ident -> String
	= [a-zA-Z_][a-zA-Z_0-9]* { match_str.to_string() }

Sep -> ()
	= "\s"+
	
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
	
	match programm(&data) {
		Ok(p) => {}
		Err(why) => panic!(why)
	}
}


