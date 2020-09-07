use super::character_parsers::{alphabetic, alphabetic_or_dash, character};
use parcel::Parser;
use parcel::{join, one_or_more, right, take_n}; // parser combinators

pub fn any_flag<'a>() -> impl parcel::Parser<'a, &'a str, String> {
    right(join(
        take_n(character('-'), 2),
        one_or_more(alphabetic_or_dash()),
    ))
    .map(|cv| cv.iter().collect::<String>())
    .or(|| right(join(take_n(character('-'), 1), alphabetic())).map(|c| c.to_string()))
}
