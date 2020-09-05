use std::default;
use std::fmt;

mod tests;

/// Action stores the flag action, signifying the behavior of the flag.
/// Examples include, storing true, false or expecting a value.
#[derive(Debug, Clone, PartialEq)]
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the command name.
    pub fn name(mut self, name: &str) -> Flag {
        self.name = name.to_string();
        self
    }

    /// Set the short_code.
    pub fn short_code(mut self, short: &str) -> Flag {
        self.short_code = short.to_string();
        self
    }

    /// Set the description.
    pub fn help_string(mut self, hs: &str) -> Flag {
        self.help_string = hs.to_string();
        self
    }

    /// Set the action field.
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
