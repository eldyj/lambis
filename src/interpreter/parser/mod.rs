pub mod lexer;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
	None,
	Variable(String),
	Integer(i128),
	Decimal(f64),
	Lambda {
		args_def: String,
		content: Box<ASTNode>
	},
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
	Definition {
		name: String,
		value: Box<ASTNode>
	},

	LambdaCall {
		lambda: Box<Value>,
		args: Vec<ASTNode>
	},

	Call {
		name: String,
		args: Vec<ASTNode>
	},

	Switch {
		compared: Box<ASTNode>,
		cases: Vec<(ASTNode, ASTNode)>
	},

	RationalPart(Box<ASTNode>),
	IntegerPart(Box<ASTNode>),
	Print(Box<ASTNode>),

	Operation {
		left: Box<ASTNode>,
		operation: Operation,
		right: Box<ASTNode>
	},
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
		self.tokens.get(self.current_index).map(|token| {
			self.current_index += 1;
			token.clone()
		})
	}
}

// peekable impl
impl Parseable {
	fn peek(&self) -> Option<lexer::Token> {
		self.tokens.get(self.current_index).cloned()
	}

	fn is_empty(&self) -> bool {
		self.peek().is_none()
	}
}

// parser impl
impl Parseable {
	fn consume(&mut self, expected: &lexer::Token) -> Result<(), String> {
		let token: lexer::Token = self.next().ok_or_else(||
			format!("ParsingError: expected «{expected:?}», got end of input")
		)?;

		if token == *expected {
			Ok(())
		} else {
			Err(format!("ParsingError: expected «{expected:?}», got «{token:?}»"))
		}
	}

	fn consume_ident(&mut self) -> Result<String, String> {
		let token: lexer::Token = self.next().ok_or_else(||
			"ParsingError: expected ident, got end of input".to_owned()
		)?;

		if let lexer::Token::Ident(name) = token {
			Ok(name)
		} else {
			Err(format!("ParsingError: expected ident, got «{token:?}»"))
		}
	}

	fn consume_integer(&mut self) -> Result<i128, String> {
		let token: lexer::Token = self.next().ok_or_else(||
			"ParsingError: expected number, got end of line".to_owned()
		)?;

		if let lexer::Token::Integer(integer) = token {
			Ok(integer)
		} else {
			Err(format!("ParsingError: expected number, got «{token:?}»"))
		}
	}

	fn parse_switch(&mut self, compared: ASTNode) -> Result<ASTNode, String> {
		let _: Option<lexer::Token> = self.next();
		self.consume(&lexer::Token::OpenBrace)?;
		let mut cases: Vec<(ASTNode, ASTNode)> = vec![];

		while !self.is_empty() && !self.is_delimiter() {
			let case: ASTNode = self.parse_expression(true, false)?;
			self.consume(&lexer::Token::Arrow)?;
			cases.push((case, self.parse_expression(true, true)?));
		}

		self.consume(&lexer::Token::CloseBrace)?;
		Ok(ASTNode::Switch {
			compared: Box::new(compared),
			cases
		})
	}

	fn is_delimiter(&self) -> bool {
		self.peek().is_some_and(|current| matches!(current,
			lexer::Token::CloseParen
			| lexer::Token::CloseBracket
			| lexer::Token::CloseBrace
			| lexer::Token::Dollar
		))
	}

	fn is_operation(&self) -> bool {
		self.peek().is_some_and(|current| matches!(current,
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
			| lexer::Token::Greater
		))
	}

	fn parse_operation(&mut self, left: ASTNode) -> Result<ASTNode, String> {
		let current: lexer::Token = self.next().unwrap_or_else(|| unreachable!("what"));

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

		let index: usize = self.current_index;
		Ok(match self.parse_expression(true, allow_operations) {
			Err(_) => {
				self.current_index = index;
				ASTNode::Value(Value::Lambda {
					args_def: "Y".to_owned(),
					content: Box::new(ASTNode::Operation {
						left: Box::new(left),
						operation,
						right: Box::new(ASTNode::Value(Value::Variable("Y".to_owned())))
					})
				})
			}

			Ok(content) => {
				let tmp: ASTNode = ASTNode::Operation {
					left: Box::new(left),
					operation,
					right: Box::new(content)
				};

				let res: ASTNode = if allow_repeat && self.is_operation() {
					self.parse_operation(tmp)?
				} else {
					tmp
				};

				if self.peek() == Some(lexer::Token::Dollar) {
					self.parse_switch(res)?
				} else {
					res
				}
			}
		})
	}

