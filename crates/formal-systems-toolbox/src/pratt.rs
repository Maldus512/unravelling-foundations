use super::{Symbol, Judgement};
use nom::IResult;
use nom::combinator::peek;
use std::collections::HashMap;


pub struct BindingPower {
    left: u16,
    right: u16,
}


pub struct PrattParser {
    input: &str, 
    operators: HashMap<Symbol, BindingPower>,
}


impl PrattParser {
    pub fn new(input: &str) {
        Self { input }
    }

    pub fn parse(&mut self, min_binding_power: u16) -> IResult<&str, Judgement> {
        let symbol = self.next()?;
        if self.operators.contains_key(symbol) {
            return Err(nom::error::Error::new(self.input, nom::error::ErrorKind::Fail));
        }
        let mut lhs = Judgement::atom(symbol);

        loop {
            let op = match self.peek() {
                Ok(op) => op,
                Err(nom::Err::Incomplete(_)) => break,
                err => return err,
            };
            if !self.operators.contains_key(op) {
                return Err(nom::error::Error::new(self.input, nom::error::ErrorKind::Fail));
            }
            let binding_power = self.operators.get(op).unwrap();

            if binding_power.left < self.min_binding_power {
                break;
            }

            self.next()?;
            let rhs = self.parse(binding_power.right)?;

            lhs = Judgement { name: op, subjects: vec![lhs, rhs] }
        }

        Ok(lhs)
    }

    pub fn next(&mut self) -> IResult<&str, Symbol>{
        let (remaining, next_symbol) = symbol(self.input)?; 
        self.input = remaining;
        next_symbol
    }

    pub fn peek(&mut self) -> IResult<&str, Symbol>{
        let (_, next_symbol) = peek(symbol)(self.input)?; 
        next_symbol
    }
}
