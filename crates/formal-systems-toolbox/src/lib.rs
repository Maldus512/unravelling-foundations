use std::collections::HashSet;
use std::fmt::Display;

pub mod logic;
pub mod parser;

#[macro_export]
macro_rules! op {
    ($name:expr,$($generic:expr),*) => {
        Judgement::Operator {
            predicate: $name.to_string(),
            subjects: vec![$($generic),*],
        }
    };
    ($name:expr) => { op!($name,) };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Judgement {
    Operator {
        predicate: String,
        subjects: Vec<Judgement>,
    },
    Variable(String),
}

impl Judgement {
    pub fn operator(predicate: &str, subjects: Vec<Judgement>) -> Self {
        Self::Operator {
            predicate: String::from(predicate),
            subjects,
        }
    }

    pub fn variable(name: &str) -> Self {
        Self::Variable(String::from(name))
    }

    pub fn get_variables(&self) -> HashSet<String> {
        use Judgement::*;
        match self {
            Variable(symbol) => HashSet::from([symbol.clone()]),
            Operator {
                predicate: _,
                subjects,
            } => subjects.iter().fold(HashSet::new(), |mut result, subject| {
                result.extend(subject.get_variables());
                result
            }),
        }
    }

    pub fn rename_variables<S>(
        &self,
        state: &mut S,
        operation: &impl Fn(&mut S, String) -> String,
    ) -> Self {
        use Judgement::*;
        match self {
            Variable(symbol) => Variable(operation(state, symbol.clone())),
            Operator {
                predicate,
                subjects,
            } => Operator {
                predicate: predicate.clone(),
                subjects: subjects
                    .iter()
                    .map(|subject| subject.rename_variables(state, operation))
                    .collect(),
            },
        }
    }
}

impl Display for Judgement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Judgement::*;
        match self {
            Variable(symbol) => f.write_str(symbol)?,
            Operator {
                predicate,
                subjects,
            } => {
                f.write_str(&predicate)?;
                f.write_str("(")?;
                for (i, subject) in subjects.iter().enumerate() {
                    f.write_str(format!("{}", subject).as_str())?;
                    if i != subjects.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str(")")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    name: String,
    premises: Vec<Judgement>,
    conclusion: Judgement,
}

impl Rule {
    pub fn new(name: &str, premises: Vec<Judgement>, conclusion: Judgement) -> Self {
        Self {
            name: String::from(name),
            premises,
            conclusion,
        }
    }

    pub fn taut(name: &str, judgement: Judgement) -> Self {
        Self::new(name, vec![], judgement)
    }

    pub fn rename_variables<S>(
        &self,
        state: &mut S,
        operation: &impl Fn(&mut S, String) -> String,
    ) -> Self {
        Self {
            name: self.name.clone(),
            premises: self
                .premises
                .iter()
                .map(|premise| premise.rename_variables(state, operation))
                .collect(),
            conclusion: self.conclusion.rename_variables(state, operation),
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        for (i, premise) in self.premises.iter().enumerate() {
            f.write_str(format!("{}", premise).as_str())?;
            if i != self.premises.len() - 1 {
                f.write_str(", ")?;
            }
        }
        f.write_str(")->")?;
        f.write_str(format!("{}", self.conclusion).as_str())?;

        Ok(())
    }
}

pub fn var(name: &str) -> Judgement {
    Judgement::variable(name)
}

pub fn constant(name: &str) -> Judgement {
    Judgement::operator(name, vec![])
}
