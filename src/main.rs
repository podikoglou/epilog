#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Term {
    Atom(String),
    Var(String),
    Compound(String, Vec<Term>),
}

fn main() {
    println!("Hello, world!");
}
