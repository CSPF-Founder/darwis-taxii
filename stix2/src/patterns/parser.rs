//! STIX Pattern Parser
//!
//! This module implements a parser for STIX pattern language expressions.

use super::{ComparisonExpression, ComparisonOperator, PatternExpression, PatternValue, Qualifier};
use crate::core::error::{Error, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{escaped, tag, tag_no_case, take_while, take_while1},
    character::complete::{char, digit1, multispace0, none_of, one_of},
    combinator::{map_res, opt, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated},
};

/// Parser for STIX patterns.
pub struct PatternParser;

impl PatternParser {
    /// Parse a STIX pattern string.
    pub fn parse(input: &str) -> Result<PatternExpression> {
        parse_pattern(input)
    }
}

/// Parse a complete STIX pattern.
pub fn parse_pattern(input: &str) -> Result<PatternExpression> {
    let input = input.trim();

    match parse_observation_expression(input) {
        Ok((remaining, expr)) if remaining.trim().is_empty() => Ok(expr),
        Ok((remaining, _)) => Err(Error::PatternParse(format!(
            "Unexpected input remaining: {}",
            remaining
        ))),
        Err(e) => Err(Error::PatternParse(format!("Parse error: {:?}", e))),
    }
}

// Observation expression (top-level)
fn parse_observation_expression(input: &str) -> IResult<&str, PatternExpression> {
    let (input, expr) = parse_or_expression(input)?;
    Ok((input, expr))
}

// OR expression
fn parse_or_expression(input: &str) -> IResult<&str, PatternExpression> {
    let (input, first) = parse_and_expression(input)?;
    let (input, rest) = many0(preceded(
        (multispace0, tag_no_case("OR"), multispace0),
        parse_and_expression,
    ))
    .parse(input)?;

    let result = rest.into_iter().fold(first, |acc, expr| {
        PatternExpression::Or(Box::new(acc), Box::new(expr))
    });

    Ok((input, result))
}

// AND expression
fn parse_and_expression(input: &str) -> IResult<&str, PatternExpression> {
    let (input, first) = parse_followedby_expression(input)?;
    let (input, rest) = many0(preceded(
        (multispace0, tag_no_case("AND"), multispace0),
        parse_followedby_expression,
    ))
    .parse(input)?;

    let result = rest.into_iter().fold(first, |acc, expr| {
        PatternExpression::And(Box::new(acc), Box::new(expr))
    });

    Ok((input, result))
}

// FOLLOWEDBY expression
fn parse_followedby_expression(input: &str) -> IResult<&str, PatternExpression> {
    let (input, first) = parse_qualified_expression(input)?;
    let (input, rest) = many0(preceded(
        (multispace0, tag_no_case("FOLLOWEDBY"), multispace0),
        parse_qualified_expression,
    ))
    .parse(input)?;

    let result = rest.into_iter().fold(first, |acc, expr| {
        PatternExpression::FollowedBy(Box::new(acc), Box::new(expr))
    });

    Ok((input, result))
}

// Qualified expression (with WITHIN, REPEATS, etc.)
fn parse_qualified_expression(input: &str) -> IResult<&str, PatternExpression> {
    let (input, expr) = parse_primary_expression(input)?;
    let (input, _) = multispace0(input)?;
    let (input, qualifier) = opt(parse_qualifier).parse(input)?;

    match qualifier {
        Some(q) => Ok((input, PatternExpression::Qualified(Box::new(expr), q))),
        None => Ok((input, expr)),
    }
}

// Primary expression (observation or parenthesized)
fn parse_primary_expression(input: &str) -> IResult<&str, PatternExpression> {
    alt((
        parse_observation,
        delimited(
            (char('('), multispace0),
            parse_observation_expression,
            (multispace0, char(')')),
        ),
    ))
    .parse(input)
}

// Single observation [...]
fn parse_observation(input: &str) -> IResult<&str, PatternExpression> {
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, comparison) = parse_comparison_expression(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, PatternExpression::Comparison(comparison)))
}

// Comparison expression: object_type:object_path operator value
fn parse_comparison_expression(input: &str) -> IResult<&str, ComparisonExpression> {
    let (input, negated) = opt(terminated(tag_no_case("NOT"), multispace0)).parse(input)?;

    let (input, object_type) = parse_object_type(input)?;
    let (input, _) = char(':')(input)?;
    let (input, object_path) = parse_object_path(input)?;
    let (input, _) = multispace0(input)?;
    let (input, operator) = parse_comparison_operator(input)?;
    let (input, _) = multispace0(input)?;
    let (input, value) = parse_value(input)?;

    Ok((
        input,
        ComparisonExpression {
            object_type: object_type.to_string(),
            object_path: object_path.to_string(),
            operator,
            value,
            negated: negated.is_some(),
        },
    ))
}

// Object type (e.g., "file", "ipv4-addr")
fn parse_object_type(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)
}

// Object path (e.g., "value", "hashes.'SHA-256'")
fn parse_object_path(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        many0(alt((
            recognize(pair(char('.'), parse_path_component)),
            recognize(delimited(char('['), take_while(|c| c != ']'), char(']'))),
        ))),
    ))
    .parse(input)
}

fn parse_path_component(input: &str) -> IResult<&str, &str> {
    alt((
        // Quoted path component (e.g., 'SHA-256')
        delimited(char('\''), take_while(|c| c != '\''), char('\'')),
        // Unquoted path component
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
    ))
    .parse(input)
}

