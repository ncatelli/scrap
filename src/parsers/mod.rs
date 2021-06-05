use crate::flag::{FlagOrValue, Value};
use parcel::{join, one_or_more, optional, right}; // parser combinators
use parcel::{MatchStatus, ParseResult, Parser};

mod character_parsers;
mod flag_parsers;
mod string_parsers;
pub use character_parsers::*;
pub use flag_parsers::*;
pub use string_parsers::*;

/// ArgumentParser handles the parsing of individual std::env::Args arguments
/// into an intermediate FlagOrValue representation.
#[derive(Default)]
pub struct ArgumentParser {}

impl ArgumentParser {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ArgumentParser::default()
    }
}

impl<'a> Parser<'a, Vec<String>, Vec<FlagOrValue>> for ArgumentParser {
    fn parse(&self, input: Vec<String>) -> ParseResult<'a, Vec<String>, Vec<FlagOrValue>> {
        let mut remainder: Vec<String> = vec![];
        let mut flags: Vec<FlagOrValue> = vec![];

        let res: Vec<Result<FlagOrValue, String>> = input
            .iter()
            .map(|s| self.parse(s.as_str()))
            .map(|pr| match pr {
                Ok(MatchStatus::Match((_, v))) => Ok(v),
                Ok(MatchStatus::NoMatch(next)) => Err(next.to_string()),
                Err(e) => panic!("{}", e),
            })
            .collect();

        for i in res.into_iter() {
            match i {
                Ok(fov) => flags.push(fov),
                Err(next) => remainder.push(next),
            }
        }

        Ok(MatchStatus::Match((remainder, flags)))
    }
}

impl<'a> Parser<'a, &'a str, FlagOrValue> for ArgumentParser {
    fn parse(&self, input: &'a str) -> ParseResult<'a, &'a str, FlagOrValue> {
        any_flag()
            .map(FlagOrValue::Flag)
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
