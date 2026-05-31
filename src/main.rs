pub mod ast;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;

    #[test]
    fn parse_fact() {
        let prog = parser::ProgramParser::new().parse("likes.").unwrap();
        assert_eq!(prog, vec![Clause::fact(Term::Atom("likes".into()))]);
    }

    #[test]
    fn parse_var_fact() {
        let prog = parser::ProgramParser::new().parse("X.").unwrap();
        assert_eq!(prog, vec![Clause::fact(Term::Var("X".into()))]);
    }

    #[test]
    fn parse_rule() {
        let prog = parser::ProgramParser::new()
            .parse("mortal(X) :- human(X).")
            .unwrap();
        assert_eq!(
            prog,
            vec![Clause::rule(
                Term::Compound("mortal".into(), vec![Term::Var("X".into())]),
                vec![Term::Compound("human".into(), vec![Term::Var("X".into())])],
            )]
        );
    }

    #[test]
    fn parse_multiple_clauses() {
        let prog = parser::ProgramParser::new()
            .parse("human(socrates). mortal(X) :- human(X).")
            .unwrap();
        assert_eq!(prog.len(), 2);
        assert_eq!(prog[0], Clause::fact(Term::Compound("human".into(), vec![Term::Atom("socrates".into())])));
        assert_eq!(
            prog[1],
            Clause::rule(
                Term::Compound("mortal".into(), vec![Term::Var("X".into())]),
                vec![Term::Compound("human".into(), vec![Term::Var("X".into())])],
            )
        );
    }

    #[test]
    fn parse_compound_multiple_args() {
        let prog = parser::ProgramParser::new()
            .parse("parent(a, B, c).")
            .unwrap();
        assert_eq!(
            prog,
            vec![Clause::fact(Term::Compound(
                "parent".into(),
                vec![
                    Term::Atom("a".into()),
                    Term::Var("B".into()),
                    Term::Atom("c".into()),
                ],
            ))]
        );
    }

    #[test]
    fn parse_underscore_var() {
        let prog = parser::ProgramParser::new()
            .parse("likes(_, X).")
            .unwrap();
        assert_eq!(
            prog,
            vec![Clause::fact(Term::Compound(
                "likes".into(),
                vec![Term::Var("_".into()), Term::Var("X".into())],
            ))]
        );
    }

    #[test]
    fn parse_multi_body_rule() {
        let prog = parser::ProgramParser::new()
            .parse("sibling(X, Y) :- parent(Z, X), parent(Z, Y).")
            .unwrap();
        assert_eq!(prog.len(), 1);
        let clause = &prog[0];
        assert_eq!(clause.body.len(), 2);
    }
}

fn main() {
    println!("Hello, world!");
}
