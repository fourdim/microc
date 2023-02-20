mod white_space {
    static WHITESPACE_MAP: [u8; 256] = [
        2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    #[inline]
    pub fn lookup(c: char) -> bool {
        match c as u32 >> 8 {
            0 => WHITESPACE_MAP[c as usize & 0xff] & 1 != 0,
            22 => c as u32 == 0x1680,
            32 => WHITESPACE_MAP[c as usize & 0xff] & 2 != 0,
            48 => c as u32 == 0x3000,
            _ => false,
        }
    }
}

pub fn is_whitespace(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    match c {
        ' ' | '\x09'..='\x0d' => true,
        c => c > '\x7f' && white_space::lookup(c),
    }
}

pub fn is_identifier_continue(c: char) -> bool {
    matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z')
}

pub fn is_digit(c: char) -> bool {
    matches!(c, '0'..='9')
}

pub fn is_expected(c: char) -> bool {
    is_identifier_continue(c)
        || is_whitespace(c)
        || matches!(c, '=' | '+' | '-' | '(' | ')' | ';' | ',')
}
