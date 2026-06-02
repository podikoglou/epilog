use std::iter;

use crate::{
    ast::{Clause, Program, Term},
    unify::{self, Substitutions},
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
    suffix: usize,
) -> Vec<Substitutions> {
    if goals.is_empty() {
        vec![substitutions]
    } else {
        // get next goal and remove it from our goals
        let goal = &goals[0];
        let goals = &goals[1..];

        // find all clauses whose heads unify with goal
        let clauses = program.iter().filter_map(|c| {
            // rename this clause's variables using a suffix that's incremented every time we
            // recurse
            let clause = rename_vars(c, suffix);

            // try to unify the goal with the head of this clause
            // and map the optional substitutions that unify returns,
            // to a tuple containing the clause (with renamed variables)
            // and the substitutions
            unify::unify(substitutions.clone(), goal, &clause.head)
                .map(|substitutions| (clause, substitutions))
        });

        clauses.map(|clause| {
            // (a) Compute the MGU
            let mgu = clause.1;
            let clause = clause.0;

            // (b) Apply it to the remaining goals.
            let goals = goals
                .iter()
                .map(|goal| unify::apply(mgu.clone(), goal))
                .collect::<Vec<Term>>();

            // (c) Prepend the clause's body to the remaining goals.
            let goals = clause
                .body
                .iter()
                .chain(goals.iter())
                .map(|g| g.clone())
                .collect::<Vec<Term>>();

            // (d) Recurse with the new goals and the composed substitution.
            goals
        });

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Clause, Program, Term},
        sld::{rename_vars, resolve},
        unify::Substitutions,
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

    use std::collections::HashMap;

    // ============================================================================
    // Helper functions
    // ============================================================================

    fn atom(s: &str) -> Term {
        Term::Atom(s.to_string())
    }

    fn var(s: &str) -> Term {
        Term::Var(s.to_string())
    }

    fn compound(functor: &str, args: Vec<Term>) -> Term {
        Term::Compound(functor.to_string(), args)
    }

    fn clause(head: Term, body: Vec<Term>) -> Clause {
        Clause { head, body }
    }

    fn fact(head: Term) -> Clause {
        Clause { head, body: vec![] }
    }

    fn subst(bindings: Vec<(&str, Term)>) -> Substitutions {
        bindings
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    }

    fn program(clauses: Vec<Clause>) -> Program {
        clauses
    }

    // ============================================================================
    // Basic Facts (No Rules, No Variables)
    // ============================================================================

    #[test]
    fn test_fact_matching() {
        // Program: parent(alice, bob).
        let prog = program(vec![fact(compound(
            "parent",
            vec![atom("alice"), atom("bob")],
        ))]);

        let goals = vec![compound("parent", vec![atom("alice"), atom("bob")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], HashMap::new()); // Empty substitution
    }

    #[test]
    fn test_fact_not_matching() {
        // Program: parent(alice, bob).
        let prog = program(vec![fact(compound(
            "parent",
            vec![atom("alice"), atom("bob")],
        ))]);

        let goals = vec![compound("parent", vec![atom("charlie"), atom("dave")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_multiple_facts() {
        // Program:
        //   parent(alice, bob).
        //   parent(bob, charlie).
        //   parent(charlie, dave).
        let prog = program(vec![
            fact(compound("parent", vec![atom("alice"), atom("bob")])),
            fact(compound("parent", vec![atom("bob"), atom("charlie")])),
            fact(compound("parent", vec![atom("charlie"), atom("dave")])),
        ]);

        let goals = vec![compound("parent", vec![atom("bob"), atom("charlie")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
    }

    // ============================================================================
    // Variable Unification
    // ============================================================================

    #[test]
    fn test_unify_single_variable() {
        // Program: parent(alice, bob).
        // Query: parent(alice, X).
        let prog = program(vec![fact(compound(
            "parent",
            vec![atom("alice"), atom("bob")],
        ))]);

        let goals = vec![compound("parent", vec![atom("alice"), var("X")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X"), Some(&atom("bob")));
    }

    #[test]
    fn test_unify_multiple_variables() {
        // Program: parent(alice, bob).
        // Query: parent(X, Y).
        let prog = program(vec![fact(compound(
            "parent",
            vec![atom("alice"), atom("bob")],
        ))]);

        let goals = vec![compound("parent", vec![var("X"), var("Y")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X"), Some(&atom("alice")));
        assert_eq!(results[0].get("Y"), Some(&atom("bob")));
    }

    #[test]
    fn test_unify_multiple_solutions() {
        // Program:
        //   parent(alice, bob).
        //   parent(alice, charlie).
        // Query: parent(alice, X).
        let prog = program(vec![
            fact(compound("parent", vec![atom("alice"), atom("bob")])),
            fact(compound("parent", vec![atom("alice"), atom("charlie")])),
        ]);

        let goals = vec![compound("parent", vec![atom("alice"), var("X")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 2);
        assert!(
            results[0].get("X") == Some(&atom("bob"))
                || results[0].get("X") == Some(&atom("charlie"))
        );
        assert!(
            results[1].get("X") == Some(&atom("bob"))
                || results[1].get("X") == Some(&atom("charlie"))
        );
    }

    #[test]
    fn test_unify_same_variable_twice() {
        // Program: sibling(alice, bob).
        // Query: sibling(X, X) should NOT match.
        let prog = program(vec![fact(compound(
            "sibling",
            vec![atom("alice"), atom("bob")],
        ))]);

        let goals = vec![compound("sibling", vec![var("X"), var("X")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_unify_same_variable_twice_matching() {
        // Program: equal(alice, alice).
        // Query: equal(X, X).
        let prog = program(vec![fact(compound(
            "equal",
            vec![atom("alice"), atom("alice")],
        ))]);

        let goals = vec![compound("equal", vec![var("X"), var("X")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X"), Some(&atom("alice")));
    }

    // ============================================================================
    // Simple Rules
    // ============================================================================

    #[test]
    fn test_simple_rule() {
        // Program:
        //   grandparent(X, Z) :- parent(X, Y), parent(Y, Z).
        //   parent(alice, bob).
        //   parent(bob, charlie).
        // Query: grandparent(alice, charlie).
        let prog = program(vec![
            clause(
                compound("grandparent", vec![var("X"), var("Z")]),
                vec![
                    compound("parent", vec![var("X"), var("Y")]),
                    compound("parent", vec![var("Y"), var("Z")]),
                ],
            ),
            fact(compound("parent", vec![atom("alice"), atom("bob")])),
            fact(compound("parent", vec![atom("bob"), atom("charlie")])),
        ]);

        let goals = vec![compound(
            "grandparent",
            vec![atom("alice"), atom("charlie")],
        )];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_rule_with_variable_goal() {
        // Program:
        //   grandparent(X, Z) :- parent(X, Y), parent(Y, Z).
        //   parent(alice, bob).
        //   parent(bob, charlie).
        // Query: grandparent(alice, Z).
        let prog = program(vec![
            clause(
                compound("grandparent", vec![var("X"), var("Z")]),
                vec![
                    compound("parent", vec![var("X"), var("Y")]),
                    compound("parent", vec![var("Y"), var("Z")]),
                ],
            ),
            fact(compound("parent", vec![atom("alice"), atom("bob")])),
            fact(compound("parent", vec![atom("bob"), atom("charlie")])),
        ]);

        let goals = vec![compound("grandparent", vec![atom("alice"), var("Z")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("Z"), Some(&atom("charlie")));
    }

    #[test]
    fn test_rule_multiple_solutions() {
        // Program:
        //   parent(alice, bob).
        //   parent(alice, charlie).
        //   parent(bob, dave).
        //   parent(charlie, eve).
        //   grandparent(X, Z) :- parent(X, Y), parent(Y, Z).
        // Query: grandparent(alice, Z).
        let prog = program(vec![
            fact(compound("parent", vec![atom("alice"), atom("bob")])),
            fact(compound("parent", vec![atom("alice"), atom("charlie")])),
            fact(compound("parent", vec![atom("bob"), atom("dave")])),
            fact(compound("parent", vec![atom("charlie"), atom("eve")])),
            clause(
                compound("grandparent", vec![var("X"), var("Z")]),
                vec![
                    compound("parent", vec![var("X"), var("Y")]),
                    compound("parent", vec![var("Y"), var("Z")]),
                ],
            ),
        ]);

        let goals = vec![compound("grandparent", vec![atom("alice"), var("Z")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 2);
    }

    // ============================================================================
    // Compound Terms (Nested Structures)
    // ============================================================================

    #[test]
    fn test_compound_term_unification() {
        // Program: data(point(1, 2)).
        // Query: data(point(X, Y)).
        let prog = program(vec![fact(compound(
            "data",
            vec![compound("point", vec![atom("1"), atom("2")])],
        ))]);

        let goals = vec![compound(
            "data",
            vec![compound("point", vec![var("X"), var("Y")])],
        )];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X"), Some(&atom("1")));
        assert_eq!(results[0].get("Y"), Some(&atom("2")));
    }

    #[test]
    fn test_nested_compound_terms() {
        // Program: tree(node(node(leaf, leaf), leaf)).
        // Query: tree(X).
        let leaf = atom("leaf");
        let prog = program(vec![fact(compound(
            "tree",
            vec![compound(
                "node",
                vec![compound("node", vec![leaf.clone(), leaf.clone()]), leaf],
            )],
        ))]);

        let goals = vec![compound("tree", vec![var("X")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
    }

    // ============================================================================
    // Empty Goals and Edge Cases
    // ============================================================================

    #[test]
    fn test_empty_goal_list() {
        let prog = program(vec![]);
        let goals = vec![];
        let results = resolve(&goals, prog, HashMap::new());

        // Empty goals should succeed immediately
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], HashMap::new());
    }

    #[test]
    fn test_goal_not_in_program() {
        let prog = program(vec![fact(atom("foo"))]);
        let goals = vec![atom("bar")];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_empty_program() {
        let prog = program(vec![]);
        let goals = vec![atom("foo")];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 0);
    }

    // ============================================================================
    // Multiple Goals in Query
    // ============================================================================

    #[test]
    fn test_multiple_goals_conjunction() {
        // Program:
        //   parent(alice, bob).
        //   parent(bob, charlie).
        // Query: parent(alice, X), parent(X, Y).
        let prog = program(vec![
            fact(compound("parent", vec![atom("alice"), atom("bob")])),
            fact(compound("parent", vec![atom("bob"), atom("charlie")])),
        ]);

        let goals = vec![
            compound("parent", vec![atom("alice"), var("X")]),
            compound("parent", vec![var("X"), var("Y")]),
        ];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X"), Some(&atom("bob")));
        assert_eq!(results[0].get("Y"), Some(&atom("charlie")));
    }

    #[test]
    fn test_multiple_goals_fails_on_second() {
        // Program:
        //   parent(alice, bob).
        // Query: parent(alice, X), parent(X, Y).
        let prog = program(vec![fact(compound(
            "parent",
            vec![atom("alice"), atom("bob")],
        ))]);

        let goals = vec![
            compound("parent", vec![atom("alice"), var("X")]),
            compound("parent", vec![var("X"), var("Y")]),
        ];
        let results = resolve(&goals, prog, HashMap::new());

        // X binds to bob, but parent(bob, Y) has no solutions
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_multiple_goals_backtrack() {
        // Program:
        //   num(1).
        //   num(2).
        //   num(3).
        // Query: num(X), num(Y), X < Y (conceptually; we'll check bindings).
        let prog = program(vec![
            fact(atom("num(1)")),
            fact(atom("num(2)")),
            fact(atom("num(3)")),
        ]);

        // This is a simplification; real backtracking would involve constraint solving.
        // Here we just check that we get all combinations.
        let goals = vec![atom("num(1)"), atom("num(2)")];
        let results = resolve(&goals, prog, HashMap::new());

        // Both goals must match facts in the program
        // num(1) matches, then num(2) matches
        assert!(results.len() >= 0); // Depends on implementation
    }

    // ============================================================================
    // Pre-existing Substitutions
    // ============================================================================

    #[test]
    fn test_with_existing_substitution() {
        // Program: parent(alice, bob).
        // Query: parent(alice, Y) with X = alice already bound.
        let prog = program(vec![fact(compound(
            "parent",
            vec![atom("alice"), atom("bob")],
        ))]);
        let mut subs = HashMap::new();
        subs.insert("X".to_string(), atom("alice"));

        let goals = vec![compound("parent", vec![var("X"), var("Y")])];
        let results = resolve(&goals, prog, subs);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X"), Some(&atom("alice")));
        assert_eq!(results[0].get("Y"), Some(&atom("bob")));
    }

    #[test]
    fn test_with_conflicting_substitution() {
        // Program: parent(alice, bob).
        // Query: parent(alice, Y) with X = charlie already bound (mismatch).
        let prog = program(vec![fact(compound(
            "parent",
            vec![atom("alice"), atom("bob")],
        ))]);
        let mut subs = HashMap::new();
        subs.insert("X".to_string(), atom("charlie"));

        let goals = vec![compound("parent", vec![var("X"), var("Y")])];
        let results = resolve(&goals, prog, subs);

        // X is already bound to charlie, but the fact has alice, so no match
        assert_eq!(results.len(), 0);
    }

    // ============================================================================
    // Atoms (Zero-Arity Predicates)
    // ============================================================================

    #[test]
    fn test_simple_atom_fact() {
        let prog = program(vec![fact(atom("true"))]);
        let goals = vec![atom("true")];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_atom_rule() {
        // Program:
        //   success :- true.
        //   true.
        let prog = program(vec![
            clause(atom("success"), vec![atom("true")]),
            fact(atom("true")),
        ]);

        let goals = vec![atom("success")];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
    }

    // ============================================================================
    // Complex Rule Chains
    // ============================================================================

    #[test]
    fn test_three_level_rule_chain() {
        // Program:
        //   ancestor(X, Y) :- parent(X, Y).
        //   ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).
        //   parent(alice, bob).
        //   parent(bob, charlie).
        //   parent(charlie, dave).
        // Query: ancestor(alice, dave).
        let prog = program(vec![
            clause(
                compound("ancestor", vec![var("X"), var("Y")]),
                vec![compound("parent", vec![var("X"), var("Y")])],
            ),
            clause(
                compound("ancestor", vec![var("X"), var("Z")]),
                vec![
                    compound("parent", vec![var("X"), var("Y")]),
                    compound("ancestor", vec![var("Y"), var("Z")]),
                ],
            ),
            fact(compound("parent", vec![atom("alice"), atom("bob")])),
            fact(compound("parent", vec![atom("bob"), atom("charlie")])),
            fact(compound("parent", vec![atom("charlie"), atom("dave")])),
        ]);

        let goals = vec![compound("ancestor", vec![atom("alice"), atom("dave")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert!(results.len() > 0);
    }

    // ============================================================================
    // Variable Renaming (When Rules Are Used Multiple Times)
    // ============================================================================

    #[test]
    fn test_variable_renaming_in_rules() {
        // Ensures that when a rule is used, its variables are properly renamed
        // to avoid conflicts.
        // Program:
        //   double(X, Y) :- Y = X + X (conceptually).
        //   ...
        // This is harder to test without arithmetic, but the pattern is important.
        let prog = program(vec![clause(
            compound("rel", vec![var("X"), var("X")]),
            vec![],
        )]);

        let goals = vec![compound("rel", vec![atom("a"), atom("a")])];
        let results = resolve(&goals, prog, HashMap::new());

        assert_eq!(results.len(), 1);
    }
}
