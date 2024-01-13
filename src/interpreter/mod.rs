pub mod parser;
use std::collections::HashMap;

fn integer_operation(op: parser::Operation, left: i128, right: i128) -> parser::Value {
	match op {
		parser::Operation::Addition => parser::Value::Integer(left + right),
		parser::Operation::Substraction => parser::Value::Integer(left - right),
		parser::Operation::Multiplication => parser::Value::Integer(left * right),
		parser::Operation::Division => {
			if left % right == 0 {
				parser::Value::Integer(left / right)
			} else {
				parser::Value::Decimal((left as f64) / (right as f64))
			}
		}

		parser::Operation::Exponent => parser::Value::Decimal((left as f64).powi(right as i32)),

		parser::Operation::Less => parser::Value::Integer((left < right) as i128),
		parser::Operation::LessEqual => parser::Value::Integer((left <= right) as i128),
		parser::Operation::Greater => parser::Value::Integer((left > right) as i128),
		parser::Operation::GreaterEqual => parser::Value::Integer((left >= right) as i128),
		parser::Operation::Equal => parser::Value::Integer((left == right) as i128),
		parser::Operation::NotEqual => parser::Value::Integer((left != right) as i128),
		//_ => panic!("InterpretationError: Unsupported operation for integers"),
	}
}

fn decimal_operation(op: parser::Operation, left: f64, right: f64) -> parser::Value {
	parser::Value::Decimal(match op {
		parser::Operation::Addition => left + right,
		parser::Operation::Substraction => left - right,
		parser::Operation::Multiplication => left * right,
		parser::Operation::Division => left / right,
		parser::Operation::Exponent => left.powf(right),

		parser::Operation::Less => (left < right) as i128 as f64,
		parser::Operation::LessEqual => (left <= right) as i128 as f64,
		parser::Operation::Greater => (left > right) as i128 as f64,
		parser::Operation::GreaterEqual => (left >= right) as i128 as f64,
		parser::Operation::Equal => (left == right) as i128 as f64,
		parser::Operation::NotEqual => (left != right) as i128 as f64,
		//_ => panic!("InterpretationError: Unsupported operation for decimals"),
	})
}

fn operation(op: parser::Operation, left: parser::Value, right: parser::Value) -> parser::Value {
	match (left, right) {
		(parser::Value::Integer(n1), parser::Value::Integer(n2)) => {
			integer_operation(op, n1, n2)
		}
		(parser::Value::Integer(n1), parser::Value::Decimal(n2)) => {
			decimal_operation(op, n1 as f64, n2)
		}
		(parser::Value::Decimal(n1), parser::Value::Integer(n2)) => {
			decimal_operation(op, n1, n2 as f64)
		}
		(parser::Value::Decimal(n1), parser::Value::Decimal(n2)) => {
			decimal_operation(op, n1, n2)
		}
		_ => panic!("InterpretationError: Unsupported value types"),
	}
}

pub fn eval(node: parser::ASTNode, variables: &mut HashMap<String, parser::Value>, args: &mut HashMap<char, parser::Value>) -> parser::Value {
	match node {
		parser::ASTNode::Nothing => parser::Value::None,

		parser::ASTNode::Print(value_) => {
			let value: parser::Value = eval(*value_, variables, args);
			match value {
				parser::Value::Lambda(args_def, _) => println!("<λ{}.>", args_def),
				parser::Value::Integer(int) => println!("{}", int),
				parser::Value::Decimal(dec) => println!("{}", dec),
				parser::Value::None => println!("Nothing"),
				what => println!("{:?}", what),
			}

			parser::Value::None
		}

		parser::ASTNode::RationalPart(value_) => {
			let value: parser::Value = eval(*value_, variables, args);
			match value {
				parser::Value::Integer(_) => parser::Value::Integer(0),
				parser::Value::Decimal(n) =>parser::Value::Decimal(n-n.floor()),
				what => panic!("InterpreterError: {{_}}: expected <Integer|Decimal>, got {:?}", what),
			}
		}

		parser::ASTNode::IntegerPart(value_) => {
			let value: parser::Value = eval(*value_, variables, args);
			match value {
				parser::Value::Integer(_) => value,
				parser::Value::Decimal(n) => parser::Value::Integer(n.floor() as i128),
				what => panic!("InterpreterError: [_]: Expected <Integer|Decimal>, got {:?}", what),
			}
		}

		parser::ASTNode::Switch(compared_, cases) => {
			let compared: parser::Value = eval(*compared_, variables, args);
			for (case_, action) in cases {
				let case: parser::Value = eval(case_, variables, args);
				if compared == case {
					return eval(action, variables, args);
				}
			}

			parser::Value::None
		}

		parser::ASTNode::Definition(name, content) => {
			let res: parser::Value = eval(*content, variables, args);
			variables.insert(name, res.clone());
			res
		}

		parser::ASTNode::Value(val) => {
			match val {
				parser::Value::Variable(name) => {
					let first: char = name.chars().nth(0).unwrap();

					if variables.contains_key(&name) {
						variables[&name].clone()
					} else if name.len() == 1 && args.contains_key(&first) {
						args[&first].clone()
					} else {
						panic!("InterpreterError: variable «{}» is undefined in current context", name)
					}
				}

				_ => val,
			}
		}

		parser::ASTNode::LambdaCall(lambda, _args) => {
			let parser::Value::Lambda(args_def, content) = *lambda.clone() else {
				unreachable!("how the fuck you managed to get non-lambda in LambdaCall???")
			};

			let mut new_args: HashMap<char, parser::Value> = args.clone();
			let len: usize = _args.len();
			let len2: usize = args_def.len();
			if len > len2 {
				panic!("InterpreterError: too much argumeths for «{:?}»", lambda.clone())
			}

			if len < len2 {
				parser::Value::Lambda(
					args_def[len..].to_string(),
					Box::new(parser::ASTNode::LambdaCall(
						Box::new(parser::Value::Lambda(
							args_def[..len].to_string(),
							content,
						)),
						_args.clone()
					))
				)
			} else {
				for (index, i) in _args.into_iter().enumerate() {
					if let Some(ch) = args_def.chars().nth(index) {
						if new_args.contains_key(&ch) {
							new_args.remove(&ch);
						}

						let _ = new_args.insert(ch, eval(i, variables, args));
					}
				}

				eval(*content.clone(), variables, &mut new_args)
			}
		}

		parser::ASTNode::Call(name, args_) => {
			let var_content: parser::Value = eval(parser::ASTNode::Value(parser::Value::Variable(name.clone())), variables, args);
			if let parser::Value::Lambda(_, _) = var_content {
				eval(parser::ASTNode::LambdaCall(Box::new(var_content), args_), variables, args)
			} else {
				panic!("InterpreterError: trying to call value «{:?}, {:?}»", var_content, name.clone())
			}
		}

		parser::ASTNode::Operation(op, left_, right_) => {
			let left: parser::Value = eval(*left_, variables, args);
			let right: parser::Value = eval(*right_, variables, args);
			operation(op, left, right)
		}
	}
}

pub fn eval_start(s: String) -> () {
	let p: Vec<parser::ASTNode> = parser::parse(s);
	let mut variables: HashMap<String, parser::Value> = HashMap::new();
	let mut arguments: HashMap<char, parser::Value> = HashMap::new();

	variables.insert("true".to_string(), parser::Value::Integer(1));
	variables.insert("false".to_string(), parser::Value::Integer(0));

	//println!("{:?}", p);

	for i in p {
		eval(i, &mut variables, &mut arguments);
	}
}
