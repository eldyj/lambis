mod check;
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
	Apostrophe,    // '

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
	Integer(i128), // [0-9]+
}

trait LexableExt<'a> {
	fn lex_ident(&mut self) -> Token;
	fn lex_integer(&mut self) -> Token;
	fn lex_multiline_comment(&mut self);
	fn lex_comment(&mut self);
	fn lex_spaces(&mut self);
	fn lex(&mut self) -> Result<Vec<Token>, String>;
}


impl LexableExt<'_> for Lexable<'_> {
	fn lex_ident(&mut self) -> Token {
		let mut result: String = String::new();

		while let Some(&ch) = self.peek() {
			if !check::is_ident(ch) {
				break
			}

			let _: Option<char> = self.next();
			result.push(ch);
		}

		Token::Ident(result)
	}

	fn lex_integer(&mut self) -> Token {
		let mut temporary: String = String::new();

		while let Some(&ch) = self.peek() {
			if !check::is_numeric(ch) {
				break
			}

			let _: Option<char> = self.next();
			temporary.push(ch);
		}

		Token::Integer(temporary.parse::<i128>().unwrap())
	}

	fn lex_spaces(&mut self) {
		while let Some(&ch) = self.peek() {
			if !check::is_space(ch) {
				break
			}

			let _: Option<char> = self.next();
		}
	}

	fn lex_multiline_comment(&mut self) {
		while let Some(&ch) = self.peek() {
			let _: Option<char> = self.next();
			if ch == '#' && self.peek() == Some(&'#') {
				let _: Option<char> = self.next();
				break
			}
		}
	}

	fn lex_comment(&mut self) {
		let _: Option<char> = self.next();
		if self.peek() == Some(&'#') {
			self.lex_multiline_comment();
		} else {
			while let Some(&ch) = self.peek() {
				if ch == '\n' {
					break
				}

				let _: Option<char> = self.next();
			}
		}
	}

	fn lex(&mut self) -> Result<Vec<Token>, String> {
		let mut result: Vec<Token> = vec![];

		while let Some(&ch) = self.peek() {
			if check::is_space(ch) {
				self.lex_spaces();
			} else if check::is_ident_start(ch) {
				result.push(self.lex_ident());
			} else if check::is_numeric(ch) {
				result.push(self.lex_integer());
			} else if ch == '#' {
				self.lex_comment();
			} else {
				result.push(match ch {
					'$' => Token::Dollar,
					'.' => Token::Period,
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
					'\'' => Token::Apostrophe,
					'→' => Token::Arrow,
					 _  => return Err("LexError: what the fuck is {ch}".to_owned()),
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
