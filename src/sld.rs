use crate::{
    ast::{Clause, Program, Term},
    unify::Substitutions,
};

/// Recursively appends a suffix "_n" where `n` is `suffix`
/// to the name of every variable under the given term.
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

/// Calls [`rename_vars_in_term`] for every term under `clause`.
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

/// Resolves some goals using SLD Resolution.
///
/// 1. If `goals` is empty yield the current substitutions.
/// 2. Otherwise, take the first goal. For each clause in the program whose head unifies with the
///    goal (after renaming the clause's variables):
///    a. Compute the MGU
///    b. Apply it to the remaining goals
///    c. Prepend the clause's body to the remaining goals.
///    d. Recurse with the new goals and the composed substitutions.
/// 3. Collect all successful substitutions into a vector and return it.
pub fn resolve(
    goals: &[Term],
    program: Program,
    substitutions: Substitutions,
) -> Vec<Substitutions> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Clause, Term},
        sld::rename_vars,
    };

    #[test]
    fn test_rename_fact() {
        // Facts have no variables, so renaming should leave them unchanged
        let clause = Clause::fact(Term::Compound(
            "parent".to_string(),
            vec![
                Term::Atom("john".to_string()),
                Term::Atom("mary".to_string()),
            ],
        ));

        let renamed = rename_vars(&clause, 1);

        assert_eq!(renamed.head, clause.head);
        assert_eq!(renamed.body, clause.body);
    }

    #[test]
    fn test_rename_simple_rule() {
        // ancestor(X, Y) :- parent(X, Y).
        let clause = Clause::rule(
            Term::Compound(
                "ancestor".to_string(),
                vec![Term::Var("X".to_string()), Term::Var("Y".to_string())],
            ),
            vec![Term::Compound(
                "parent".to_string(),
                vec![Term::Var("X".to_string()), Term::Var("Y".to_string())],
            )],
        );

        let renamed = rename_vars(&clause, 1);

        // Head should be: ancestor(X_1, Y_1)
        match renamed.head {
            Term::Compound(f, args) => {
                assert_eq!(f, "ancestor");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Term::Var("X_1".to_string()));
                assert_eq!(args[1], Term::Var("Y_1".to_string()));
            }
            _ => panic!("Expected compound term"),
        }

        // Body should be: [parent(X_1, Y_1)]
        assert_eq!(renamed.body.len(), 1);
        match &renamed.body[0] {
            Term::Compound(f, args) => {
                assert_eq!(f, "parent");
                assert_eq!(args[0], Term::Var("X_1".to_string()));
                assert_eq!(args[1], Term::Var("Y_1".to_string()));
            }
            _ => panic!("Expected compound term in body"),
        }
    }

    #[test]
    fn test_rename_recursive_rule() {
        // ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
        let clause = Clause::rule(
            Term::Compound(
                "ancestor".to_string(),
                vec![Term::Var("X".to_string()), Term::Var("Y".to_string())],
            ),
            vec![
                Term::Compound(
                    "parent".to_string(),
                    vec![Term::Var("X".to_string()), Term::Var("Z".to_string())],
                ),
                Term::Compound(
                    "ancestor".to_string(),
                    vec![Term::Var("Z".to_string()), Term::Var("Y".to_string())],
                ),
            ],
        );

        let renamed = rename_vars(&clause, 2);

        // Head: ancestor(X_2, Y_2)
        match renamed.head {
            Term::Compound(_, args) => {
                assert_eq!(args[0], Term::Var("X_2".to_string()));
                assert_eq!(args[1], Term::Var("Y_2".to_string()));
            }
            _ => panic!("Expected compound term"),
        }

        // Body should have 2 goals
        assert_eq!(renamed.body.len(), 2);

        // First goal: parent(X_2, Z_2)
        match &renamed.body[0] {
            Term::Compound(f, args) => {
                assert_eq!(f, "parent");
                assert_eq!(args[0], Term::Var("X_2".to_string()));
                assert_eq!(args[1], Term::Var("Z_2".to_string()));
            }
            _ => panic!("Expected compound term"),
        }

        // Second goal: ancestor(Z_2, Y_2)
        match &renamed.body[1] {
            Term::Compound(f, args) => {
                assert_eq!(f, "ancestor");
                assert_eq!(args[0], Term::Var("Z_2".to_string()));
                assert_eq!(args[1], Term::Var("Y_2".to_string()));
            }
            _ => panic!("Expected compound term"),
        }
    }

    #[test]
    fn test_rename_different_suffixes() {
        // Same clause renamed twice with different suffixes should produce different variables
        let clause = Clause::rule(
            Term::Compound("foo".to_string(), vec![Term::Var("X".to_string())]),
            vec![Term::Compound(
                "bar".to_string(),
                vec![Term::Var("X".to_string())],
            )],
        );

        let renamed_1 = rename_vars(&clause, 1);
        let renamed_2 = rename_vars(&clause, 2);

        // They should be different
        assert_ne!(renamed_1.head, renamed_2.head);
        assert_ne!(renamed_1.body, renamed_2.body);

        // Check that they have different suffixes
        match renamed_1.head {
            Term::Compound(_, args) => {
                assert_eq!(args[0], Term::Var("X_1".to_string()));
            }
            _ => panic!("Expected compound term"),
        }

        match renamed_2.head {
            Term::Compound(_, args) => {
                assert_eq!(args[0], Term::Var("X_2".to_string()));
            }
            _ => panic!("Expected compound term"),
        }
    }

    #[test]
    fn test_rename_mixed_atoms_and_vars() {
        // sibling(X, Y) :- parent(Z, X), parent(Z, Y).
        let clause = Clause::rule(
            Term::Compound(
                "sibling".to_string(),
                vec![Term::Var("X".to_string()), Term::Var("Y".to_string())],
            ),
            vec![
                Term::Compound(
                    "parent".to_string(),
                    vec![Term::Var("Z".to_string()), Term::Var("X".to_string())],
                ),
                Term::Compound(
                    "parent".to_string(),
                    vec![Term::Var("Z".to_string()), Term::Var("Y".to_string())],
                ),
            ],
        );

        let renamed = rename_vars(&clause, 5);

        // Atoms should remain unchanged
        match &renamed.body[0] {
            Term::Compound(f, _) => {
                assert_eq!(f, "parent"); // Functor name unchanged
            }
            _ => panic!("Expected compound term"),
        }

        // All variables should be renamed with suffix _5
        match renamed.head {
            Term::Compound(_, args) => {
                assert_eq!(args[0], Term::Var("X_5".to_string()));
                assert_eq!(args[1], Term::Var("Y_5".to_string()));
            }
            _ => panic!("Expected compound term"),
        }

        match &renamed.body[0] {
            Term::Compound(_, args) => {
                assert_eq!(args[0], Term::Var("Z_5".to_string()));
                assert_eq!(args[1], Term::Var("X_5".to_string()));
            }
            _ => panic!("Expected compound term"),
        }
    }

    #[test]
    fn test_rename_preserves_structure() {
        // Renaming should not change the structure, only variable names
        let clause = Clause::rule(
            Term::Compound(
                "test".to_string(),
                vec![
                    Term::Var("A".to_string()),
                    Term::Atom("constant".to_string()),
                ],
            ),
            vec![
                Term::Compound("foo".to_string(), vec![Term::Var("A".to_string())]),
                Term::Atom("bar".to_string()),
            ],
        );

        let renamed = rename_vars(&clause, 10);

        // Head arity should be preserved
        match renamed.head {
            Term::Compound(_, args) => {
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected compound term"),
        }

        // Body length should be preserved
        assert_eq!(renamed.body.len(), 2);

        // Atoms should remain exactly the same
        match &renamed.body[1] {
            Term::Atom(a) => assert_eq!(a, "bar"),
            _ => panic!("Expected atom"),
        }
    }

    #[test]
    fn test_rename_empty_body() {
        // Fact with single atom head
        let clause = Clause::fact(Term::Atom("true".to_string()));

        let renamed = rename_vars(&clause, 1);

        assert_eq!(renamed.head, Term::Atom("true".to_string()));
        assert_eq!(renamed.body.len(), 0);
    }
}
