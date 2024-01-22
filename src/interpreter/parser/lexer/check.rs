pub fn is_space(c: char) -> bool {
	matches!(c, ' '|' '|'\t'|'\n')
}

pub fn is_numeric(c: char) -> bool {
	c.is_ascii_digit()
}

pub fn is_ident(c: char) -> bool {
	is_ident_start(c)
	|| is_numeric(c)
}

pub fn is_ident_start(c: char) -> bool {
	c.is_ascii_alphabetic()
}
