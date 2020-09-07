use parcel::{MatchStatus, Parser};

pub fn match_string<'a>(expected: String) -> impl Parser<'a, &'a str, String> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok(MatchStatus::Match((&input[1..], next.to_string()))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn alphabetic<'a>() -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) if next.is_alphabetic() => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn character<'a>(expected: char) -> impl Parser<'a, &'a str, char> {
    move |input: &'a str| match input.chars().next() {
        Some(next) if next == expected => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}
