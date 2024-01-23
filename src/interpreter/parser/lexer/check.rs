pub const fn is_space(c: char) -> bool {
	matches!(c, ' '|' '|'\t'|'\n')
}

pub const fn is_numeric(c: char) -> bool {
	c.is_ascii_digit()
}

pub const fn is_ident(c: char) -> bool {
	is_ident_start(c)
	|| is_numeric(c)
}

pub const fn is_ident_start(c: char) -> bool {
	c.is_ascii_alphabetic()
}
