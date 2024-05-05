use std::{iter::Peekable, str::Chars};
type Lexable<'a> = Peekable<Chars<'a>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
	OpenParen,     // (
	CloseParen,    // )
	OpenBrace,     // {
	CloseBrace,    // }
	OpenBracket,   // [
	CloseBracket,  // ]
	Exclam,        // !
	Bar,           // |
	Underscore,    // _
	Plus,          // +
	Minus,         // -
	Asterisk,      // *
	Slash,         // /
	Circumflex,    // ^
	Equal,         // =
	NotEqual,      // !=
	Less,          // <
	Greater,       // >
	LessEqual,     // >=
	GreaterEqual,  // <=
	Arrow,         // ->
	Lambda,        // λ
	Period,        // .
	Dollar,        // $
    Ident(String), // [a-zA-Z0-9]+
    Word(String),  // '[a-zA-Z0-9]+
	Integer(i128), // [0-9]+
}

trait LexableExt<'a> {
	fn lex_ident(&mut self) -> Token;
    fn lex_word(&mut self) -> Token;
	fn lex_integer(&mut self) -> Token;
	fn lex_multiline_comment(&mut self);
	fn lex_comment(&mut self);
	fn lex_spaces(&mut self);
	fn lex(&mut self) -> Result<Vec<Token>, String>;
}


impl LexableExt<'_> for Lexable<'_> {
	fn lex_ident(&mut self) -> Token {
		let mut result: String = String::new();

		while self.peek().is_some_and(char::is_ascii_alphanumeric) {
			result.push(self.next().unwrap());
		}

		Token::Ident(result)
	}

    fn lex_word(&mut self) -> Token {
        let _: Option<char> = self.next();
        let Token::Ident(s) = self.lex_ident() else {
            unreachable!("urmom");
        };
        Token::Word(s)
    }

	fn lex_integer(&mut self) -> Token {
		let mut temporary: String = String::new();

		while self.peek().is_some_and(char::is_ascii_digit) {
			temporary.push(self.next().unwrap());
		}

		Token::Integer(temporary.parse::<i128>().unwrap())
	}

	fn lex_spaces(&mut self) {
		while self.peek().is_some_and(|&ch: &char| ch.is_ascii_whitespace()) {
			let _: Option<char> = self.next();
		}
	}

	fn lex_multiline_comment(&mut self) {
        self.next();
		while self.peek().is_some() && !(self.next().unwrap() == '#' && self.peek().is_some_and(|&ch: &char| ch == '#')) {
            let _: Option<char> = self.next();
		}
	}

	fn lex_comment(&mut self) {
		let _: Option<char> = self.next();
		if self.peek().is_some_and(|&ch: &char| ch == '#') {
			self.lex_multiline_comment();
		} else {
			while self.peek().is_some_and(|&ch: &char| ch != '\n') {
				let _: Option<char> = self.next();
			}
		}
	}

	fn lex(&mut self) -> Result<Vec<Token>, String> {
		let mut result: Vec<Token> = vec![];

		while let Some(&ch) = self.peek() {
			if ch.is_ascii_whitespace() {
				self.lex_spaces();
			} else if ch.is_ascii_alphabetic() {
				result.push(self.lex_ident());
			} else if ch.is_ascii_digit() {
				result.push(self.lex_integer());
			} else if ch == '#' {
				self.lex_comment();
			} else {
				result.push(match ch {
					'$' => Token::Dollar,
					'.' => Token::Period,
                    '\'' => self.lex_word(),
					'<' => {
						let mut clone: Lexable = self.clone();
						let _: Option<char> = clone.next();

						clone.peek().and_then(|ch: &char| -> Option<Token> {
							let res: Option<Token> = Some(match *ch {
								'=' => Token::LessEqual,
								'>' => Token::NotEqual,
								_ => return None
							});

							let _: Option<char> = self.next();
							res
						}).unwrap_or(Token::Less)
					}

					'>' => {
						let mut clone: Lexable = self.clone();
						let _: Option<char> = clone.next();
						if clone.peek() == Some(&'=') {
							let _: Option<char> = self.next();
							Token::GreaterEqual
						} else {
							Token::Greater
						}
					}

					'!' => {
						let mut clone: Lexable = self.clone();
						let _: Option<char> = clone.next();

						clone.peek().and_then(|ch: &char| -> Option<Token> {
							let res: Option<Token> = Some(match *ch {
								'=' => Token::NotEqual,
								'<' => Token::GreaterEqual,
								'>' => Token::LessEqual,
								_ => return None,
							});

							let _: Option<char> = self.next();
							res
						}).unwrap_or(Token::Exclam)
					}

					'⩾'|'≧'|'≥'|'≮' => Token::GreaterEqual,
					'⩽'|'≦'|'≤'|'≯' => Token::LessEqual,
					'≱' => Token::Less,
					'≰' => Token::Greater,
					'≠' => Token::NotEqual,
					'=' => Token::Equal,
					'_' => Token::Underscore,
					'+' => Token::Plus,
					'-' => {
						let mut clone: Lexable = self.clone();
						let _: Option<char> = clone.next();

						if clone.peek() == Some(&'>') {
							let _: Option<char> = self.next();
							Token::Arrow
						} else {
							Token::Minus
						}
					}
					'*' => Token::Asterisk,
					'/' => Token::Slash,
					'^' => Token::Circumflex,
					'[' => Token::OpenBracket,
					']' => Token::CloseBracket,
					'{' => Token::OpenBrace,
					'}' => Token::CloseBrace,
					'(' => Token::OpenParen,
					')' => Token::CloseParen,
					'λ'|'\\' => Token::Lambda,
					'|' => Token::Bar,
					'→' => Token::Arrow,
					 _  => return Err(format!("LexError: what the fuck is {ch}").to_owned()),
				});

				let _: Option<char> = self.next();
			}
		}
		Ok(result)
	}
}

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
	source.chars().peekable().lex()
}
