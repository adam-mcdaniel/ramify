use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    error::{make_error, ErrorKind},
    IResult,
};

pub(crate) fn sp(input: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";

    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| chars.contains(c))(input)
}

pub(crate) fn parse_keyword(input: &str) -> IResult<&str, &str> {
    let (input, _) = sp(input)?;
    alt((
        tag("const"),
        tag("from"),
        tag("import"),
        tag("data"),
        tag("type"),
        tag("let"),
        tag("in"),
        tag("case"),
        tag("of"),
        tag("rec"),
        tag("if"),
        tag("then"),
        tag("else"),
    ))(input)
}

pub(crate) fn parse_identifier(input: &str) -> IResult<&str, &str> {
    let (input, _) = sp(input)?;

    let is_ident_ch = |c: char| c == '_' || c.is_alphanumeric();

    // Check to see if a keyword can be extracted from input
    if let Ok((i, keyword)) = parse_keyword(input) {
        // If the remaining input is empty
        let next_char_is_ident = i.chars().nth(0).map_or(false, is_ident_ch);

        if i.is_empty() || !next_char_is_ident {
            return Err(nom::Err::Error(make_error(keyword, ErrorKind::Tag)));
        }
    }

    let (input, parsed) = take_while1(is_ident_ch)(input)?;

    if parsed.chars().nth(0).unwrap().is_digit(10) {
        Err(nom::Err::Error(make_error(parsed, ErrorKind::Digit)))
    } else {
        Ok((input, parsed))
    }
}

pub(crate) fn parse_string(input: &str) -> IResult<&str, &str> {
    let mut end = 0;
    let mut skip_next = false;
    for (n, ch) in input.chars().enumerate() {
        end = n + 1;
        if skip_next {
            continue;
        }
        if n == 0 {
            if ch != '"' {
                return Err(nom::Err::Error(make_error(input, ErrorKind::Tag)));
            }
            continue;
        }

        match ch {
            '"' => break,
            '\\' => {
                if input.chars().nth(n + 1).unwrap() == '"' {
                    skip_next = true;
                }
            }
            _ => {}
        }
    }

    let quote_count = input[0..end].matches('"').count() - input[0..end].matches("\\\"").count();
    if quote_count == 2 {
        Ok((&input[end..], &input[0..end]))
    } else if quote_count > 0 {
        return Err(nom::Err::Error(make_error(input, ErrorKind::Escaped)));
    } else {
        return Err(nom::Err::Error(make_error(input, ErrorKind::Eof)));
    }
}
