use parcel::{join, one_of, zero_or_more}; // parcel parser combinators
use parcel::{MatchStatus, ParseResult, Parser};
use std::collections::HashMap;
use std::default;
use std::fmt;
use std::path::Path;

pub mod flag;
use flag::{Action, Flag, FlagOrValue, Value, ValueType};

mod parsers;
use parsers::ArgumentParser;
use parsers::{match_any_flag, match_value_type};

#[cfg(test)]
mod tests;

/// Config represents a String -> Value mapping as parsed from flags.
pub type Config = HashMap<String, Value>;

fn config_from_string_value_tuple(value_pairing: Vec<(String, Value)>) -> Config {
    let mut cm = Config::new();

    for (k, v) in value_pairing.into_iter() {
        cm.insert(k, v);
    }

    cm
}

fn config_from_defaults(flags: &[Flag]) -> Config {
    let mut cm = Config::new();

    for f in flags.iter() {
        match f.default_value {
            Some(ref v) => cm.insert(f.name.clone(), v.clone()),
            None => continue,
        };
    }

    cm
}

/// Represents the result of a dispatch function call.
pub type DispatchFnResult = Result<i32, String>;

/// DispatchFn stores an invocable function to be called by the cli
pub type DispatchFn = dyn Fn(Config) -> DispatchFnResult;

/// CmdDispatcher captures a parsed config and a handler function to carry forward
/// commands.
pub struct CmdDispatcher {
    pub config: Config,
    pub help_string: String,
    pub handler_func: Box<DispatchFn>,
}

impl CmdDispatcher {
    pub fn new(config: Config, help_string: String, handler_func: Box<DispatchFn>) -> Self {
        CmdDispatcher {
            config,
            help_string,
            handler_func,
        }
    }

    /// Explicilty converts a dispatcher to its enclosed config.
    pub fn to_config(self) -> Config {
        self.config
    }

    /// dispatch accepts a config as an argument to be passed on to the
    /// commands contained handler method.
    pub fn dispatch(self) -> Result<i32, String> {
        if Some(&Value::Bool(true)) == self.config.get("help") {
            println!("{}", self.help_string);
            Ok(0)
        } else {
            (self.handler_func)(self.config)
        }
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

/// CmdGroup provides a group of a root cmd and child subcommands.
pub struct CmdGroup {
    command: Cmd,
    subcommands: Vec<Cmd>,
}

impl CmdGroup {
    pub fn new(cmd: Cmd) -> Self {
        Self {
            command: cmd,
            subcommands: Vec::new(),
        }
    }

    /// Set the command name.
    pub fn name(mut self, name: &str) -> Self {
        self.command.name = name.to_string();
        self
    }

    /// Set the author name.
    pub fn author(mut self, author: &str) -> Self {
        self.command.author = author.to_string();
        self
    }

    /// Set the short description.
    pub fn description(mut self, desc: &str) -> Self {
        self.command.description = desc.to_string();
        self
    }

    /// Set the version.
    pub fn version(mut self, vers: &str) -> Self {
        self.command.version = vers.to_string();
        self
    }

    /// Set a flag.
    pub fn flag(mut self, f: Flag) -> Self {
        self.command.flags.push(f);
        self
    }

    /// Set a cmd handler.
    pub fn handler(mut self, handler: Box<DispatchFn>) -> Self {
        self.command.handler_func = handler;
        self
    }

    /// add a subcommand to the CmdGroup
    pub fn subcommand(mut self, sc: Cmd) -> Self {
        self.subcommands.push(sc);
        self
    }

    pub fn flatten(self) -> Vec<Cmd> {
        let mut cv: Vec<Cmd> = vec![self.command];
        cv.extend(self.subcommands.into_iter());
        cv
    }
}

impl fmt::Display for CmdGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.command)
    }
}

