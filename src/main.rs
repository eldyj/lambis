mod interpreter;
use std::{fs, env::args};

fn main() {
	let argv: &mut dyn Iterator<Item=String> = &mut args();
	let program: String = argv.next().unwrap();
	let Some(file) = argv.next() else {
		panic!("Usage: {} <file>", program)
	};

	interpreter::eval_start(
		fs::read_to_string(file.clone())
			.expect(format!("failed to open file {}", file).as_str()));
}
