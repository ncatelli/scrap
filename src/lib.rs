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

/// Config represents a String -> Value mapping as parsed from flags.
pub type Config = HashMap<String, Value>;

/// Represents the result of a dispatch function call.
pub type DispatchFnResult = Result<u32, String>;

/// DispatchFn stores an invocable function to be called by the cli
pub type DispatchFn = dyn Fn(Config) -> DispatchFnResult;

/// CmdDispatcher captures a parsed config and a handler function to carry forward
/// commands.
pub struct CmdDispatcher {
    pub config: Config,
    pub handler_func: Box<DispatchFn>,
}

impl CmdDispatcher {
    pub fn new(config: Config, handler_func: Box<DispatchFn>) -> Self {
        CmdDispatcher {
            config,
            handler_func,
        }
    }

    /// Explicilty converts a dispatcher to its enclosed config.
    pub fn to_config(self) -> Config {
        self.config
    }

    /// dispatch accepts a config as an argument to be passed on to the
    /// commands contained handler method.
    pub fn dispatch(self) -> Result<u32, String> {
        (self.handler_func)(self.config)
    }
}

impl fmt::Debug for CmdDispatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CmdDispatcher")
            .field("config", &self.config)
            .field("handler_func", &"Fn(Config) -> DispatchFnResult")
            .finish()
    }
}

impl PartialEq for CmdDispatcher {
    fn eq(&self, other: &CmdDispatcher) -> bool {
        self.config == other.config
    }
}

/// Cmd functions as the top level wrapper for a command command line tool
/// storing information about the tool, author, version and a brief description.
pub struct Cmd {
    name: String,
    author: String,
    description: String,
    version: String,
    flags: Vec<Flag>,
    handler_func: Box<DispatchFn>,
}

impl Cmd {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the command name.
    pub fn name(mut self, name: &str) -> Cmd {
        self.name = name.to_string();
        self
    }

    /// Set the author name.
    pub fn author(mut self, author: &str) -> Cmd {
        self.author = author.to_string();
        self
    }

    /// Set the short description.
    pub fn description(mut self, desc: &str) -> Cmd {
        self.description = desc.to_string();
        self
    }

    /// Set the version.
    pub fn version(mut self, vers: &str) -> Cmd {
        self.version = vers.to_string();
        self
    }

    /// Set a flag.
    pub fn flag(mut self, f: Flag) -> Cmd {
        self.flags.push(f);
        self
    }

    /// Set a cmd handler.
    pub fn handler(mut self, handler: Box<DispatchFn>) -> Cmd {
        self.handler_func = handler;
        self
    }
}

impl fmt::Display for Cmd {
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

impl default::Default for Cmd {
    fn default() -> Self {
        Cmd {
            name: String::new(),
            author: String::new(),
            description: String::new(),
            version: String::new(),
            flags: Vec::new(),
            handler_func: Box::new(|_conf| Err("Unimplemented".to_string())),
        }
    }
}

impl Cmd {
    /// parse expects a Vec<String> representing all argumets provided from
    /// std::env::Args, including the base command and attempts to parse it
    /// into a corresponding Command Dispatcher.
    pub fn parse(self, input: Vec<String>) -> Result<CmdDispatcher, String> {
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

        Ok(CmdDispatcher::new(cm, self.handler_func))
    }
}
