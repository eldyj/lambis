mod interpreter;
use std::{fs, env::args};

fn main() {
	let argv: &mut dyn Iterator<Item=String> = &mut args();
	let program: String = argv.next().unwrap();
	let file: String = argv.next().unwrap_or_else(|| panic!("Usage: {program} <file>"));

	interpreter::eval_start(
		fs::read_to_string(file)
			.unwrap()
			.as_str()
	).unwrap();
}
