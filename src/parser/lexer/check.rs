pub fn is_space(c: char) -> bool {
	match c {
		' '|' '|'\t'|'\n' => true,
		_ => false,
	}
}

pub fn is_numeric(c: char) -> bool {
	match c {
		'0'..='9' => true,
		_ => false,
	}
}

pub fn is_ident(c: char) -> bool {
	is_ident_start(c)
	|| is_numeric(c)
}

pub fn is_ident_start(c: char) -> bool {
	match c {
		'a'..='z'|'A'..='Z' => true,
		_ => false,
	}
}