	fn parse_partial_operation(&mut self) -> Result<ASTNode, String> {
		let result: ASTNode = self.parse_operation(ASTNode::Value(Value::Variable("X".to_owned())))?;
		let (args_def, content): (String, Box<ASTNode>) =
			if let ASTNode::Value(Value::Lambda {args_def: ad, content: ct}) = result {
				("X".to_owned() + ad.as_str(), ct)
			} else {
				("X".to_owned(), Box::new(result))
			};

		Ok(ASTNode::Value(Value::Lambda {
			args_def,
			content
		}))
	}

	fn parse_pair(&mut self, from_call: bool, allow_operations: bool) -> Result<ASTNode, String> {
		let _: Option<lexer::Token> = self.next();

		if self.peek() == Some(lexer::Token::CloseParen) {
			let _: Option<lexer::Token> = self.next();
			return Ok(ASTNode::Value(Value::None))
		}

		let result: ASTNode = self.parse_expression(false, true)?;
		self.consume(&lexer::Token::CloseParen)?;

		let res: ASTNode = if let ASTNode::Value(ref value) = result {
			if from_call {
				result
			} else if let Value::Lambda {..} = value {
				let mut args: Vec<ASTNode> = vec![];

				while !self.is_empty()
				&& !self.is_delimiter()
				&& !self.is_operation()
				&& self.peek() != Some(lexer::Token::Period) {
					args.push(self.parse_expression(false, true)?);
				}

				if args.is_empty() {
					result
				} else {
					ASTNode::LambdaCall {
						lambda: Box::new(value.clone()),
						args
					}
				}
			} else {
				result
			}
		} else {
			result
		};

		Ok(if let Some(token) = self.peek() {
			if allow_operations && self.is_operation() {
				self.parse_operation(res)?
			} else if token == lexer::Token::Dollar {
				self.parse_switch(res)?
			} else if token == lexer::Token::Period {
				let _: Option<lexer::Token> = self.next();
				res
			} else {
				res
			}
		} else {
			res
		})
	}

	fn parse_integer_part(&mut self, allow_operations: bool) -> Result<ASTNode, String> {
		let _: Option<lexer::Token> = self.next();

		Ok(if self.peek() == Some(lexer::Token::CloseBracket) {
			let _: Option<lexer::Token> =  self.next();
			ASTNode::Value(Value::Lambda {
				args_def: "X".to_owned(),
				content: Box::new(
					ASTNode::IntegerPart(Box::new(
						ASTNode::Value(Value::Variable("X".to_owned()))
					)))
			})
		} else {
			let result: ASTNode = ASTNode::IntegerPart(Box::new(self.parse_expression(false, true)?));
			self.consume(&lexer::Token::CloseBracket)?;

			if allow_operations && self.is_operation() {
				self.parse_operation(result)?
			} else if self.peek() == Some(lexer::Token::Dollar) {
				self.parse_switch(result)?
			} else {
				result
			}
		})
	}

	fn parse_rational_part(&mut self, allow_operations: bool) -> Result<ASTNode, String> {
		let _: Option<lexer::Token> = self.next();
		Ok(if self.peek() == Some(lexer::Token::CloseBrace) {
			let _: Option<lexer::Token> = self.next();

			ASTNode::Value(Value::Lambda {
				args_def: "X".to_owned(),
				content: Box::new(
				ASTNode::RationalPart(Box::new(
					ASTNode::Value(Value::Variable("X".to_owned()))
				)))
			})
		} else {
			let result: ASTNode = ASTNode::RationalPart(Box::new(self.parse_expression(false, true)?));
			self.consume(&lexer::Token::CloseBrace)?;

			if allow_operations && self.is_operation() {
				self.parse_operation(result)?
			} else {
				result
			}
		})
	}


