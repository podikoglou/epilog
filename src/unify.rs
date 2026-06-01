use std::collections::HashMap;

use crate::ast::Term;

type Substitutions = HashMap<String, Term>;

pub fn apply(substitutions: Substitutions, term: &Term) -> Term {
    match term {
        Term::Atom(_) => term.clone(),
        Term::Var(var) => substitutions.get(var).unwrap_or(term).clone(),
        Term::Compound(atom, terms) => Term::Compound(
            atom.clone(),
            terms
                .iter()
                .map(|term| apply(substitutions.clone(), term))
                .collect::<Vec<_>>(),
        ),
    }
}

pub fn var_occurs(var: &str, term: &Term) -> bool {
    match term {
        Term::Atom(_) => false,
        Term::Var(var2) => var2 == var,
        Term::Compound(_, terms2) => terms2.iter().any(|term| var_occurs(var, term)),
    }
}
