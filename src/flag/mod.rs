use crate::parsers::*;
use parcel::prelude::v1::Parser;
use parcel::ParseResult;
use std::default;
use std::fmt;

mod tests;

/// FlagOrvalue represents a state of being either a flag or a value to be returned my the parser.
#[derive(Debug, PartialEq, Clone)]
pub enum FlagOrValue {
    Flag(String),
    Value(Value),
}

/// ValueType represents one of the values that can be assigned to a flag.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValueType {
    Any,
    Str,
    Bool,
    Integer,
    Float,
}
/// Value represents one of the values that can be encoded into an argument.
/// Currently, this can be represented as either a string, boolean or number.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Str(String),
    Bool(bool),
    Integer(u64),
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
    pub name: String,
    pub short_code: String,
    help_string: String,
    action: Action,
    pub value_type: ValueType,
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

    /// Set the action field.
    #[allow(dead_code)]
    pub fn value_type(mut self, vt: ValueType) -> Flag {
        self.value_type = vt;
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
            value_type: ValueType::Any,
        }
    }
}

impl PartialEq<FlagOrValue> for Flag {
    fn eq(&self, fov: &FlagOrValue) -> bool {
        match fov {
            FlagOrValue::Flag(ref f) => f == &self.name,
            _ => false,
        }
    }
}

impl<'a> Parser<'a, &'a [FlagOrValue], (String, Value)> for Flag {
    fn parse(
        &self,
        input: &'a [FlagOrValue],
    ) -> ParseResult<'a, &'a [FlagOrValue], (String, Value)> {
        match self.action {
            Action::StoreTrue => {
                match_owned_flag(Flag::new().name(&self.name).short_code(&self.short_code))
                    .map(|f| (f.name.clone(), Value::Bool(true)))
            }
            Action::StoreFalse => {
                match_owned_flag(Flag::new().name(&self.name).short_code(&self.short_code))
                    .map(|f| (f.name.clone(), Value::Bool(false)))
            }
            Action::ExpectSingleValue => {
                match_owned_flag(Flag::new().name(&self.name).short_code(&self.short_code))
                    .map(|f| (f.name.clone(), Value::Bool(false)))
            }
        }
        .parse(input)
    }
}