	fn parse_ident(&mut self, from_call: bool, allow_operations: bool) -> Result<ASTNode, String> {
		let name: String = self.consume_ident()?;

		Ok(if from_call {
			let result: ASTNode = ASTNode::Value(Value::Variable(name));
			if allow_operations && self.is_operation() {
				self.parse_operation(result)?
			} else if self.peek() == Some(lexer::Token::Dollar) {
				self.parse_switch(result)?
			} else {
				result
			}
		} else if self.peek() == Some(lexer::Token::Equal) {
			let _: Option<lexer::Token> = self.next();
			ASTNode::Definition {
				name,
				value: Box::new(self.parse_expression(false, true)?)
			}
		} else {
			let mut args: Vec<ASTNode> = vec![];

			while !self.is_empty()
			&& !self.is_delimiter()
			&& !self.is_operation()
			&& self.peek() != Some(lexer::Token::Period) {
				args.push(self.parse_expression(true, true)?);
			}

			let result: ASTNode = if args.is_empty() {
				ASTNode::Value(Value::Variable(name))
			} else {
				ASTNode::Call {
					name,
					args
				}
			};

			if let Some(token) = self.peek() {
				if allow_operations && self.is_operation() {
					self.parse_operation(result)?
				} else if token == lexer::Token::Period {
					let _: Option<lexer::Token> = self.next();
					result
				} else if !from_call && token == lexer::Token::Dollar {
					self.parse_switch(result)?
				} else {
					if token == lexer::Token::Period {
						let _: Option<lexer::Token> = self.next();
					}

					result
				}
			} else {
				result
			}
		})
	}

	fn parse_expression(&mut self, from_call: bool, allow_operations: bool) -> Result<ASTNode, String> {
		let Some(current): Option<lexer::Token> = self.peek() else {
			return Ok(ASTNode::Nothing)
		};

		match current {
			lexer::Token::Exclam => {
				let _: Option<lexer::Token> = self.next();
				Ok(ASTNode::Print(Box::new(self.parse_expression(false, true)?)))
			}

			lexer::Token::OpenBracket => self.parse_integer_part(allow_operations),
			lexer::Token::OpenBrace => self.parse_rational_part(allow_operations),
			lexer::Token::OpenParen => self.parse_pair(from_call, allow_operations),

			lexer::Token::Lambda => {
				let _: Option<lexer::Token> = self.next();
				let mut args_def: String = self.consume_ident()?;
				self.consume(&lexer::Token::Period)?;
				let mut body: Box<ASTNode> = Box::new(self.parse_expression(false, true)?);

				if let ASTNode::Value(Value::Lambda{args_def: args_def_, content}) = *body.clone() {
					args_def += args_def_.as_str();
					body = content;
				}

				if self.peek() == Some(lexer::Token::Period) {
					let _: Option<lexer::Token> = self.next();
				}

				Ok(ASTNode::Value(Value::Lambda{
					args_def,
					content: body
				}))
			}

			lexer::Token::Integer(_) => {
				let integer: i128 = self.consume_integer()?;

				let result: ASTNode = if self.peek() == Some(lexer::Token::Period) {
					let _: Option<lexer::Token> = self.next();
					let rational: i128 = self.consume_integer()?;
					ASTNode::Value(Value::Decimal(format!("{integer}.{rational}").parse::<f64>().unwrap()))
				} else {
					ASTNode::Value(Value::Integer(integer))
				};

				Ok(if allow_operations && self.is_operation() {
					self.parse_operation(result)?
				} else if !from_call && self.peek() == Some(lexer::Token::Dollar) {
					self.parse_switch(result)?
				} else {
					result
				})
			}

			lexer::Token::Ident(_) => self.parse_ident(from_call, allow_operations),
			_ if self.is_operation() => self.parse_partial_operation(),

			what => {
				Err(format!("ParsingError: expected expression start, got «{:?}», previous token is {:?}",
						what, self.tokens.get(self.current_index-1)))
			}
		}
	}

	pub fn parse(&mut self) -> Result<Vec<ASTNode>, String> {
		let mut result: Vec<ASTNode> = vec![];

		while !self.is_empty() {
			result.push(self.parse_expression(false, true)?);
		}

		Ok(result)
	}
}

pub fn parse(source: &str) -> Result<Vec<ASTNode>, String> {
	Parseable::new(lexer::lex(source)?).parse()
}
