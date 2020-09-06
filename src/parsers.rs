use parcel::{MatchStatus, Parser};

pub fn match_string<'a>(expected: String) -> impl Parser<'a, &'a [&'a str], &'a str> {
    move |input: &'a [&str]| match input.get(0) {
        Some(&next) if next == expected => Ok(MatchStatus::Match((&input[1..], next))),
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}
