use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{alpha1, char, digit1, multispace0, multispace1, one_of},
    combinator::{cut, map, map_res, opt},
    error::{context, convert_error, make_error, ErrorKind, ParseError},
    multi::{many0, many1, separated_list},
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Err, IResult,
};

use crate::ast::{Constructor, Data, Expression};
use alloc::{
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

use crate::parse::{
    arithmetic::parse_arithmetic,
    basic::{parse_identifier, parse_string, sp},
    lambda::{parse_abstraction, parse_application, parse_tailcall},
    statements::parse_constructor_declaration,
};

pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    if input == "" {
        return Err(nom::Err::Failure(make_error(input, ErrorKind::Eof)));
    }
    if &input[0..1] == ")" {
        return Err(nom::Err::Failure(make_error(input, ErrorKind::Tag)));
    }

    let (input, _) = sp(input)?;
    alt((
        parse_if_then_else,
        parse_case_of,
        parse_deconstruct,
        parse_abstraction,
        parse_tailcall,
        parse_application,
        parse_arithmetic,
        parse_atom,
        delimited(
            tuple((sp, tag("("), sp)),
            parse_expression,
            tuple((sp, tag(")"), sp)),
        ),
    ))(input)
}

pub(crate) fn parse_atom(input: &str) -> IResult<&str, Expression> {
    let (input, result) = delimited(
        sp,
        alt((
            map(
                tuple((alt((tag("!"), tag("-"))), delimited(sp, parse_atom, sp))),
                |(symbol, value)| match symbol {
                    "!" => Expression::Not(Rc::new(value)),
                    "-" => Expression::Negate(Rc::new(value)),
                    _ => unreachable!(),
                },
            ),
            parse_list,
            parse_table,
            map(double, Expression::Number),
            map(parse_string, |s| {
                Expression::String(s[1..s.len() - 1].to_string())
            }),
            parse_abstraction,
            parse_constructor_instance,
            delimited(
                sp,
                map(parse_identifier, |var| {
                    Expression::Identifier(String::from(var))
                }),
                sp,
            ),
            delimited(
                tuple((sp, tag("("), sp)),
                parse_expression,
                tuple((sp, tag(")"), sp)),
            ),
        )),
        sp,
    )(input)?;

    Ok((input, result))
}

pub(crate) fn parse_if_then_else(input: &str) -> IResult<&str, Expression> {
    let (input, _) = tuple((sp, tag("if"), sp))(input)?;
    let (input, c) = parse_expression(input)?;
    let (input, _) = tuple((sp, tag("then"), sp))(input)?;
    let (input, a) = parse_expression(input)?;
    let (input, _) = tuple((sp, tag("else"), sp))(input)?;
    let (input, b) = parse_expression(input)?;

    Ok((
        input,
        Expression::IfThenElse {
            condition: Rc::new(c),
            then_case: Rc::new(a),
            else_case: Rc::new(b),
        },
    ))
}

pub(crate) fn parse_list(input: &str) -> IResult<&str, Expression> {
    let (input, _) = tuple((sp, tag("["), sp))(input)?;
    let (input, values) = separated_list(tuple((sp, tag(","), sp)), parse_atom)(input)?;
    let (input, _) = tuple((sp, tag("]"), sp))(input)?;

    Ok((
        input,
        Expression::List(values.iter().map(|c| Rc::new(c.clone())).collect()),
    ))
}

pub(crate) fn parse_table(input: &str) -> IResult<&str, Expression> {
    let (input, _) = tuple((sp, tag("{"), sp))(input)?;
    let (input, values) = separated_list(
        tuple((sp, tag(","), sp)),
        separated_pair(parse_string, tuple((sp, tag(":"), sp)), parse_atom),
    )(input)?;
    let (input, _) = tuple((sp, tag("}"), sp))(input)?;

    let mut table = BTreeMap::new();
    for (k, v) in values {
        table.insert(k.to_string(), Rc::new(v.clone()));
    }

    Ok((input, Expression::Table(table)))
}

pub(crate) fn parse_case_of(input: &str) -> IResult<&str, Expression> {
    let (input, _) = tuple((sp, tag("case"), sp))(input)?;
    let (input, value) = parse_expression(input)?;
    let (input, _) = tuple((sp, tag("of"), sp))(input)?;
    let (input, cases) = many1(preceded(
        tag("|"),
        separated_pair(parse_constructor_declaration, tag("=>"), parse_expression),
    ))(input)?;

    Ok((
        input,
        Expression::CaseOf {
            data_type: None,
            value: Rc::new(value),
            cases: cases
                .iter()
                .map(|(cons, expr)| {
                    (
                        cons.get_name().to_string(),
                        cons.get_members().clone(),
                        Rc::new(expr.clone()),
                    )
                })
                .collect::<Vec<(String, Vec<String>, Rc<Expression>)>>(),
        },
    ))
    // // case input "> " of
    // //  | Ok(val) => print "You said: " val
    // //  | Err(_)  => print "There was a problem retrieving input"
    // CaseOf {
    //     /// This field is optional because the data type is derived from
    //     /// the constructors. Likely, the first compiler pass will not
    //     /// be able to determine the data type of the constructor cases.
    //     data_type: Option<Data>,
    //     /// The value to scrutinize
    //     value: Rc<Self>,
    //     /// Constructor name, constructor arguments, body of case
    //     ///
    //     /// Name Arguments Body of case
    //     ///  |     |         |
    //     ///  v v---/         v
    //     /// Ok(x) => print "You said: " x
    //     cases: Vec<(String, Vec<String>, Rc<Self>)>,
    // },

    // let (input, cons) = parse_constructor_declaration(input)?;
    // let (input, _) = tuple((sp, tag("in"), sp))(input)?;
    // let (input, body) = parse_expression(input)?;
    // let cons_name = cons.get_name().to_string();
    // let members = cons.get_members().clone();
    // Ok((
    //     input,
    //     Expression::Deconstruct {
    //         data_type: None,
    //         cons_name,
    //         members,
    //         value: Rc::new(value),
    //         body: Rc::new(body),
    //     },
    // ))
}

pub(crate) fn parse_deconstruct(input: &str) -> IResult<&str, Expression> {
    let (input, _) = tuple((sp, tag("let"), sp))(input)?;
    let (input, cons) = parse_constructor_declaration(input)?;
    let (input, _) = tuple((sp, tag("="), sp))(input)?;
    let (input, value) = parse_expression(input)?;
    let (input, _) = tuple((sp, tag("in"), sp))(input)?;
    let (input, body) = parse_expression(input)?;
    let cons_name = cons.get_name().to_string();
    let members = cons.get_members().clone();
    Ok((
        input,
        Expression::Deconstruct {
            data_type: None,
            cons_name,
            members,
            value: Rc::new(value),
            body: Rc::new(body),
        },
    ))
}

pub(crate) fn parse_constructor_instance(input: &str) -> IResult<&str, Expression> {
    let (input, name) = parse_identifier(input)?;
    let (input, args) = delimited(
        tuple((sp, tag("("), sp)),
        separated_list(tuple((sp, tag(","), sp)), parse_expression),
        tuple((sp, tag(")"), sp)),
    )(input)?;

    if args.len() <= 1 {
        return Err(nom::Err::Error(make_error(input, ErrorKind::Many1)));
    }

    Ok((
        input,
        Expression::Construct {
            data_type: None,
            cons_name: String::from(name),
            members: args
                .iter()
                .map(|e| Rc::new(e.clone()))
                .collect::<Vec<Rc<Expression>>>(),
        },
    ))
}
