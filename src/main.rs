pub mod ast;

use lalrpop_util::lalrpop_mod;

use crate::ast::{Clause, Term};
lalrpop_mod!(pub parser);

fn main() {
    let clauses = vec![
        Clause::fact(Term::Compound(
            String::from("parent"),
            vec![
                Term::Atom(String::from("john")),
                Term::Atom(String::from("mary")),
            ],
        )),
        Clause::fact(Term::Compound(
            String::from("parent"),
            vec![
                Term::Atom(String::from("john")),
                Term::Atom(String::from("david")),
            ],
        )),
        Clause::fact(Term::Compound(
            String::from("parent"),
            vec![
                Term::Atom(String::from("mary")),
                Term::Atom(String::from("alice")),
            ],
        )),
        Clause::fact(Term::Compound(
            String::from("parent"),
            vec![
                Term::Atom(String::from("david")),
                Term::Atom(String::from("bob")),
            ],
        )),
        Clause::rule(
            Term::Compound(
                String::from("ancestor"),
                vec![Term::Var(String::from("X")), Term::Var(String::from("Y"))],
            ),
            vec![Term::Compound(
                String::from("parent"),
                vec![Term::Var(String::from("X")), Term::Var(String::from("Y"))],
            )],
        ),
        Clause::rule(
            Term::Compound(
                String::from("ancestor"),
                vec![Term::Var(String::from("X")), Term::Var(String::from("Y"))],
            ),
            vec![
                Term::Compound(
                    String::from("parent"),
                    vec![Term::Var(String::from("X")), Term::Var(String::from("Z"))],
                ),
                Term::Compound(
                    String::from("ancestor"),
                    vec![Term::Var(String::from("Z")), Term::Var(String::from("Y"))],
                ),
            ],
        ),
    ];

    for clause in clauses {
        println!("{}", clause)
    }
}