// Comparison operator
fn parse_comparison_operator(input: &str) -> IResult<&str, ComparisonOperator> {
    alt((
        value(ComparisonOperator::NotEqual, tag("!=")),
        value(ComparisonOperator::LessThanOrEqual, tag("<=")),
        value(ComparisonOperator::GreaterThanOrEqual, tag(">=")),
        value(ComparisonOperator::Equal, tag("=")),
        value(ComparisonOperator::LessThan, tag("<")),
        value(ComparisonOperator::GreaterThan, tag(">")),
        value(ComparisonOperator::Matches, tag_no_case("MATCHES")),
        value(ComparisonOperator::Like, tag_no_case("LIKE")),
        value(ComparisonOperator::In, tag_no_case("IN")),
        value(ComparisonOperator::IsSubset, tag_no_case("ISSUBSET")),
        value(ComparisonOperator::IsSuperset, tag_no_case("ISSUPERSET")),
    ))
    .parse(input)
}

// Parse a value
fn parse_value(input: &str) -> IResult<&str, PatternValue> {
    alt((
        parse_string_value,
        parse_boolean_value,
        parse_timestamp_value,
        parse_hex_value,
        parse_binary_value,
        parse_float_value,
        parse_integer_value,
        parse_list_value,
    ))
    .parse(input)
}

fn parse_string_value(input: &str) -> IResult<&str, PatternValue> {
    let (input, s) = delimited(
        char('\''),
        escaped(none_of("'\\"), '\\', one_of("'\\nrt")),
        char('\''),
    )
    .parse(input)?;
    Ok((input, PatternValue::String(s.to_string())))
}

fn parse_boolean_value(input: &str) -> IResult<&str, PatternValue> {
    alt((
        value(PatternValue::Boolean(true), tag_no_case("true")),
        value(PatternValue::Boolean(false), tag_no_case("false")),
    ))
    .parse(input)
}

fn parse_timestamp_value(input: &str) -> IResult<&str, PatternValue> {
    let (input, _) = tag("t'")(input)?;
    let (input, ts) = take_while(|c| c != '\'')(input)?;
    let (input, _) = char('\'')(input)?;
    Ok((input, PatternValue::Timestamp(ts.to_string())))
}

fn parse_hex_value(input: &str) -> IResult<&str, PatternValue> {
    let (input, _) = tag("h'")(input)?;
    let (input, hex) = take_while(|c: char| c.is_ascii_hexdigit())(input)?;
    let (input, _) = char('\'')(input)?;
    Ok((input, PatternValue::Hex(hex.to_string())))
}

fn parse_binary_value(input: &str) -> IResult<&str, PatternValue> {
    let (input, _) = tag("b'")(input)?;
    let (input, bytes) = map_res(
        take_while(|c: char| c.is_alphanumeric() || c == '+' || c == '/' || c == '='),
        |b64: &str| BASE64.decode(b64),
    )
    .parse(input)?;
    let (input, _) = char('\'')(input)?;

    Ok((input, PatternValue::Binary(bytes)))
}

fn parse_integer_value(input: &str) -> IResult<&str, PatternValue> {
    let (input, neg) = opt(char('-')).parse(input)?;
    let (input, value) = map_res(digit1, |digits: &str| digits.parse::<i64>()).parse(input)?;
    let value = if neg.is_some() { -value } else { value };

    Ok((input, PatternValue::Integer(value)))
}

fn parse_float_value(input: &str) -> IResult<&str, PatternValue> {
    let (input, neg) = opt(char('-')).parse(input)?;
    let (input, value) = map_res(recognize((digit1, char('.'), digit1)), |s: &str| {
        s.parse::<f64>()
    })
    .parse(input)?;
    let value = if neg.is_some() { -value } else { value };

    Ok((input, PatternValue::Float(value)))
}

fn parse_list_value(input: &str) -> IResult<&str, PatternValue> {
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, items) =
        separated_list0((multispace0, char(','), multispace0), parse_value).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;

    Ok((input, PatternValue::List(items)))
}

// Parse a qualifier
fn parse_qualifier(input: &str) -> IResult<&str, Qualifier> {
    alt((parse_within_qualifier, parse_repeats_qualifier)).parse(input)
}

fn parse_within_qualifier(input: &str) -> IResult<&str, Qualifier> {
    let (input, _) = tag_no_case("WITHIN")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, seconds) = map_res(digit1, |s: &str| s.parse::<u64>()).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag_no_case("SECONDS")(input)?;

    Ok((input, Qualifier::Within(seconds)))
}

fn parse_repeats_qualifier(input: &str) -> IResult<&str, Qualifier> {
    let (input, _) = tag_no_case("REPEATS")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, count) = map_res(digit1, |s: &str| s.parse::<u64>()).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag_no_case("TIMES")(input)?;

    Ok((input, Qualifier::Repeats(count)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_pattern() {
        let pattern = "[ipv4-addr:value = '10.0.0.1']";
        let result = parse_pattern(pattern);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_file_hash_pattern() {
        let pattern = "[file:hashes.'SHA-256' = 'abc123']";
        let result = parse_pattern(pattern);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_pattern() {
        let pattern = "[ipv4-addr:value = '10.0.0.1'] AND [ipv4-addr:value = '10.0.0.2']";
        let result = parse_pattern(pattern);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_or_pattern() {
        let pattern = "[ipv4-addr:value = '10.0.0.1'] OR [domain-name:value = 'example.com']";
        let result = parse_pattern(pattern);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_qualifier() {
        let pattern = "[file:name = 'test.exe'] WITHIN 300 SECONDS";
        let result = parse_pattern(pattern);
        assert!(result.is_ok());
    }
}
