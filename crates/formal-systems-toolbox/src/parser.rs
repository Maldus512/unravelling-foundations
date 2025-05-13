use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::{map, opt, peek};
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use crate::{Judgement, Rule};

pub enum Ast {
    Rule(Rule),
    Judgement(Judgement),
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: Fn(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn symbol(input: &str) -> IResult<&str, String> {
    map(
        tuple((alpha1, many0(alt((alphanumeric1, tag("'")))))),
        |(first, rest): (&str, Vec<&str>)| {
            let mut name = String::from(first);
            for c in rest {
                name.push_str(c);
            }
            name
        },
    )(input)
}

pub fn predicate(input: &str) -> IResult<&str, Judgement> {
    alt((
        map(
            tuple((
                symbol,
                tag("("),
                multispace0,
                many0(terminated(predicate, opt(ws(tag(","))))),
                ws(tag(")")),
            )),
            |(predicate, _, _, subjects, _)| Judgement::Operator {
                predicate: predicate.clone(),
                subjects: subjects.iter().map(|sym| sym.clone()).collect(),
            },
        ),
        map(symbol, |sym| Judgement::Variable(sym)),
    ))(input)
}

pub fn rule(input: &str) -> IResult<&str, Rule> {
    fn bar(input: &str) -> IResult<&str, String> {
        map(
            tuple((
                take_until::<&str, &str, nom::error::Error<&str>>("-"),
                many1(tag("-")),
                tag("\n"),
            )),
            |(name, _, _)| String::from(name),
        )(input)
    }

    fn premises(input: &str) -> IResult<&str, Vec<Judgement>> {
        fn judgement_separator(input: &str) -> IResult<&str, ()> {
            map(
                tuple((
                    alt((tag("    "), tag("\t"))),
                    many0(alt((tag(" "), tag("\t")))),
                )),
                |_| (),
            )(input)
        }

        map(
            tuple((separated_list0(judgement_separator, predicate), tag("\n"))),
            |(premises, _): (Vec<Judgement>, &str)| premises,
        )(input)
    }

    map(
        tuple((opt(tuple((premises, tag("\n")))), bar, predicate)),
        |(premises, name, conclusion)| {
            Rule::new(
                name.as_str(),
                premises.map(|(premises, _)| premises).unwrap_or(vec![]),
                conclusion,
            )
        },
    )(input)
}

pub struct BindingPower {
    left: u16,
    right: u16,
}

pub fn judgement<'a>(
    operators: &HashMap<String, BindingPower>,
    min_binding_power: u16,
    mut input: &'a str,
) -> IResult<&'a str, Judgement> {
    let (remaining, symbol) = next_operator(input)?;

    if operators.contains_key(&symbol) {
        return Err(nom::Err::Error(nom::error::Error::new(
            remaining,
            nom::error::ErrorKind::Fail,
        )));
    }
    let mut lhs = Judgement::operator(symbol.as_str(), vec![]);
    input = remaining;

    loop {
        let (remaining, op) = match peek_operator(input) {
            Ok(op) => op,
            Err(nom::Err::Incomplete(_)) => break,
            Err(err) => return Err(err),
        };
        if !operators.contains_key(&op) {
            return Err(nom::Err::Error(nom::error::Error::new(
                remaining,
                nom::error::ErrorKind::Fail,
            )));
        }
        let binding_power = operators.get(&op).unwrap();

        if binding_power.left < min_binding_power {
            break;
        }

        let (remaining, _) = next_operator(remaining)?;
        let (_, rhs) = judgement(operators, binding_power.right, remaining)?;

        lhs = Judgement::Operator {
            predicate: op,
            subjects: vec![lhs, rhs],
        };

        input = remaining;
    }

    Ok((input, lhs))
}

fn next_operator<'a>(input: &'a str) -> IResult<&'a str, String> {
    let (remaining, next_symbol) = symbol(input)?;
    Ok((remaining, next_symbol))
}

fn peek_operator<'a>(input: &'a str) -> IResult<&'a str, String> {
    let (remaining, next_symbol) = peek(symbol)(input)?;
    Ok((remaining, next_symbol))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::op;

    #[test]
    fn parse_identifier() {
        assert_eq!(symbol("x1"), Ok(("", String::from("x1"))));
        assert_eq!(symbol("x'''"), Ok(("", String::from("x'''"))));
    }

    #[test]
    fn parse_judgement() {
        let max = op!("max", op!("succ", op!("zero")), Judgement::variable("x"));
        assert_eq!(predicate(max.to_string().as_str()), Ok(("", max.clone())));
    }

    #[test]
    fn parse_rule() {
        {
            let rule_str = "nat1----\nnat(n)";
            assert_eq!(
                rule(rule_str),
                Ok(("", Rule::taut("nat1", op!("nat", Judgement::variable("n")))))
            );
        }

        {
            let rule_str = "nat(n)\nnat2----\nnat(succ(n))";
            assert_eq!(
                rule(rule_str),
                Ok((
                    "",
                    Rule::new(
                        "nat2",
                        vec![op!("nat", Judgement::variable("n"))],
                        op!("nat", op!("succ", Judgement::variable("n")))
                    )
                ))
            );
        }
    }
}
