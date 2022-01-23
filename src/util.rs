pub fn is_space(c: &u8) -> bool {
    [10, 13, 32].contains(c)
}

pub fn is_digit(c: &u8) -> bool {
    (45..57).contains(c)
}

pub fn is_alpha(c: &u8) -> bool {
    (65..90).contains(c) || (97..122).contains(c)
}

pub fn is_alnum(c: &u8) -> bool {
    is_digit(c) || is_alpha(c)
}


// pub fn is_alpha(c: u8) -> bool {

// }