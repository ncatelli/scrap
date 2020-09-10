use crate::flag::{Flag, FlagOrValue, Value, ValueType};
use crate::parsers::match_value_type;
use crate::parsers::ArgumentParser;
use parcel::join;
use parcel::MatchStatus;
use parcel::Parser;
use parcel::{one_of, one_or_more}; // parcel parser combinators
use std::collections::HashMap;
use std::default;
use std::fmt;

mod flag;
mod parsers;

#[cfg(test)]
mod tests;

pub type Config = HashMap<String, Value>;

/// App functions as the top level wrapper for a command command line tool
/// storing information about the tool, author, version and a brief description.
#[derive(Debug, Clone, PartialEq)]
pub struct App {
    name: String,
    author: String,
    description: String,
    version: String,
    flags: Vec<Flag>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the command name.
    pub fn name(mut self, name: &str) -> App {
        self.name = name.to_string();
        self
    }

    /// Set the author name.
    pub fn author(mut self, author: &str) -> App {
        self.author = author.to_string();
        self
    }

    /// Set the short description.
    pub fn description(mut self, desc: &str) -> App {
        self.description = desc.to_string();
        self
    }

    /// Set the version.
    pub fn version(mut self, vers: &str) -> App {
        self.version = vers.to_string();
        self
    }

    /// Set a flag.
    pub fn flag(mut self, f: Flag) -> App {
        self.flags.push(f);
        self
    }
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = if !self.description.is_empty() {
            format!("{}\n", &self.description)
        } else {
            "".to_string()
        };
        let name = if !self.name.is_empty() {
            format!(" {} ", &self.name)
        } else {
            "".to_string()
        };

        write!(f, "{}Usage:{}[OPTIONS] [SUBCOMMAND]", desc, name)
    }
}

impl default::Default for App {
    fn default() -> Self {
        App {
            name: String::new(),
            author: String::new(),
            description: String::new(),
            version: String::new(),
            flags: Vec::new(),
        }
    }
}

impl App {
    /// parse expects a Vec<String> representing all argumets provided from
    /// std::env::Args, including the base command and attempts to parse it
    /// into a corresponding Config.
    pub fn parse(&self, input: Vec<String>) -> Result<Config, String> {
        let mut cm = Config::new();

        // set defaults
        for f in self.flags.iter() {
            match f.default_value {
                Some(ref v) => cm.insert(f.name.clone(), v.clone()),
                None => continue,
            };
        }

        let res = match ArgumentParser::new().parse(input)? {
            MatchStatus::Match((_, res)) => Ok(res),
            MatchStatus::NoMatch(remainder) => {
                Err(format!("unable to parse full arg string: {:?}", remainder))
            }
        }?;

        let config_pairs = match join(
            match_value_type(ValueType::Str),
            one_or_more(one_of(self.flags.clone())),
        )
        .parse(&res)?
        {
            MatchStatus::Match((remainder, (_, res))) => {
                let unparsed_flags: Vec<&String> = remainder
                    .iter()
                    .map(|fov| match fov {
                        FlagOrValue::Flag(f) => Ok(f),
                        _ => Err(()),
                    })
                    .filter_map(|f| f.ok())
                    .collect();

                if !unparsed_flags.is_empty() {
                    Err(format!("unable to parse all flags: {:?}", unparsed_flags))
                } else {
                    Ok(res)
                }
            }
            MatchStatus::NoMatch(remainder) => {
                Err(format!("unable to parse full arg string: {:?}", remainder))
            }
        }?;

        for pair in config_pairs.into_iter() {
            cm.insert(pair.0, pair.1);
        }

        Ok(cm)
    }
}
