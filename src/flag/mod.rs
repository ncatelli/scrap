use crate::parsers::*;
use parcel::prelude::v1::Parser;
use parcel::ParseResult;
use parcel::{join, one_or_more, right, take_n}; // parser combinators
use std::default;
use std::fmt;

mod tests;

/// FlagOrvalue represents a state of being either a flag or a value to be returned my the parser.
#[derive(Debug, PartialEq, Clone)]
pub enum FlagOrValue {
    Flag(String),
    Value(Value),
}

/// Value represents one of the values that can be encoded into an argument.
/// Currently, this can be represented as either a string, boolean or number.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Str(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
}

/// Action stores the flag action, signifying the behavior of the flag.
/// Examples include, storing true, false or expecting a value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    StoreTrue,
    StoreFalse,
    ExpectSingleValue,
}

#[derive(Debug, Clone, PartialEq)]
/// Flag represents an option that can be passed to an App, providing setable
/// fields for the name, short_code, help_string and whether the flag expects
/// a value
pub struct Flag {
    name: String,
    short_code: String,
    help_string: String,
    action: Action,
}

impl Flag {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the command name.
    #[allow(dead_code)]
    pub fn name(mut self, name: &str) -> Flag {
        self.name = name.to_string();
        self
    }

    /// Set the short_code.
    #[allow(dead_code)]
    pub fn short_code(mut self, short: &str) -> Flag {
        self.short_code = short.to_string();
        self
    }

    /// Set the description.
    #[allow(dead_code)]
    pub fn help_string(mut self, hs: &str) -> Flag {
        self.help_string = hs.to_string();
        self
    }

    /// Set the action field.
    #[allow(dead_code)]
    pub fn action(mut self, action: Action) -> Flag {
        self.action = action;
        self
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut format_string = format!("--{}", &self.name);

        if !&self.short_code.is_empty() {
            format_string.push_str(&format!(", -{}", &self.short_code))
        };

        if !&self.help_string.is_empty() {
            format_string.push_str(&format!(": {}", &self.help_string))
        };

        write!(f, "{}", &format_string)
    }
}

impl default::Default for Flag {
    fn default() -> Self {
        Flag {
            name: String::new(),
            short_code: String::new(),
            help_string: String::new(),
            action: Action::ExpectSingleValue,
        }
    }
}

impl<'a> Parser<'a, &'a [&'a str], (String, Value)> for Flag {
    fn parse(&self, _input: &'a [&'a str]) -> ParseResult<'a, &'a [&'a str], (String, Value)> {
        Err("Unimplemented".to_string())
        /*
        let name = self.name.clone();
        let shortcode = self.short_code.clone();
        match self.action {
            Action::StoreTrue => match_flag(name)
                .or(move || match_flag(shortcode.clone()))
                .map(|res| (res, Value::Bool(true))),
            Action::StoreFalse => match_flag(name)
                .or(move || match_flag(shortcode.clone()))
                .map(|res| (res, Value::Bool(false))),
            Action::ExpectSingleValue => join(
                match_flag(name).or(move || match_flag(shortcode.clone())),
                right(join(whitespace(), one_or_more(alphabetic()))),
            )
            .map(|(res, v)| (res, Value::Str(v.iter().collect::<String>()))),
        }
        .parse(input)
        */
    }
}

impl<'a> Parser<'a, &'a str, (String, Value)> for Flag {
    fn parse(&self, input: &'a str) -> ParseResult<'a, &'a str, (String, Value)> {
        let name = self.name.clone();
        let shortcode = self.short_code.clone();
        match self.action {
            Action::StoreTrue => match_flag(name)
                .or(move || match_flag(shortcode.clone()))
                .map(|res| (res, Value::Bool(true))),
            Action::StoreFalse => match_flag(name)
                .or(move || match_flag(shortcode.clone()))
                .map(|res| (res, Value::Bool(false))),
            Action::ExpectSingleValue => join(
                match_flag(name).or(move || match_flag(shortcode.clone())),
                right(join(whitespace(), one_or_more(alphabetic()))),
            )
            .map(|(res, v)| (res, Value::Str(v.iter().collect::<String>()))),
        }
        .parse(input)
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
