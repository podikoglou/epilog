use crate::ast::{Clause, Term};

fn rename_vars_in_term(term: &Term, suffix: usize) -> Term {
    match term {
        Term::Var(name) => Term::Var(format!("{}_{}", name.to_string(), suffix)),
        Term::Compound(functor, terms) => Term::Compound(
            functor.to_string(),
            terms
                .iter()
                .map(|term| rename_vars_in_term(term, suffix))
                .collect(),
        ),

        _ => term.clone(),
    }
}

fn rename_vars(clause: &Clause, suffix: usize) -> Clause {
    let head = rename_vars_in_term(&clause.head, suffix);

    if clause.body.is_empty() {
        Clause::fact(head)
    } else {
        let body = clause
            .body
            .iter()
            .map(|term| rename_vars_in_term(term, suffix))
            .collect::<Vec<_>>();

        Clause::rule(head, body)
    }
}
