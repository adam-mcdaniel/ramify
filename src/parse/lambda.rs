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
use crate::parse::{
    basic::{parse_identifier, sp},
    expression::{parse_atom, parse_expression},
};
use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

pub(crate) fn parse_tailcall(input: &str) -> IResult<&str, Expression> {
    // println!("parse_application: {:?}", input);
    let (input, _) = sp(input)?;
    let (input, mut f) = tag("rec")(input)?;

    let (input, args) = many1(alt((parse_atom, parse_abstraction)))(input)?;

    Ok((
        input,
        Expression::TailCall(args.iter().map(|c| Rc::new(c.clone())).collect()),
    ))
}

pub(crate) fn parse_application(input: &str) -> IResult<&str, Expression> {
    // println!("parse_application: {:?}", input);
    let (input, _) = sp(input)?;
    let (input, mut f) = alt((parse_atom, parse_application))(input)?;

    let (input, args) = many1(alt((parse_atom, parse_abstraction)))(input)?;

    for arg in args {
        f = Expression::Application(Rc::new(f), Rc::new(arg));
    }

    Ok((input, f))
}

pub(crate) fn parse_abstraction(input: &str) -> IResult<&str, Expression> {
    // println!("parse_abstraction: {:?}", input);
    let (input, _) = sp(input)?;
    map(
        separated_pair(parse_identifier, tag("."), parse_expression),
        |(var, expr)| Expression::Lambda(String::from(var), Rc::new(expr)),
    )(input)
}
