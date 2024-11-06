//mod pratt;
//mod ast;
use std::collections::hash_map::HashMap;
use std::sync::Arc;
use std::iter::zip;

macro_rules! var {
    ($i:expr) => {
        Judgement::Variable(String::from($i))
    };
}


macro_rules! op {
    ($name:expr,$($generic:expr)*) => {
        Judgement::Operator {
            predicate: $name.to_string(),
            subjects: vec![$($generic),*],
        }
    };
    ($name:expr) => { op!($name,) };
}


pub type Symbol = String;
pub type UnificationTable = HashMap<Symbol, Arc<Judgement>>;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Judgement {
    Operator {predicate: Symbol, subjects: Vec<Judgement>},
    Variable(Symbol),
}


impl Judgement {
    pub fn operator(predicate: &str, subjects: Vec<Judgement>) -> Self {
        Self::Operator { predicate: String::from(predicate), subjects }
    }

    pub fn atom(name: &str) -> Self {
        Self::Operator { predicate : String::from(name), subjects : vec![] }
    }

    pub fn variable(name: &str) -> Self {
        Self::Variable(String::from(name))
    }
}


pub fn unify(left: &Judgement, right: &Judgement) -> Result<UnificationTable, String> {
    let mut substitutions: UnificationTable = HashMap::new();
    unify_substitution(Arc::new(left.clone()), Arc::new(right.clone()), &mut substitutions)?;
    Ok(substitutions)
}

pub fn unify_substitution(left: Arc<Judgement>, right: Arc<Judgement>, substitutions: &mut UnificationTable) -> Result<(), String> {
    use Judgement::*;
    match (left.as_ref(), right.as_ref()) {
        (Variable(symbol_left), Variable(symbol_right)) if symbol_left == symbol_right => { },
        (_, Variable(symbol)) => {
            if let Some(substitution) = substitutions.get(symbol) {
                unify_substitution(left.clone(), substitution.clone(), substitutions)?;
            }

            //TODO: check for recursive types
            substitutions.insert(symbol.clone(), left);
        }
        (Variable(symbol), _) => {
            if let Some(substitution) = substitutions.get(symbol) {
                unify_substitution(substitution.clone(), right.clone(), substitutions)?;
            }

            //TODO: check for recursive types
            substitutions.insert(symbol.clone(), right);
        }
        (Operator { predicate: predicate_left, subjects: subjects_left } , Operator { predicate: predicate_right, subjects: subjects_right}) => {
            if subjects_left.len() != subjects_right.len() {
                return Err(format!("Predicates {:?} and {:?} have different arieties: {} and {}", predicate_left, predicate_right, subjects_left.len(), subjects_right.len()));
            }

            for (left, right) in zip(subjects_left, subjects_right) {
              unify_substitution(Arc::new(left.clone()), Arc::new(right.clone()), substitutions)?;
            }
        }
    }

    Ok(())
}


pub fn substitute(judgement: &Judgement, substitutions: &UnificationTable) -> Judgement {
    use Judgement::*;
    match judgement.clone() {
        Variable(symbol) => {
            if let Some(substitution) = substitutions.get(&symbol) {
                substitute(substitution.as_ref(), substitutions)
            } else {
                judgement.clone()
            }
        },
        Operator { predicate, subjects } => {
            Operator { predicate, subjects: subjects.iter().map(|subject| substitute(subject, substitutions)).collect() }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_unification() {
        let x = var!("x");
        let y = var!("y");

        let unification = unify(&x, &y);
        assert!(unification.is_ok());
        let unification = unification.unwrap();

        let unified_left = substitute(&x, &unification);
        let unified_right = substitute(&y, &unification);

        assert_eq!(unified_left, unified_right);
        assert_eq!(unified_left, var!("x"));
        assert_eq!(unified_right, var!("x"));
    }

    #[test]
    fn operator_unification() {
        let left = op!("succ", var!("n"));
        let right = op!("succ", op!("succ", var!("m")));

        let unification = unify(&left, &right);
        assert!(unification.is_ok());
        let unification = unification.unwrap();

        let unified_left = substitute(&left, &unification);
        let unified_right = substitute(&right, &unification);

        assert_eq!(unified_left, unified_right);

        let n_substitution = unification.get("n".into());
        assert!(n_substitution.is_some());

        let n_substitution = n_substitution.unwrap();
        assert_eq!(n_substitution.as_ref().clone(), op!("succ", var!("m")));
    }

    #[test]
    fn infinite_unification() {
        let left = var!("n");
        let right = op!("succ", var!("n"));

        let unification = unify(&left, &right);
        assert!(unification.is_err());
    }
}
