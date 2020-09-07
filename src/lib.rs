use crate::parsers::ArgumentParser;
use parcel::MatchStatus;
use parcel::Parser;
use std::default;
use std::fmt;

mod flag;
mod parsers;

#[cfg(test)]
mod tests;

/// App functions as the top level wrapper for a command command line tool
/// storing information about the tool, author, version and a brief description.
#[derive(Debug, Clone, PartialEq)]
pub struct App {
    name: String,
    author: String,
    description: String,
    version: String,
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
        }
    }
}

impl App {
    pub fn parse(&self, input: Vec<String>) -> Result<(), String> {
        let ap = ArgumentParser::new();
        let res = match ap.parse(input)? {
            MatchStatus::Match((_, res)) => Ok(res),
            MatchStatus::NoMatch(remainder) => {
                Err(format!("unable to parse full arg string: {:?}", remainder))
            }
        }?;

        Err("Unimplemented".to_string())
    }
}
