mod interpreter;
use std::{fs, env::args};

fn main() {
	let argv: &mut dyn Iterator<Item=String> = &mut args();
	let program: String = argv.next().unwrap();
	let Some(file) = argv.next() else {
		panic!("Usage: {} <file>", program)
	};

	if let Err(x) = interpreter::eval_start(
		fs::read_to_string(
			file.clone())
				.unwrap_or_else(|_| panic!("failed to open file {}", file))
	) {
		panic!("{}", x);
	};
}