impl CmdGroup {
    /// run expects a Vec<String> representing all argumets provided from
    /// std::env::Args, including the base command and attempts to parse it
    /// into a corresponding Command Dispatcher.
    pub fn run(self, input: Vec<String>) -> Result<CmdDispatcher, String> {
        let ap_res = match ArgumentParser::new().parse(input)? {
            MatchStatus::Match((_, res)) => Ok(res),
            MatchStatus::NoMatch(remainder) => {
                Err(format!("unable to parse full arg string: {:?}", remainder))
            }
        }?;

        let mut cm = config_from_defaults(&self.command.flags);
        let mut remainder: &[FlagOrValue] = &ap_res;

        for cmd in self.flatten().into_iter() {
            match cmd.parse(remainder)? {
                MatchStatus::Match((rem, conf)) if rem.is_empty() => {
                    cm.extend(conf);
                    return Ok(CmdDispatcher::new(
                        cm,
                        format!("{}", &cmd),
                        cmd.handler_func,
                    ));
                }
                MatchStatus::Match((rem, conf)) => {
                    remainder = rem;
                    cm.extend(conf);
                }
                MatchStatus::NoMatch(rem) if rem.is_empty() => {
                    return Err(format!("unable to parse full arg string: {:?}", rem))
                }
                MatchStatus::NoMatch(rem) => {
                    remainder = rem;
                }
            }
        }

        return Err(format!("unable to parse full arg string: {:?}", remainder));
    }
}

/// Cmd functions as the wrapper for a command line tool storing information
/// about the tool, author, version and a brief description.
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
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set the author name.
    pub fn author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    /// Set the short description.
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set the version.
    pub fn version(mut self, vers: &str) -> Self {
        self.version = vers.to_string();
        self
    }

    /// Set a flag.
    pub fn flag(mut self, f: Flag) -> Self {
        self.flags.push(f);
        self
    }

    /// Set a cmd handler.
    pub fn handler(mut self, handler: Box<DispatchFn>) -> Self {
        self.handler_func = handler;
        self
    }

    /// add a subcommand, implicily converting to a CmdGroup.
    pub fn subcommand(self, sc: Cmd) -> CmdGroup {
        let mut cg = CmdGroup::new(self);
        cg.subcommands.push(sc);
        cg
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
            flags: vec![Flag::new()
                .name("help")
                .short_code("h")
                .action(Action::StoreTrue)
                .value_type(ValueType::Bool)],
            handler_func: Box::new(|_| Err("Unimplemented".to_string())),
        }
    }
}

impl Cmd {
    /// Run Converts the command to a CmdGroup node in the cli and then calls
    /// that types corresponding run method.
    pub fn run(self, input: Vec<String>) -> Result<CmdDispatcher, String> {
        CmdGroup::new(self).run(input)
    }
}

impl<'a> Parser<'a, &'a [FlagOrValue], Config> for Cmd {
    fn parse(&self, input: &'a [FlagOrValue]) -> ParseResult<'a, &'a [FlagOrValue], Config> {
        let preparse_input = input;
        match join(
            match_value_type(ValueType::Str),
            zero_or_more(one_of(self.flags.clone())),
        )
        .parse(input)?
        {
            MatchStatus::Match((remainder, (Value::Str(cmd), res)))
                if Path::new(&cmd).ends_with(&self.name) =>
            {
                match match_any_flag().parse(remainder)? {
                    MatchStatus::Match((_remaining_fov, _)) => {
                        Err("unable to parse all flags".to_string())
                    }
                    MatchStatus::NoMatch(remaining_fov) => Ok(MatchStatus::Match((
                        remaining_fov,
                        config_from_string_value_tuple(res),
                    ))),
                }
            }
            MatchStatus::Match(_) => Ok(MatchStatus::NoMatch(preparse_input)),
            MatchStatus::NoMatch(remaining_fov) if remaining_fov.is_empty() => {
                Ok(MatchStatus::NoMatch(remaining_fov))
            }
            MatchStatus::NoMatch(remaining_fov) => Ok(MatchStatus::NoMatch(remaining_fov)),
        }
    }
}
