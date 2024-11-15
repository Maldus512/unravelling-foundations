//mod pratt;
//mod ast;
use std::collections::hash_map::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::iter::zip;

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

pub type UnificationTable = HashMap<String, Judgement>;

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

    pub fn alpha_conversion(
        &self,
        names: &HashSet<String>,
        operation: &impl Fn(String) -> String,
    ) -> Self {
        use Judgement::*;
        match self {
            Variable(symbol) => {
                let mut new_symbol = symbol.clone();

                while names.contains(new_symbol.as_str()) {
                    new_symbol = operation(new_symbol);
                }

                Variable(new_symbol)
            }
            Operator {
                predicate,
                subjects,
            } => Operator {
                predicate: predicate.clone(),
                subjects: subjects
                    .iter()
                    .map(|subject| subject.alpha_conversion(names, operation))
                    .collect(),
            },
        }
    }

    pub fn apply_substitution(&self, substitutions: &UnificationTable) -> Judgement {
        use Judgement::*;
        match self.clone() {
            Variable(symbol) => {
                if let Some(substitution) = substitutions.get(&symbol) {
                    substitution.apply_substitution(substitutions)
                } else {
                    self.clone()
                }
            }
            Operator {
                predicate,
                subjects,
            } => Operator {
                predicate,
                subjects: subjects
                    .iter()
                    .map(|subject| subject.apply_substitution(substitutions))
                    .collect(),
            },
        }
    }

    pub fn variable_occurs_with_substitution(
        &self,
        variable: String,
        substitutions: &UnificationTable,
    ) -> bool {
        use Judgement::*;
        match self {
            Variable(occurrence) => {
                if let Some(substitution) = substitutions.get(occurrence.as_str()) {
                    substitution.variable_occurs_with_substitution(variable, substitutions)
                } else {
                    occurrence.clone() == variable
                }
            }
            Operator {
                predicate: _,
                subjects,
            } => {
                for subject in subjects {
                    if subject.variable_occurs_with_substitution(variable.clone(), substitutions) {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn unify(&self, other: &Judgement) -> Result<UnificationTable, String> {
        let mut substitutions: UnificationTable = HashMap::new();
        self.unify_with_substitution(other, &mut substitutions)?;
        Ok(substitutions)
    }

    fn unify_with_substitution(
        &self,
        other: &Judgement,
        substitutions: &mut UnificationTable,
    ) -> Result<(), String> {
        use Judgement::*;
        //println!("Unifying {} with {}", left, other);
        match (self, other) {
            (Variable(symbol_left), Variable(symbol_right)) if symbol_left == symbol_right => {}
            (judgement, Variable(symbol)) | (Variable(symbol), judgement) => {
                if let Some(substitution) = substitutions.get(&symbol.clone()) {
                    judgement.unify_with_substitution(&substitution.clone(), substitutions)?;
                }

                if judgement.variable_occurs_with_substitution(symbol.clone(), substitutions) {
                    return Err("Recursive unification!".into());
                }
                substitutions.insert(symbol.clone(), judgement.clone());
            }
            (
                Operator {
                    predicate: predicate_left,
                    subjects: subjects_left,
                },
                Operator {
                    predicate: predicate_right,
                    subjects: subjects_right,
                },
            ) => {
                if predicate_left != predicate_right {
                    return Err(format!(
                        "Different predicates: {} != {}",
                        predicate_left, predicate_right,
                    ));
                } else if subjects_left.len() != subjects_right.len() {
                    return Err(format!(
                        "Predicates with different arieties: {} and {}",
                        subjects_left.len(),
                        subjects_right.len()
                    ));
                }

                for (left, right) in zip(subjects_left, subjects_right) {
                    left.unify_with_substitution(right, substitutions)?;
                }
            }
        }

        Ok(())
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

#[derive(Clone)]
pub struct Derivation {
    pub premises: Vec<Derivation>,
    pub conclusion: Judgement,
    pub rule_label: String,
}

impl Derivation {
    pub fn apply_substitution(&self, substitutions: &UnificationTable) -> Self {
        Self {
            premises: self
                .premises
                .iter()
                .map(|premise| premise.apply_substitution(substitutions))
                .collect(),
            conclusion: self.conclusion.apply_substitution(substitutions),
            rule_label: self.rule_label.clone(),
        }
    }

    pub fn pretty_print(&self) -> Vec<String> {
        let mut lines: Vec<String> = vec![];

        let mut premises_results: Vec<Vec<String>> = vec![];
        let mut premises_width: usize = 0;
        let mut max_premise_height: usize = 0;

        let conclusion_string = self.conclusion.to_string();

        let rule_label = self.rule_label.clone();
        let conclusion_width: usize = conclusion_string.len();
        let padded_width = conclusion_width + rule_label.len();

        for (premise, last) in self
            .premises
            .iter()
            .enumerate()
            .map(|(i, el)| (el, i == self.premises.len() - 1))
        {
            let premise_tree = if !last {
                premise
                    .pretty_print()
                    .into_iter()
                    .map(|line| line + "  ")
                    .collect()
            } else {
                premise.pretty_print()
            };

            if premise_tree.len() > max_premise_height {
                max_premise_height = premise_tree.len();
            }

            premises_width += premise_tree.get(0).map(|s| s.len()).unwrap_or(0);
            premises_results.push(premise_tree);
        }

        let max_width = std::cmp::max(premises_width, padded_width);
        let bar_width = std::cmp::max(max_width, conclusion_width + 2);
        let max_width = std::cmp::max(max_width, bar_width + rule_label.len());

        lines.push(format!(
            "{}{: ^width$}",
            " ".repeat(rule_label.len()),
            conclusion_string,
            width = max_width - rule_label.len()
        ));
        lines.push(format!(
            "{}{: ^width$}",
            rule_label,
            ("-".repeat(bar_width)).as_str(),
            width = max_width - rule_label.len()
        ));

        // Merge
        for i in 0..max_premise_height {
            let mut line = String::new();

            for premise_tree in &premises_results {
                if let Some(premise_line) = premise_tree.get(i) {
                    line.push_str(premise_line.as_str());
                } else {
                    line.push_str(" ".repeat(premise_tree.get(0).unwrap().len()).as_str());
                }
            }

            lines.push(format!("{: ^width$}", line, width = max_width));
        }

        lines
    }

    pub fn to_string_tree(&self) -> String {
        let mut lines = self.pretty_print();
        let mut result = String::from("\n");

        lines.reverse();
        for line in &lines {
            result += line;
            result.push_str("\n");
        }

        result
    }
}

#[derive(Clone)]
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

    pub fn alpha_conversion(
        &self,
        names: &HashSet<String>,
        operation: &impl Fn(String) -> String,
    ) -> Self {
        Self {
            name: self.name.clone(),
            premises: self
                .premises
                .iter()
                .map(|premise| premise.alpha_conversion(names, operation))
                .collect(),
            conclusion: self.conclusion.alpha_conversion(names, operation),
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

pub struct FormalSystem {
    axioms: Vec<Rule>,
}

impl FormalSystem {
    pub fn new(axioms: Vec<Rule>) -> Self {
        Self { axioms }
    }

    pub fn verify(&self, judgement: &Judgement) -> Option<Derivation> {
        let mut substitutions = UnificationTable::new();
        self.verify_with_substitutions(&mut substitutions, judgement)
            .map(|derivation| derivation.apply_substitution(&substitutions))
    }

    fn verify_with_substitutions(
        &self,
        substitutions: &mut UnificationTable,
        judgement: &Judgement,
    ) -> Option<Derivation> {
        //println!("Proving {}", judgement);

        for axiom in &self.axioms {
            //println!("Trying {}", axiom);

            let mut variables = judgement.get_variables();
            variables.extend(substitutions.clone().into_keys());

            let axiom = axiom.alpha_conversion(&variables, &|symbol| symbol + "'");

            let mut unification_substitutions = substitutions.clone();

            match judgement
                .unify_with_substitution(&axiom.conclusion, &mut unification_substitutions)
            {
                Ok(_) => {
                    substitutions.extend(unification_substitutions);

                    /*
                    println!(
                        "Matched {} ({}), recursion...",
                        axiom.conclusion.apply_substitution(&substitutions),
                        axiom
                    );

                    for (from, to) in substitutions.iter() {
                        println!("{} -> {}", from, to);
                    }
                    */

                    let mut premises: Vec<Derivation> = vec![];

                    for premise in &axiom.premises {
                        //println!("Recursing premise {}", premise);
                        let premise_proof = self.verify_with_substitutions(substitutions, &premise);

                        if let Some(proof) = premise_proof {
                            premises.push(proof);
                        } else {
                            return Option::None;
                        }
                    }

                    //println!("Proved {}!", judgement);
                    return Option::Some(Derivation {
                        premises,
                        conclusion: judgement.clone(),
                        rule_label: axiom.name.clone(),
                    });
                }
                Err(_e) => {
                    //println!("No dice {}!", e);
                }
            }
        }

        return Option::None;
    }
}

pub fn var(name: &str) -> Judgement {
    Judgement::variable(name)
}

pub fn atom(name: &str) -> Judgement {
    Judgement::operator(name, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_unification() {
        let x = var("x");
        let y = var("y");

        let unification = x.unify(&y);
        assert!(unification.is_ok());
        let unification = unification.unwrap();

        let unified_left = x.apply_substitution(&unification);
        let unified_right = y.apply_substitution(&unification);

        assert_eq!(unified_left, unified_right);
        assert_eq!(unified_left, var("x"));
        assert_eq!(unified_right, var("x"));
    }

    #[test]
    fn operator_unification() {
        let left = op!("succ", var("n"));
        let right = op!("succ", op!("succ", var("m")));

        let unification = left.unify(&right);
        assert!(unification.is_ok());
        let unification = unification.unwrap();

        let unified_left = left.apply_substitution(&unification);
        let unified_right = right.apply_substitution(&unification);

        assert_eq!(unified_left, unified_right);

        let n_substitution = unification.get("n".into());
        assert!(n_substitution.is_some());

        let n_substitution = n_substitution.unwrap();
        assert_eq!(n_substitution.clone(), op!("succ", var("m")));
    }

    #[test]
    fn infinite_unification() {
        let left = var("n");
        let right = op!("succ", var("n"));

        let unification = left.unify(&right);
        assert!(unification.is_err());
    }

    #[test]
    fn nat_formal_system() {
        let nat = FormalSystem::new(vec![
            Rule::new(
                "succ",
                vec![op!("nat", var("n"))],
                op!("nat", op!("succ", var("n"))),
            ),
            Rule::taut("zero", op!("nat", atom("zero"))),
        ]);

        let proof = nat.verify(&op!("nat", atom("zero")));
        assert!(proof.is_some());
    }
}
