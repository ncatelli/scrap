use std::fmt;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct SemanticVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

impl SemanticVersion {
    fn new(major: u16, minor: u16, patch: u16) -> SemanticVersion {
        SemanticVersion {
            major,
            minor,
            patch,
        }
    }
}

#[derive(Debug)]
pub struct App {
    name: String,
    author: String,
    description: String,
    version: SemanticVersion,
}

impl App {
    pub fn new() -> Self {
        App {
            name: String::new(),
            author: String::new(),
            description: String::new(),
            version: SemanticVersion::new(0, 0, 0),
        }
    }

    pub fn name(mut self, name: String) -> App {
        self.name = name;
        self
    }

    pub fn author(mut self, author: String) -> App {
        self.author = author;
        self
    }

    pub fn description(mut self, desc: String) -> App {
        self.description = desc;
        self
    }

    pub fn version(mut self, vers: SemanticVersion) -> App {
        self.version = vers;
        self
    }
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\nUSAGE:\n\t{} [OPTIONS] [SUBCOMMAND]",
            self.description, self.name
        )
    }
}
