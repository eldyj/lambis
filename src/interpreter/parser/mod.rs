pub mod lexer;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
	None,
	Variable(String),
	Integer(i128),
	Decimal(f64),
	Lambda(String, Box<ASTNode>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
	Addition,
	Substraction,
	Multiplication,
	Division,
	Exponent,

	LessEqual,
	Less,
	Greater,
	GreaterEqual,
	Equal,
	NotEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
	Nothing,
	Value(Value),
	Definition(String, Box<ASTNode>),
	LambdaCall(Box<Value>, Vec<ASTNode>),
	Call(String, Vec<ASTNode>),
	Switch(Box<ASTNode>, Vec<(ASTNode, ASTNode)>),
	RationalPart(Box<ASTNode>),
	IntegerPart(Box<ASTNode>),
	Print(Box<ASTNode>),
	Operation(Operation, Box<ASTNode>, Box<ASTNode>),
}

pub struct ParseableIter {
	tokens: Vec<lexer::Token>,
	current_index: usize,
}

pub type Parseable = ParseableIter;

impl ParseableIter {
	fn new(source: Vec<lexer::Token>)  -> Self {
		Self {
			tokens: source,
			current_index: 0,
		}
	}
}

// iter impl
impl Iterator for ParseableIter {
	type Item = lexer::Token;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(token) = self.tokens.get(self.current_index) {
			self.current_index += 1;
			Some(token.clone())
		} else {
			None
		}
	}
}

// peekable impl
impl Parseable {
	fn peek(&self) -> Option<lexer::Token> {
		if let Some(token) = self.tokens.get(self.current_index) {
			Some(token.clone())
		} else {
			None
		}
	}

	fn is_empty(&self) -> bool {
		self.peek() == None
	}
}

// parser impl
impl Parseable {
	fn consume(&mut self, expected: lexer::Token) -> () {
		let Some(token) = self.next() else {
			panic!("ParsingError: expected «{:?}», got end of input", expected)
		};

		if token != expected {
			panic!("ParsingError: expected «{:?}», got «{:?}»", expected, token)
		}
	}

	fn consume_ident(&mut self) -> String {
		let Some(token) = self.next() else {
			panic!("ParsingError: expected ident, got end of input")
		};

		if let lexer::Token::Ident(name) = token {
			name.clone()
		} else {
			panic!("ParsingError: expected ident, got «{:?}»", token)
		}
	}

	fn consume_integer(&mut self) -> i128 {
		let Some(token) = self.next() else {
			panic!("ParsingError: expected number, got end of line")
		};

		if let lexer::Token::Integer(integer) = token {
			integer
		} else {
			panic!("ParsingError: expected number, got «{:?}»", token)
		}
	}

	fn parse_switch(&mut self, compared: ASTNode) -> ASTNode {
		self.consume(lexer::Token::Dollar);
		self.consume(lexer::Token::OpenBrace);
		let mut cases: Vec<(ASTNode, ASTNode)> = vec![];

		while !self.is_empty() && !self.is_delimiter() {
			let case: ASTNode = self.parse_expression(true, false);
			self.consume(lexer::Token::Arrow);
			cases.push((case, self.parse_expression(true, true)));
		}

		self.consume(lexer::Token::CloseBrace);
		ASTNode::Switch(Box::new(compared), cases)
	}

	fn is_delimiter(&self) -> bool {
		let Some(current) = self.peek() else {
			return false;
		};

		match current {
			lexer::Token::CloseParen
			| lexer::Token::CloseBracket
			| lexer::Token::CloseBrace
			| lexer::Token::Dollar => true,
			_ => false,
		}
	}

	fn is_operation(&self) -> bool {
		let Some(current) = self.peek() else {
			return false;
		};

		match current {
			lexer::Token::Plus
			| lexer::Token::Minus
			| lexer::Token::Asterisk
			| lexer::Token::Slash
			| lexer::Token::Circumflex
			| lexer::Token::Equal
			| lexer::Token::NotEqual
			| lexer::Token::LessEqual
			| lexer::Token::Less
			| lexer::Token::GreaterEqual
			| lexer::Token::Greater => true,
			_ => false,
		}
	}

	fn parse_operation(&mut self, left: ASTNode) -> ASTNode {
		let Some(current) = self.next() else {
			unreachable!("what")
		};

		let (allow_operations, allow_repeat): (bool, bool) = match current {
			lexer::Token::Plus | lexer::Token::Minus => (true, true),
			lexer::Token::Asterisk | lexer::Token::Slash | lexer::Token::Circumflex => (false, true),
			_ => (false, false),
		};

		let operation: Operation = match current {
			lexer::Token::Plus => Operation::Addition,
			lexer::Token::Minus => Operation::Substraction,
			lexer::Token::Asterisk => Operation::Multiplication,
			lexer::Token::Slash => Operation::Division,
			lexer::Token::Circumflex => Operation::Exponent,
			lexer::Token::Equal => Operation::Equal,
			lexer::Token::NotEqual => Operation::NotEqual,
			lexer::Token::LessEqual => Operation::LessEqual,
			lexer::Token::Less => Operation::Less,
			lexer::Token::GreaterEqual => Operation::GreaterEqual,
			lexer::Token::Greater => Operation::Greater,
			_ => unreachable!("what2"),
		};

		let tmp: ASTNode = ASTNode::Operation(operation, Box::new(left), Box::new(self.parse_expression(true, allow_operations)));
		let res: ASTNode = if allow_repeat && self.is_operation() {
			self.parse_operation(tmp)
		} else {
			tmp
		};

		if self.peek() == Some(lexer::Token::Dollar) {
			self.parse_switch(res)
		} else {
			res
		}
	}

