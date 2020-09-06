use parcel::{MatchStatus, Parser};

pub fn match_string<'a>(expected: String) -> impl Parser<'a, &'a str, String> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok(MatchStatus::Match((&input[1..], next.to_string()))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}
