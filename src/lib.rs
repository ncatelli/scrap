#[cfg(test)]
mod tests;

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

pub struct App {
    name: String,
    author: String,
    description: String,
    version: SemanticVersion,
}

impl App {
    fn new() -> Self {
        App {
            name: String::new(),
            author: String::new(),
            description: String::new(),
            version: SemanticVersion::new(0, 0, 0),
        }
    }

    fn name(mut self, name: String) -> App {
        self.name = name;
        self
    }

    fn author(mut self, author: String) -> App {
        self.author = author;
        self
    }

    fn description(mut self, desc: String) -> App {
        self.description = desc;
        self
    }

    fn version(mut self, vers: SemanticVersion) -> App {
        self.version = vers;
        self
    }
}
