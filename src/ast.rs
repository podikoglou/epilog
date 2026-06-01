use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Term {
    Atom(String),
    Var(String),
    Compound(String, Vec<Term>),
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Atom(atom) => write!(f, "{}", atom),
            Term::Var(var) => write!(f, "{}", var),
            Term::Compound(atom, terms) => {
                write!(
                    f,
                    "{}({})",
                    atom,
                    terms
                        .iter()
                        .map(|term| term.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Clause {
    pub head: Term,
    pub body: Vec<Term>,
}

impl Clause {
    pub fn fact(head: Term) -> Self {
        Clause { head, body: vec![] }
    }

    pub fn rule(head: Term, body: Vec<Term>) -> Self {
        Clause { head, body }
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.body.is_empty() {
            write!(f, "{}.", self.head)
        } else {
            write!(
                f,
                "{} :- {}.",
                self.head,
                self.body
                    .iter()
                    .map(|term| term.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
