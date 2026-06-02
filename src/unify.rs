use std::{collections::HashMap, iter};

use crate::ast::Term;

pub type Substitutions = HashMap<String, Term>;

pub fn apply(substitutions: Substitutions, term: &Term) -> Term {
    match term {
        Term::Atom(_) => term.clone(),
        Term::Var(var) => substitutions.get(var).unwrap_or(term).clone(),
        Term::Compound(functor, terms) => Term::Compound(
            functor.clone(),
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

/// The algorithm proceeds by cases:
///
/// 1. If both terms are the same variable, return the current substitution.
///
/// 2. If one term is a variable V: apply the current substitution to both sides.
///    - If V is now bound, unify the binding with the other term.
///    - Otherwise, check that V does not occur in the other term (occurs check),
///      - then extend the substitution with V → other term.
///
/// 3. If both are atoms with the same name, succeed.
///
/// 4. If both are compound terms with the same functor and arity, unify arguments pairwise.
///
/// 5. Otherwise, fail (return None).
pub fn unify(substitutions: Substitutions, term_1: &Term, term_2: &Term) -> Option<Substitutions> {
    match (term_1, term_2) {
        // 1. If both terms are the same variable, return the current substitution.
        (Term::Var(a), Term::Var(b)) if a == b => Some(substitutions),

        // 2. If one term is a variable V; apply the current substitution to both sides
        //    - If V is now bound, unify the binding with the other term.
        //    - Otherwise, check that V does not occur in the other term (occurs check),
        //      - then extend the substitution with V → other term.
        (v @ Term::Var(_), other) | (other, v @ Term::Var(_)) => {
            // apply substitutions to variable
            let v_updated = apply(substitutions.clone(), v);

            // apply substitutions to other term
            let other_updated = apply(substitutions.clone(), other);

            // v_updated may be either still a Term::Var(_) or it may be something else now like an
            // atom.
            if let Term::Var(v_name) = &v_updated {
                // if it is still a var (i.e. there was no v_name in the substitutions map),
                // let's check if it occurrs anywhere in the other term
                let occurs = var_occurs(v_name, &other_updated);

                // if it doesn't occur, we can substitute v_name -> other_updated
                // (the reason we have the occurs check is because, if v_name occured in
                // other_updated, then this substitution would be recursive)
                if !occurs {
                    // this is a very rusty way of saying substitutions + (v_name, other_updated)
                    Some(
                        substitutions
                            .into_iter()
                            .chain(iter::once((v_name.to_string(), other_updated)))
                            .collect::<HashMap<String, Term>>(),
                    )
                } else {
                    None
                }
            } else {
                // v_updated is no longer a variable and has been substituted
                //
                // so we wanna unify v_updated and other_updated
                unify(substitutions, &v_updated, &other_updated)
            }
        }

        // 3. If both are atoms with the same name, succeed.
        (Term::Atom(a), Term::Atom(b)) if a == b => Some(substitutions),

        // 4. If both are compound terms with the same functor and arity, unify arguments pairwise.
        (Term::Compound(functor_left, args_left), Term::Compound(functor_right, args_right))
            if functor_left == functor_right && args_left.len() == args_right.len() =>
        {
            // iterator yielding pairs of args (arg_left, arg_right)
            let args = args_left.iter().zip(args_right.iter());

            args.fold(
                Some(substitutions),
                |acc, (arg_left, arg_right)| match acc {
                    Some(subst) => unify(subst, arg_left, arg_right),
                    None => None,
                },
            )
        }

        _ => None,
    }
}

pub fn compose(sub1: Substitutions, sub2: Substitutions) -> Substitutions {
    sub1.into_iter()
        .map(|(left, right)| (left, apply(sub2.clone(), &right)))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{ast::Term, unify::unify};

    #[test]
    fn test_unify_f_x_a_with_f_b_y() {
        // f(X, a) with f(b, Y) should yield {X→b, Y→a}
        let t1 = Term::Compound(
            "f".to_string(),
            vec![Term::Var("X".to_string()), Term::Atom("a".to_string())],
        );
        let t2 = Term::Compound(
            "f".to_string(),
            vec![Term::Atom("b".to_string()), Term::Var("Y".to_string())],
        );
        let result = unify(HashMap::new(), &t1, &t2).unwrap();
        assert_eq!(result.get("X"), Some(&Term::Atom("b".to_string())));
        assert_eq!(result.get("Y"), Some(&Term::Atom("a".to_string())));
    }

    #[test]
    fn test_unify_f_x_x_with_f_a_b_fails() {
        // f(X, X) with f(a, b) should fail
        let t1 = Term::Compound(
            "f".to_string(),
            vec![Term::Var("X".to_string()), Term::Var("X".to_string())],
        );
        let t2 = Term::Compound(
            "f".to_string(),
            vec![Term::Atom("a".to_string()), Term::Atom("b".to_string())],
        );
        assert!(unify(HashMap::new(), &t1, &t2).is_none());
    }

    #[test]
    fn test_unify_occurs_check_fails() {
        // f(X) with f(g(X)) should fail due to occurs check
        let t1 = Term::Compound("f".to_string(), vec![Term::Var("X".to_string())]);
        let t2 = Term::Compound(
            "f".to_string(),
            vec![Term::Compound(
                "g".to_string(),
                vec![Term::Var("X".to_string())],
            )],
        );
        assert!(unify(HashMap::new(), &t1, &t2).is_none());
    }
}
