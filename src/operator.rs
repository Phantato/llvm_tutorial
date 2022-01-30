use std::cmp::Ordering;

#[derive(PartialEq, Eq, Debug)]
pub enum Operator {
    Assign,
    Les,
    Add,
    Sub,
    Mul,
    Div,
    Other,
}

impl Operator {
    pub fn precedence(&self) -> i8 {
        match self {
            Operator::Assign => 10,
            Operator::Les => 20,
            Operator::Add => 30,
            Operator::Sub => 30,
            Operator::Mul => 40,
            Operator::Div => 40,
            _ => 0,
        }
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Operator {
    fn cmp(&self, other: &Self) -> Ordering {
        self.precedence().cmp(&other.precedence())
    }
}
