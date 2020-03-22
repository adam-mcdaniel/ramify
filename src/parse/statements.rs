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

use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    ast::{Constant, Constructor, Data, Function, Import, AST},
    parse::{
        basic::{parse_identifier, sp},
        expression::parse_expression,
    },
};

enum Statement {
    Import(Import),
    Constant(Constant),
    Constructor(Constructor),
    Data(Data),
    Function(Function),
}

pub fn parse_ast(input: &str) -> IResult<&str, AST> {
    let (input, _) = sp(input)?;
    // let (input, statements) = many0(terminated(
    //     delimited(
    //         sp,
    //         alt((
    //             map(parse_data_declaration, Statement::Data),
    //             map(parse_type_declaration, Statement::Constructor),
    //             map(parse_function_declaration, Statement::Function),
    //         )),
    //         sp,
    //     ),
    //     tag(";"),
    // ))(input)?;
    let (input, statements) = many0(alt((
        map(parse_import, Statement::Import),
        map(parse_data_declaration, Statement::Data),
        map(parse_type_declaration, Statement::Constructor),
        map(parse_constant_declaration, Statement::Constant),
        map(parse_function_declaration, Statement::Function),
    )))(input)?;
    let (input, _) = sp(input)?;

    let mut imports = Vec::new();
    let mut constants = Vec::new();
    let mut cons = Vec::new();
    let mut data = Vec::new();
    let mut functions = Vec::new();

    for stmt in statements {
        match stmt {
            Statement::Import(i) => imports.push(i),
            Statement::Constant(c) => constants.push(c),
            Statement::Constructor(c) => cons.push(c),
            Statement::Data(d) => data.push(d),
            Statement::Function(f) => functions.push(f),
        }
    }

    let mut ast = AST::new(imports, constants, cons, data, functions);
    ast.resolve_tailcalls();
    ast.replace_constructors();
    ast.resolve_types();
    ast.inline_functions();
    Ok((input, ast))
}

pub fn parse_import(input: &str) -> IResult<&str, Import> {
    let (input, _) = tuple((sp, tag("from"), sp))(input)?;
    let (input, file) = parse_identifier(input)?;
    let (input, _) = tuple((sp, tag("import"), sp))(input)?;
    let (input, items) = separated_list(tag(","), parse_identifier)(input)?;
    let (input, _) = sp(input)?;

    Ok((
        input,
        Import::new(
            file.to_string(),
            items.iter().map(ToString::to_string).collect(),
        ),
    ))
}

pub fn parse_data_declaration(input: &str) -> IResult<&str, Data> {
    let (input, _) = tuple((sp, tag("data"), sp))(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = tuple((sp, tag("="), sp))(input)?;
    let (input, constructors) = separated_list(tag("|"), parse_constructor_declaration)(input)?;
    let (input, _) = sp(input)?;

    Ok((input, Data::new(String::from(name), constructors)))
}

pub fn parse_type_declaration(input: &str) -> IResult<&str, Constructor> {
    let (input, _) = tuple((sp, tag("type"), sp))(input)?;
    let (input, cons) = parse_constructor_declaration(input)?;
    let (input, _) = sp(input)?;

    Ok((input, cons))
}

pub fn parse_constant_declaration(input: &str) -> IResult<&str, Constant> {
    let (input, _) = tuple((sp, tag("const"), sp))(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = tuple((sp, tag("="), sp))(input)?;
    let (input, expr) = parse_expression(input)?;
    let (input, _) = sp(input)?;

    Ok((input, Constant::new(name.to_string(), expr)))
}

pub fn parse_function_declaration(input: &str) -> IResult<&str, Function> {
    let (input, _) = tuple((sp, tag("let"), sp))(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, args) = many1(parse_identifier)(input)?;
    let (input, _) = tuple((sp, tag("="), sp))(input)?;
    let (input, body) = parse_expression(input)?;
    let (input, _) = sp(input)?;
    Ok((
        input,
        Function::new(
            name.to_string(),
            args.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
            body,
        ),
    ))
}

pub fn parse_constructor_declaration(input: &str) -> IResult<&str, Constructor> {
    let (input, _) = sp(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, opt_args) = opt(delimited(
        tuple((sp, tag("("), sp)),
        separated_list(tuple((sp, tag(","), sp)), parse_identifier),
        tuple((sp, tag(")"), sp)),
    ))(input)?;
    let (input, _) = sp(input)?;

    let args = opt_args.unwrap_or(Vec::new());

    Ok((
        input,
        Constructor::new(
            String::from(name),
            args.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        ),
    ))
}
