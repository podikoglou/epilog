#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Term {
    Atom(String),
    Var(String),
    Compound(String, Vec<Term>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Clause {
    head: Term,
    body: Vec<Term>,
}

impl Clause {
    pub fn fact(head: Term) -> Self {
        Clause { head, body: vec![] }
    }

    pub fn rule(head: Term, body: Vec<Term>) -> Self {
        Clause { head, body }
    }
}
