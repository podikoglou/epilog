use chumsky::prelude::*;

use crate::ast::{Clause, Term};

pub type Program = Vec<Clause>;

fn var<'src>() -> impl Parser<'src, &'src str, String> + Clone {
    regex(r"[_A-Z][a-zA-Z0-9_]*").map(|s: &str| s.to_string())
}

fn atom<'src>() -> impl Parser<'src, &'src str, String> + Clone {
    regex(r"[a-z][a-zA-Z0-9_]*").map(|s: &str| s.to_string())
}

fn term<'src>() -> impl Parser<'src, &'src str, Term> + Clone {
    recursive(|term| {
        let var_term = var().map(Term::Var);

        let compound = atom()
            .then(
                term.clone()
                    .separated_by(just(','))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(name, args)| Term::Compound(name, args));

        let atom_term = atom().map(Term::Atom);

        choice((var_term, compound, atom_term)).padded()
    })
}

fn terms<'src>() -> impl Parser<'src, &'src str, Vec<Term>> + Clone {
    term()
        .separated_by(just(','))
        .allow_trailing()
        .collect::<Vec<_>>()
        .padded()
}

fn clause<'src>() -> impl Parser<'src, &'src str, Clause> + Clone {
    let fact = term().then_ignore(just('.')).map(Clause::fact);

    let rule = term()
        .then_ignore(just(':').then(just('-')))
        .then(terms())
        .then_ignore(just('.'))
        .map(|(head, body)| Clause::rule(head, body));

    choice((rule, fact)).padded()
}

pub fn program<'src>() -> impl Parser<'src, &'src str, Program> {
    clause().repeated().collect::<Vec<_>>().padded().then_ignore(end())
}

pub fn parse(input: &str) -> Result<Program, Vec<String>> {
    program()
        .parse(input)
        .into_result()
        .map_err(|errs| errs.into_iter().map(|e| e.to_string()).collect())
}