	fn parse_expression(&mut self, from_call: bool, allow_operations: bool) -> ASTNode {
		let Some(current) = self.peek() else {
			return ASTNode::Nothing;
		};

		match current {
			lexer::Token::Exclam => {
				self.consume(lexer::Token::Exclam);
				ASTNode::Print(Box::new(self.parse_expression(false, true)))
			}

			lexer::Token::OpenBracket => {
				self.consume(lexer::Token::OpenBracket);
				let result: ASTNode = self.parse_expression(false, true);
				self.consume(lexer::Token::CloseBracket);

				if allow_operations && self.is_operation() {
					self.parse_operation(result)
				} else {
					ASTNode::IntegerPart(Box::new(result))
				}
			}

			lexer::Token::OpenBrace => {
				self.consume(lexer::Token::OpenBrace);
				let result: ASTNode = ASTNode::RationalPart(Box::new(self.parse_expression(false, true)));
				self.consume(lexer::Token::CloseBrace);

				if allow_operations && self.is_operation() {
					self.parse_operation(result)
				} else {
					result
				}
			}

			lexer::Token::OpenParen => {
				self.consume(lexer::Token::OpenParen);
				let result: ASTNode = self.parse_expression(false, true);
				self.consume(lexer::Token::CloseParen);

				let res: ASTNode = if let ASTNode::Value(ref value) = result {
					if from_call {
						result
					} else if let Value::Lambda(_, _) = value {
						let mut args: Vec<ASTNode> = vec![];

						while !self.is_empty()
						&& !self.is_delimiter()
						&& !self.is_operation()
						&& self.peek() != Some(lexer::Token::Period){
							args.push(self.parse_expression(false, true));
						}

						if args.len() != 0 {
							ASTNode::LambdaCall(Box::new(value.clone()), args)
						} else {
							result
						}
					} else {
						result
					}
				} else {
					result
				};

				if let Some(token) = self.peek() {
					if allow_operations && self.is_operation() {
						self.parse_operation(res)
					} else if token == lexer::Token::Dollar {
						self.parse_switch(res)
					} else if token == lexer::Token::Period {
						self.consume(lexer::Token::Period);
						res
					} else {
						res
					}
				} else {
					res
				}
			}

			lexer::Token::Lambda => {
				self.consume(lexer::Token::Lambda);
				let mut variables: String = self.consume_ident();
				self.consume(lexer::Token::Period);
				let mut body: Box<ASTNode> = Box::new(self.parse_expression(false, true));

				if let ASTNode::Value(value) = *body.clone() {
					if let Value::Lambda(vars, content) = value {
						variables += vars.as_str();
						body = content;
					}
				}

				if self.peek() == Some(lexer::Token::Period) {
					self.consume(lexer::Token::Period);
				}

				ASTNode::Value(Value::Lambda(variables, body.clone()))
			}

			lexer::Token::Integer(_) => {
				let integer: i128 = self.consume_integer();

				let result: ASTNode = if self.peek() == Some(lexer::Token::Period) {
					self.consume(lexer::Token::Period);
					let rational: i128 = self.consume_integer();
					ASTNode::Value(Value::Decimal(format!("{}.{}", integer, rational).parse::<f64>().unwrap()))
				} else {
					ASTNode::Value(Value::Integer(integer))
				};

				if allow_operations && self.is_operation() {
					self.parse_operation(result)
				} else if !from_call && self.peek() == Some(lexer::Token::Dollar) {
					self.parse_switch(result)
				} else {
					result
				}
			}

			lexer::Token::Ident(_) => {
				let name: String = self.consume_ident();

				if from_call {
					let result: ASTNode = ASTNode::Value(Value::Variable(name));
					if allow_operations && self.is_operation() {
						self.parse_operation(result)
					} else if self.peek() == Some(lexer::Token::Dollar) {
						self.parse_switch(result)
					} else {
						result
					}
				} else if self.peek() == Some(lexer::Token::Equal) {
					self.consume(lexer::Token::Equal);
					ASTNode::Definition(name, Box::new(self.parse_expression(false, true)))
				} else {
					let mut args: Vec<ASTNode> = vec![];

					while !self.is_empty()
					&& !self.is_delimiter()
					&& !self.is_operation()
					&& self.peek() != Some(lexer::Token::Period) {
						args.push(self.parse_expression(true, true));
					}

					let result: ASTNode = if args.len() == 0 {
						ASTNode::Value(Value::Variable(name))
					} else {
						ASTNode::Call(name, args)
					};

					if let Some(token) = self.peek() {
						if allow_operations && self.is_operation() {
							self.parse_operation(result)
						} else if token == lexer::Token::Period {
							self.consume(lexer::Token::Period);
							result
						} else if !from_call && token == lexer::Token::Dollar {
							self.parse_switch(result)
						} else {
							if token == lexer::Token::Period {
								self.consume(lexer::Token::Period);
							}

							result
						}
					} else {
						result
					}
				}
			}

			what => {
				panic!("ParsingError: expected expression start, got «{:?}», previous token is {:?}", what, self.tokens[self.current_index-1])
			}
		}
	}

	pub fn parse(&mut self) -> Vec<ASTNode> {
		let mut result: Vec<ASTNode> = vec![];

		while !self.is_empty() {
			result.push(self.parse_expression(false, true));
		}

		result
	}
}

pub fn parse(source: String) -> Vec<ASTNode> {
	Parseable::new(lexer::lex(source)).parse()
}
