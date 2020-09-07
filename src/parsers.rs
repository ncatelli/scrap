use crate::flag::{FlagOrValue, Value};
use parcel::{join, one_or_more, optional, right, take_n}; // parser combinators
use parcel::{MatchStatus, ParseResult, Parser};

pub fn whitespace<'a>() -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) if next.is_whitespace() => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn any<'a>() -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn alphabetic<'a>() -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) if next.is_alphabetic() => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn alphabetic_or_dash<'a>() -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) if next.is_alphabetic() || next == '-' || next == '_' => {
            Ok(MatchStatus::Match((&input[1..], next)))
        }
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn character<'a>(expected: char) -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) if next == expected => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn numeric<'a>() -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) if next.is_digit(10) => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn match_flag<'a>(expected: String) -> impl parcel::Parser<'a, &'a str, String> {
    any_flag().predicate(move |f| *f == expected)
}

pub fn any_flag<'a>() -> impl parcel::Parser<'a, &'a str, String> {
    right(join(
        take_n(character('-'), 2),
        one_or_more(alphabetic_or_dash()),
    ))
    .map(|cv| cv.iter().collect::<String>())
    .or(|| right(join(take_n(character('-'), 1), alphabetic())).map(|c| c.to_string()))
}

/// ArgumentParser handles the parsing of individual std::env::Args arguments
/// into an intermediate FlagOrValue representation.
pub struct ArgumentParser {}

impl ArgumentParser {
    pub fn new() -> Self {
        ArgumentParser {}
    }
}

impl<'a> Parser<'a, &'a str, FlagOrValue> for ArgumentParser {
    fn parse(&self, input: &'a str) -> ParseResult<'a, &'a str, FlagOrValue> {
        any_flag()
            .map(|f| FlagOrValue::Flag(f))
            .or(|| {
                join(
                    one_or_more(numeric()),
                    optional(right(join(character('.'), one_or_more(numeric())))),
                )
                .map(|(whole, decimal)| match decimal {
                    Some(num) => FlagOrValue::Value(Value::Float(
                        format!(
                            "{}.{}",
                            whole.iter().collect::<String>(),
                            num.iter().collect::<String>()
                        )
                        .parse()
                        .unwrap(),
                    )),
                    None => FlagOrValue::Value(Value::Integer(
                        whole.iter().collect::<String>().parse().unwrap(),
                    )),
                })
            })
            .or(|| {
                one_or_more(any())
                    .map(|cv| FlagOrValue::Value(Value::Str(cv.iter().collect::<String>())))
            })
            .parse(input)
    }
}
