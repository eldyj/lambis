mod interpreter;

fn main() {
	interpreter::eval_start("
		int = λx. [x].
		floor = λx. x - {x}.

		factorial = λx.
			x $ {
				0 -> 1
				x -> (factorial x-1)*x
			}.

		! (factorial 10)".to_string()
	);
}
