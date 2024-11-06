use nom::character::complete::{alpha1, alphanumeric1};
use nom::bytes::complete::tag;
use nom::sequence::tuple;
use nom::combinator::map;
use nom::multi::{many0, many1, separated_list0};
use nom::branch::alt;
use nom::IResult;


#[derive(Debug, PartialEq)]
pub enum Ast {
    Rule { premises : Vec<Ast>, result : Box<Ast> },
}


pub fn symbol(input: &str) -> IResult<&str, Symbol> {
    map(tuple((alpha1, many0(alphanumeric1))), |(first, rest) : (&str, Vec<&str>)| {
        let mut name = Symbol::from(first);
        for c in rest {
            name.push_str(c);
        }
        name
    })(input)
}

//pub fn judgement(input: &str) -> IResult<&str, Ast> {
    //map(symbol, |sym|  )(input)
//}

pub fn judgement_separator(input: &str) -> IResult<&str, ()> {
    map(tuple((alt((tag("    "), tag("\t"))), many0(alt((tag(" "), tag("\t")))))), |_| ())(input)
}

pub fn rule_bar(input: &str) -> IResult<&str, ()> {
    map(tuple((many1(tag("-")), tag("\n"))), |_| ())(input)
}

//pub fn premises(input: &str) -> IResult<&str, Vec<Ast>> {
    //map(tuple((separated_list0(judgement_separator, judgement), tag("\n"))), |(premises, _): (Vec<Ast>, &str)| premises)(input)
//}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifier() {
        assert_eq!(symbol("x1"), Ok(("", Symbol::from("x1"))));
        assert_eq!(symbol("x"), Ok(("", Symbol::from("x"))));
    }
}
