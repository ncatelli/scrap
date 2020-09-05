use std::default;
use std::fmt;

#[cfg(test)]
mod tests;

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

    pub fn name(mut self, name: &str) -> App {
        self.name = name.to_string();
        self
    }

    pub fn author(mut self, author: &str) -> App {
        self.author = author.to_string();
        self
    }

    pub fn description(mut self, desc: &str) -> App {
        self.description = desc.to_string();
        self
    }

    pub fn version(mut self, vers: &str) -> App {
        self.version = vers.to_string();
        self
    }
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\nUsage: {} [OPTIONS] [SUBCOMMAND]",
            self.description, self.name
        )
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
