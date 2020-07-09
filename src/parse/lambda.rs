use nom::{
    branch::alt, bytes::complete::tag, combinator::map, multi::many1, sequence::separated_pair,
    IResult,
};

use crate::ast::Expression;
use crate::parse::{
    basic::{parse_identifier, sp},
    expression::{parse_atom, parse_expression},
};
use alloc::{rc::Rc, string::String};

pub(crate) fn parse_tailcall(input: &str) -> IResult<&str, Expression> {
    // println!("parse_application: {:?}", input);
    let (input, _) = sp(input)?;
    let (input, _) = tag("rec")(input)?;

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
