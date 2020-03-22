use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{alpha1, char, digit1, multispace0, multispace1, one_of},
    combinator::{cut, map, map_res, opt},
    error::{context, convert_error, make_error, ErrorKind, ParseError},
    multi::{many0, many1, separated_list},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Err, IResult,
};

use crate::ast::{Constructor, Data, Expression};
use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

use crate::parse::{basic::sp, expression::parse_atom};

#[inline]
pub fn parse_arithmetic(input: &str) -> IResult<&str, Expression> {
    parse_bottom_precedence(input)
}

pub fn parse_top_precedence(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            tuple((parse_atom, alt((tag("*"), tag("/"))), parse_top_precedence)),
            |(left, symbol, right)| match symbol {
                "*" => Expression::Multiply(Rc::new(left), Rc::new(right)),
                "/" => match right {
                    Expression::Divide(a, b) => {
                        Expression::Divide(Rc::new(Expression::Divide(Rc::new(left), a)), b)
                    }
                    Expression::Multiply(a, b) => {
                        Expression::Multiply(Rc::new(Expression::Divide(Rc::new(left), a)), b)
                    }
                    otherwise => Expression::Divide(Rc::new(left), Rc::new(otherwise)),
                },
                _ => unreachable!(),
            },
        ),
        parse_atom,
    ))(input)
}

pub fn parse_high_precedence(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            tuple((
                parse_top_precedence,
                alt((tag("+"), tag("-"))),
                parse_high_precedence,
            )),
            |(left, symbol, right)| match symbol {
                "+" => Expression::Add(Rc::new(left), Rc::new(right)),
                "-" => match right {
                    Expression::Add(a, b) => {
                        Expression::Add(Rc::new(Expression::Subtract(Rc::new(left), a)), b)
                    }
                    Expression::Subtract(a, b) => {
                        Expression::Subtract(Rc::new(Expression::Subtract(Rc::new(left), a)), b)
                    }
                    otherwise => Expression::Subtract(Rc::new(left), Rc::new(otherwise)),
                },
                _ => unreachable!(),
            },
        ),
        parse_top_precedence,
    ))(input)
}

pub fn parse_mid_precedence(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            tuple((
                parse_high_precedence,
                alt((
                    tag("=="),
                    tag("!="),
                    tag(">="),
                    tag(">"),
                    tag("<="),
                    tag("<"),
                )),
                parse_mid_precedence,
            )),
            |(left, symbol, right)| match symbol {
                "==" => Expression::Equal(Rc::new(left), Rc::new(right)),
                "!=" => Expression::NotEqual(Rc::new(left), Rc::new(right)),
                ">" => Expression::Greater(Rc::new(left), Rc::new(right)),
                ">=" => Expression::GreaterEqual(Rc::new(left), Rc::new(right)),
                "<" => Expression::Less(Rc::new(left), Rc::new(right)),
                "<=" => Expression::LessEqual(Rc::new(left), Rc::new(right)),
                _ => unreachable!(),
            },
        ),
        parse_high_precedence,
    ))(input)
}

pub fn parse_low_precedence(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            separated_pair(parse_mid_precedence, tag("&&"), parse_low_precedence),
            |(left, right)| Expression::And(Rc::new(left), Rc::new(right)),
        ),
        parse_mid_precedence,
    ))(input)
}

pub fn parse_bottom_precedence(input: &str) -> IResult<&str, Expression> {
    alt((
        map(
            separated_pair(parse_low_precedence, tag("||"), parse_bottom_precedence),
            |(left, right)| Expression::Or(Rc::new(left), Rc::new(right)),
        ),
        parse_low_precedence,
    ))(input)
}
