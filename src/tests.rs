use crate::{
    ast::{Clause, Term},
    parser,
};

#[test]
fn roundtrip() {
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

    // pretty print clauses
    let clauses_str = clauses
        .iter()
        .map(|clause| clause.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    // parse
    let par = parser::ProgramParser::new();
    let parsed_clauses = par.parse(&clauses_str).expect("syntax error");

    // pretty print clauses
    let parsed_clauses_str = parsed_clauses
        .iter()
        .map(|clause| clause.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(clauses_str, parsed_clauses_str);
}
