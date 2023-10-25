use std::str::FromStr;

use nom::combinator::peek;
use nom::sequence::terminated;
use nom::{
    bytes::complete::take_until1,
    character::complete::space1,
    sequence::{delimited, preceded, separated_pair},
    Finish, IResult, Parser,
};

use crate::ResultItem;

/// Parses a result line from a String into some target type.
/// The target type can be something like a `HashMap<&str, ResultItem>` or a `Vec<(&str, ResultItem)>`.
///
/// # Arguments
///
/// * `input`: The input result line to parse.
///
/// Returns: The data stored in the result line in form of the target type.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use serde_result_line::ResultItem;
///
/// let s = r#"RESULT a="some value" "a key"=12315 c=true"#;
///
/// let map: HashMap<&str, ResultItem> = serde_result_line::from_string(s).unwrap();
/// let mut expected = HashMap::<&str, ResultItem>::new();
/// expected.insert("a", ResultItem::Text("some value".to_owned()));
/// expected.insert("a key", ResultItem::Integer(12315));
/// expected.insert("c", ResultItem::Boolean(true));
/// ```
pub fn from_string<'a, Target>(input: &'a str) -> Result<Target, nom::error::Error<&str>>
where
    Target: FromIterator<(&'a str, ResultItem)>,
{
    parse_result_line::<Target>(input)
        .finish()
        .map(|(_, target)| target)
}

fn parse_delimited_string<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    delimited(
        nom::character::complete::char('"'),
        take_until1("\""),
        nom::character::complete::char('"'),
    )
}

fn parse_key<'a>() -> impl Parser<&'a str, &'a str, nom::error::Error<&'a str>> {
    nom::branch::alt((parse_delimited_string(), take_until1("=")))
}

fn parse_value(input: &str) -> IResult<&str, ResultItem> {
    use nom::branch::alt;
    let tag = nom::bytes::complete::tag::<&str, &str, nom::error::Error<&str>>;
    let mut parser = alt((
        parse_delimited_string().map(ResultItem::from),
        alt((tag("true"), tag("false")))
            .map(|s| bool::from_str(s).unwrap())
            .map(ResultItem::from),
        terminated(nom::character::complete::i64, peek(space1))
            .map(|i| i as isize)
            .map(ResultItem::from),
        nom::number::complete::double.map(ResultItem::from),
        parse_key().map(ResultItem::from),
    ));

    parser.parse(input)
}

fn parse_named_item(input: &str) -> IResult<&str, (&str, ResultItem)> {
    separated_pair(
        parse_key(),
        nom::character::complete::char('='),
        parse_value,
    )
    .parse(input)
}

fn parse_result_line<'a, Target>(input: &'a str) -> IResult<&'a str, Target>
where
    Target: FromIterator<(&'a str, ResultItem)>,
{
    let tag = nom::bytes::complete::tag::<&str, &str, nom::error::Error<&str>>;
    let (mut input, _) = tag("RESULT")(input)?;
    let mut named_item_parser = preceded(space1, parse_named_item);
    // Create an iterator parsing all item pairs
    let pairs = std::iter::from_fn(|| match named_item_parser(input) {
        Ok((rest, pair)) => {
            input = rest;
            Some(pair)
        }
        Err(_) => None,
    });

    // Collect them into the target type
    let target: Target = pairs.collect();

    Ok((input, target))
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::ResultItem;

    #[test]
    fn test() {
        const S: &str =
            r#"RESULT a="hello world" b=-123423904 "a key"=8123 nowhitespace=8123.23 d=true"#;
        let map = super::parse_result_line::<HashMap<&str, ResultItem>>(S).map(|(_, map)| map);

        let mut expected = HashMap::<&str, ResultItem>::new();
        expected.insert("a", ResultItem::Text("hello world".to_owned()));
        expected.insert("b", ResultItem::Integer(-123423904));
        expected.insert("a key", ResultItem::Integer(8123));
        expected.insert("nowhitespace", ResultItem::Float(8123.23));
        expected.insert("d", ResultItem::Boolean(true));

        assert_eq!(Ok(expected), map, "Parsed map does not match expected map");
    }
}
