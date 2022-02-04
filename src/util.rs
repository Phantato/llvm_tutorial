use std::str;

pub fn is_space(c: &u8) -> bool {
    [10, 13, 32].contains(c)
}

pub fn is_digit(c: &u8) -> bool {
    (48..58).contains(c)
}

pub fn is_alpha(c: &u8) -> bool {
    (65..91).contains(c) || (97..123).contains(c)
}

pub fn is_alnum(c: &u8) -> bool {
    is_digit(c) || is_alpha(c)
}

pub fn str_from_u8(v: &Vec<u8>) -> &str {
    str::from_utf8(v).unwrap()
}
