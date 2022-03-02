//! A minimal command-line utility framework built with zero external
//! dependencies. This tool attempts to retain type information throughout the
//! entire lifecycle of a command parse with the intent of lifting validation
//! of a command's handler to compile-time verifiable.
//!
//! # Example
//!
//! I've include a bare minimal example of a command definition below.
//!
//! ```
//! use scrap::prelude::v1::*;
//! use std::env;
//!
//! let raw_args: Vec<String> = env::args().into_iter().collect::<Vec<String>>();
//! let args = raw_args.iter().map(|a| a.as_str()).collect::<Vec<&str>>();
//!
//! // The `Flag` type defines helpers for generating various common flag
//! // evaluators.
//! // Shown below, the `help` flag represents common boolean flag with default
//! // a default value.
//! let help = scrap::Flag::store_true("help", "h", "output help information.")
//!     .optional()
//!     .with_default(false);
//! // `direction` provides a flag with a finite set of choices, matching a
//! // string value.
//! let direction = scrap::Flag::with_choices(
//!     "direction",
//!     "d",
//!     "a cardinal direction.",
//!     [
//!         "north".to_string(),
//!         "south".to_string(),
//!         "east".to_string(),
//!         "west".to_string(),
//!     ],
//!     scrap::StringValue,
//! );
//!
//! // `Cmd` defines the named command, combining metadata without our above defined command.
//! let cmd = scrap::Cmd::new("minimal")
//!     .description("A minimal example cli.")
//!     .author("John Doe <jdoe@example.com>")
//!     .version("1.2.3")
//!     .with_flag(help)
//!     .with_flag(direction)
//!     // Finally a handler is defined with its signature being a product of
//!     //the cli's defined flags.
//!     .with_handler(|(_, direction)| println!("You chose {}.", direction));
//!
//! // The help method generates a help command based on the output rendered
//! // from all defined flags.
//! let help_string = cmd.help();
//!
//! // Evaluate attempts to parse the input, evaluating all commands and flags
//! // into concrete types which can be passed to `dispatch`, the defined
//! // handler.
//! let res = cmd
//!     .evaluate(&args[..])
//!     .map_err(|e| e.to_string())
//!     .and_then(|Value { span, value }| match value {
//!         (help, direction) if !help => {
//!             cmd.dispatch(Value::new(span, (help, direction)));
//!             Ok(())
//!         }
//!         _ => Err("output help".to_string()),
//!     });
//!
//! match res {
//!     Ok(_) => (),
//!     Err(_) => println!("{}", help_string),
//! }
//! ```

pub mod prelude;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum CliError {
    AmbiguousCommand,
    ValueEvaluation,
    FlagEvaluation(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AmbiguousCommand => write!(f, "ambiguous command"),
            Self::ValueEvaluation => write!(f, "value missmatch"),
            Self::FlagEvaluation(name) => write!(f, "unable to evaluate flag: {}", name),
        }
    }
}

/// CmdGroup functions as a grouping of multiple dispatchable commands under a
/// single command grouping.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// let left_cmd = Cmd::new("test_one")
///     .description("first test cmd")
///     .with_flag(
///         Flag::expect_string("name", "n", "A name.")
///             .optional()
///             .with_default("foo".to_string())
///     )
///     .with_handler(|name| {
///         format!("name: {}", &name);
///     });
///
/// let right_cmd = Cmd::new("test_two")
///     .description("a test cmd")
///     .with_flag(
///         Flag::store_true("debug", "d", "Run command in debug mode.")
///             .optional()
///             .with_default(false)
///     )
///     .with_handler(|debug| {
///         format!("debug: {}", &debug);
///     });
///
/// let commands = OneOf::new(left_cmd, right_cmd);
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(0..4), Either::Left("test".to_string()))),
///     CmdGroup::new("testgroup").with_command(commands)
///         .evaluate(&["testgroup", "test_one", "-n", "test"][..])
/// );
/// ```
#[derive(Debug)]
pub struct CmdGroup<C> {
    name: &'static str,
    description: &'static str,
    author: &'static str,
    version: &'static str,
    commands: C,
}

impl CmdGroup<()> {
    /// Instantiates a new instance of `CmdGroup` with the name field set to
    /// the passed value. All other fields will be set to their default values.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// CmdGroup::new("test");
    /// ```
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            description: "",
            author: "",
            version: "",
            commands: (),
        }
    }

    /// Returns a new instance of `CmdGroup` with the type derived from the value of
    /// the passed Cmd.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// CmdGroup::new("test_group")
    ///     .with_command(Cmd::new("test"));
    /// ```
    pub fn with_command<NC>(self, new_cmd: NC) -> CmdGroup<NC> {
        CmdGroup {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            commands: new_cmd,
        }
    }
}

impl<C> CmdGroup<C> {
    /// Returns CmdGroup with the name field set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// CmdGroup::new("test").name("other_test");
    /// ```
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Returns CmdGroup with the description field set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// CmdGroup::new("test").description("a test command group");
    /// ```
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Returns CmdGroup with the author field set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// CmdGroup::new("test").description("a test command group");
    /// ```
    pub fn author(mut self, author: &'static str) -> Self {
        self.author = author;
        self
    }

    /// Returns CmdGroup with the version field set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// CmdGroup::new("test").version("0.1.0");
    /// ```
    pub fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }
}

impl<C> CmdGroup<C>
where
    C: IsCmd,
{
    /// Returns a new instance of `CmdGroup` with the type derived from the value of
    /// the passed Cmd.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// CmdGroup::new("test_group")
    ///     .with_command(Cmd::new("test"));
    /// ```
    pub fn with_command<NC>(self, new_cmd: NC) -> CmdGroup<OneOf<C, NC>> {
        CmdGroup {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            commands: OneOf::new(self.commands, new_cmd),
        }
    }
}

impl<'a, C, B> Evaluatable<'a, &'a [&'a str], B> for CmdGroup<C>
where
    C: Evaluatable<'a, &'a [&'a str], B>,
    B: std::fmt::Debug,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, B> {
        let filename = input
            .get(0)
            .map(|&bin| std::path::Path::new(bin).file_name());

        match filename {
            Some(Some(name)) if name == self.name => self
                .commands
                .evaluate(&input[1..])
                .map(|v| v.from_offset(1)),
            _ => Err(CliError::AmbiguousCommand),
        }
        // Add group to range
        .map(|v| Value::new(Span::from_range(0..1).join(v.span), v.value))
    }
}

impl<'a, C, A, B, R> Dispatchable<A, B, R> for CmdGroup<C>
where
    C: Evaluatable<'a, A, B> + Dispatchable<A, B, R>,
{
    fn dispatch(self, flag_values: Value<B>) -> R {
        self.commands.dispatch(flag_values)
    }
}

impl<'a, C, A, B, R> DispatchableWithArgs<A, B, R> for CmdGroup<C>
where
    C: Evaluatable<'a, A, B> + DispatchableWithArgs<A, B, R>,
{
    fn dispatch_with_args(self, flag_values: Value<B>, args: Vec<String>) -> R {
        self.commands.dispatch_with_args(flag_values, args)
    }
}

impl<'a, C, B, R> DispatchableWithHelpString<B, R> for CmdGroup<C>
where
    Self: Helpable<Output = String>,
    C: DispatchableWithHelpString<B, R>,
{
    fn dispatch_with_helpstring(self, flag_values: Value<B>) -> R {
        let help_string = self.help();
        self.commands
            .dispatch_with_supplied_helpstring(help_string, flag_values)
    }

    fn dispatch_with_supplied_helpstring(self, help_string: String, flag_values: Value<B>) -> R {
        self.commands
            .dispatch_with_supplied_helpstring(help_string, flag_values)
    }
}

impl<C> Helpable for CmdGroup<C>
where
    C: ShortHelpable<Output = String>,
{
    type Output = String;

    fn help(&self) -> Self::Output {
        format!(
            "Usage: {} [OPTIONS]\n{}\nSubcommands:\n{}",
            self.name,
            self.description,
            self.commands.short_help()
        )
    }
}

/// Either, much like Result, provides an enum for encapsulating one of two
/// exclusive values.
#[derive(Debug, PartialEq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

/// OneOf provides methods for joining two Cmd evaluators into a single,
/// exclusive object. This functions much like `Join` however in the case of
/// `OneOf` only one type can correctly evaluate.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// let left_cmd = Cmd::new("test_one")
///     .description("first test cmd")
///     .with_flag(
///         Flag::expect_string("name", "n", "A name.")
///             .optional()
///             .with_default("foo".to_string())
///     )
///     .with_handler(|name| {
///         format!("name: {}", &name);
///     });
///
/// let right_cmd = Cmd::new("test_two")
///     .description("a test cmd")
///     .with_flag(
///         Flag::store_true("debug", "d", "Run command in debug mode.")
///             .optional()
///             .with_default(false)
///     )
///     .with_handler(|debug| {
///         format!("debug: {}", &debug);
///     });
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(0..3), Either::Left("test".to_string()))),
///     OneOf::new(left_cmd, right_cmd)
///         .evaluate(&["test_one", "-n", "test"][..])
/// );
/// ```
#[derive(Debug)]
pub struct OneOf<C1, C2> {
    left: C1,
    right: C2,
}

impl<C1, C2> OneOf<C1, C2> {
    /// Instantiates a new instance of `OneOf` with the types associated with
    /// the passed values.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// OneOf::new(Cmd::new("left"), Cmd::new("right"));
    /// ```
    pub fn new(left: C1, right: C2) -> Self {
        Self { left, right }
    }
}

impl<'a, C1, C2, B, C> Evaluatable<'a, &'a [&'a str], Either<B, C>> for OneOf<C1, C2>
where
    C1: Evaluatable<'a, &'a [&'a str], B>,
    C2: Evaluatable<'a, &'a [&'a str], C>,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, Either<B, C>> {
        match (self.left.evaluate(input), self.right.evaluate(input)) {
            (Ok(Value { span, value: b }), Err(_)) => Ok(Value::new(span, Either::Left(b))),
            (Err(_), Ok(Value { span, value: c })) => Ok(Value::new(span, Either::Right(c))),
            _ => Err(CliError::AmbiguousCommand),
        }
    }
}

impl<'a, C1, C2, A, B, C, R> Dispatchable<A, Either<B, C>, R> for OneOf<C1, C2>
where
    C1: Evaluatable<'a, A, B> + Dispatchable<A, B, R>,
    C2: Evaluatable<'a, A, C> + Dispatchable<A, C, R>,
{
    fn dispatch(self, flag_values: Value<Either<B, C>>) -> R {
        let span = flag_values.span;
        let values = flag_values.value;

        match values {
            Either::Left(b) => self.left.dispatch(Value::new(span, b)),
            Either::Right(c) => self.right.dispatch(Value::new(span, c)),
        }
    }
}

impl<'a, C1, C2, A, B, C, R> DispatchableWithArgs<A, Either<B, C>, R> for OneOf<C1, C2>
where
    C1: Evaluatable<'a, A, B> + DispatchableWithArgs<A, B, R>,
    C2: Evaluatable<'a, A, C> + DispatchableWithArgs<A, C, R>,
{
    fn dispatch_with_args(self, flag_values: Value<Either<B, C>>, args: Vec<String>) -> R {
        let span = flag_values.span;
        let values = flag_values.value;

        match values {
            Either::Left(b) => self.left.dispatch_with_args(Value::new(span, b), args),
            Either::Right(c) => self.right.dispatch_with_args(Value::new(span, c), args),
        }
    }
}

impl<'a, C1, C2, B, C, R> DispatchableWithHelpString<Either<B, C>, R> for OneOf<C1, C2>
where
    Self: Helpable<Output = String>,
    C1: DispatchableWithHelpString<B, R>,
    C2: DispatchableWithHelpString<C, R>,
{
    fn dispatch_with_helpstring(self, flag_values: Value<Either<B, C>>) -> R {
        let help_string = self.help();
        let span = flag_values.span;
        let values = flag_values.value;

        match values {
            Either::Left(b) => self
                .left
                .dispatch_with_supplied_helpstring(help_string, Value::new(span, b)),
            Either::Right(c) => self
                .right
                .dispatch_with_supplied_helpstring(help_string, Value::new(span, c)),
        }
    }

    fn dispatch_with_supplied_helpstring(
        self,
        help_string: String,
        flag_values: Value<Either<B, C>>,
    ) -> R {
        let span = flag_values.span;
        let values = flag_values.value;

        match values {
            Either::Left(b) => self
                .left
                .dispatch_with_supplied_helpstring(help_string, Value::new(span, b)),
            Either::Right(c) => self
                .right
                .dispatch_with_supplied_helpstring(help_string, Value::new(span, c)),
        }
    }
}

impl<C1, C2> ShortHelpable for OneOf<C1, C2>
where
    C1: ShortHelpable<Output = String>,
    C2: ShortHelpable<Output = String>,
{
    type Output = String;

    fn short_help(&self) -> Self::Output {
        format!("{}\n{}", self.left.short_help(), self.right.short_help())
    }
}

/// A marker trait to denote cmd-like objects from terminal objects.
pub trait IsCmd {}

/// Cmd represents an executable Cmd for the purpose of collating both flags
/// and a corresponding handler.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(0..3), ("foo".to_string(), "info".to_string()))),
///     Cmd::new("test")
///         .description("a test cmd")
///         .with_flag(
///             Flag::expect_string("name", "n", "A name.")
///                 .optional()
///                 .with_default("foo".to_string())
///         )
///         .with_flag(
///             Flag::expect_string(
///                 "log-level",
///                 "l",
///                 "A given log level setting.",
///             )
///         )
///         .with_handler(|(l, r)| {
///             format!("(Left: {}, Right: {})", &l, &r);
///         })
///         .evaluate(&["test", "-l", "info"][..])
/// )
/// ```
#[derive(Debug)]
pub struct Cmd<F, H> {
    name: &'static str,
    description: &'static str,
    author: &'static str,
    version: &'static str,
    flags: F,
    handler: H,
}

impl<F, H> IsCmd for Cmd<F, H> {}

impl Cmd<(), Box<dyn Fn()>> {
    /// Instantiates a new instance of `Cmd` with the name field set. All other
    /// fields will default to initial values (primarily empty strings).
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test");
    /// ```
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            description: "",
            author: "",
            version: "",
            flags: (),
            handler: Box::new(|| ()),
        }
    }
}

impl<H> Cmd<(), H> {
    /// Returns a new instance of `Cmd` with the type derived from the value of
    /// the passed Flag.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test")
    ///     .with_flag(
    ///         Flag::expect_string(
    ///             "log-level",
    ///             "l",
    ///             "A given log level setting.",
    ///         )
    ///     );
    /// ```
    pub fn with_flag<NF>(self, new_flag: NF) -> Cmd<NF, H> {
        Cmd {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            flags: new_flag,
            handler: self.handler,
        }
    }
}

impl<T, H> Cmd<T, H> {
    /// Returns Cmd with the name string set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test").name("other_test");
    /// ```
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Returns Cmd with the description string set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test").description("A test command.");
    /// ```
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Returns Cmd with the author string set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test").author("John Doe <jdoe@example.com");
    /// ```
    pub fn author(mut self, author: &'static str) -> Self {
        self.author = author;
        self
    }

    /// Returns Cmd with the version string set to the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test").version("1.0.0");
    /// ```
    pub fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }

    /// Returns Cmd with the handler set to the provided function in the format
    /// of `Fn(evaluator return) -> R`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test").with_handler(|_| ());
    /// ```
    pub fn with_handler<'a, A, B, NH, R>(self, handler: NH) -> Cmd<T, NH>
    where
        T: Evaluatable<'a, A, B>,
        NH: Fn(B) -> R,
    {
        Cmd {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            flags: self.flags,
            handler,
        }
    }

    /// Returns Cmd with the handler set to the provided function in the format
    /// of `Fn(evaluator return, Vec<String>) -> R`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test").with_args_handler(|(), _args| ());
    /// ```
    pub fn with_args_handler<'a, A, B, NH, R>(self, handler: NH) -> Cmd<T, NH>
    where
        T: Evaluatable<'a, A, B>,
        NH: Fn(B, Vec<String>) -> R,
    {
        Cmd {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            flags: self.flags,
            handler,
        }
    }

    /// Returns Cmd with the handler set to the provided function in the format
    /// of `Fn(helpstring, evaluator return) -> R`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test").with_helpstring_handler(|_helpstring, ()| ());
    /// ```
    pub fn with_helpstring_handler<'a, A, B, NH, R>(self, handler: NH) -> Cmd<T, NH>
    where
        T: Evaluatable<'a, A, B>,
        NH: Fn(String, B) -> R,
    {
        Cmd {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            flags: self.flags,
            handler,
        }
    }
}

impl<T, H> Cmd<T, H>
where
    T: IsFlag,
{
    /// Appends a flag to a given command.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Cmd::new("test")
    ///     .with_flag(
    ///         Flag::store_false("no-wait", "n", "don't wait for a response.")
    ///     );
    /// ```
    pub fn with_flag<NF>(self, new_flag: NF) -> Cmd<Join<T, NF>, H> {
        Cmd {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            flags: Join::new(self.flags, new_flag),
            handler: self.handler,
        }
    }
}

impl<'a, F, H, B> Evaluatable<'a, &'a [&'a str], B> for Cmd<F, H>
where
    B: std::fmt::Debug,
    F: Evaluatable<'a, &'a [&'a str], B>,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, B> {
        let filename = input
            .get(0)
            .map(|&bin| std::path::Path::new(bin).file_name());

        match filename {
            Some(Some(name)) if name == self.name => {
                // capture offset for binary.
                self.flags.evaluate(&input[1..]).map(|v| v.from_offset(1))
            }
            _ => Err(CliError::AmbiguousCommand),
        }
        // include binary in span range
        .map(|v| Value::new(Span::from_range(0..1).join(v.span), v.value))
    }
}

impl<F, H> ShortHelpable for Cmd<F, H> {
    type Output = String;

    fn short_help(&self) -> Self::Output {
        format!("{:<15} {}", self.name, self.description,)
    }
}

// Cmd has no flags
impl<H> Helpable for Cmd<(), H> {
    type Output = String;

    fn help(&self) -> Self::Output {
        format!(
            "Usage: {} [OPTIONS]\n{}\nFlags:\n",
            self.name, self.description,
        )
    }
}

impl<F, H> Helpable for Cmd<F, H>
where
    F: ShortHelpable<Output = FlagHelpCollector>,
{
    type Output = String;

    fn help(&self) -> Self::Output {
        format!(
            "Usage: {} [OPTIONS]\n{}\nFlags:\n{}",
            self.name,
            self.description,
            self.flags.short_help()
        )
    }
}

impl<'a, T, H, A, B, R> Dispatchable<A, B, R> for Cmd<T, H>
where
    T: Evaluatable<'a, A, B>,
    H: Fn(B) -> R,
{
    fn dispatch(self, flag_values: Value<B>) -> R {
        let inner = flag_values.unwrap();
        (self.handler)(inner)
    }
}

impl<'a, T, H, A, B, R> DispatchableWithArgs<A, B, R> for Cmd<T, H>
where
    T: Evaluatable<'a, A, B>,
    H: Fn(B, Vec<String>) -> R,
{
    fn dispatch_with_args(self, flag_values: Value<B>, args: Vec<String>) -> R {
        let inner = flag_values.unwrap();
        (self.handler)(inner, args)
    }
}

impl<'a, T, H, B, R> DispatchableWithHelpString<B, R> for Cmd<T, H>
where
    Self: Helpable<Output = String>,
    H: Fn(String, B) -> R,
{
    fn dispatch_with_helpstring(self, flag_values: Value<B>) -> R {
        let inner = flag_values.unwrap();
        let help_string = self.help();
        (self.handler)(help_string, inner)
    }

    fn dispatch_with_supplied_helpstring(self, help_string: String, flag_values: Value<B>) -> R {
        let inner = flag_values.unwrap();
        (self.handler)(help_string, inner)
    }
}

/// Defines behaviors for types that can dispatch an evaluator to a function.
pub trait Dispatchable<A, B, R> {
    fn dispatch(self, flag_values: Value<B>) -> R;
}

/// Defines behaviors for types that can dispatch an evaluator to a function.
/// with an optional set of unmatched arguments.
pub trait DispatchableWithArgs<A, B, R> {
    fn dispatch_with_args(self, flag_values: Value<B>, args: Vec<String>) -> R;
}

/// Defines behaviors for types that can dispatch an evaluator to a function
/// with additional help documentation.
pub trait DispatchableWithHelpString<B, R> {
    fn dispatch_with_helpstring(self, flag_values: Value<B>) -> R;
    fn dispatch_with_supplied_helpstring(self, help_string: String, flag_values: Value<B>) -> R;
}

/// Much like Helpable, ShortHelpable is for defining the functionality to
/// output short, summary, help strings for an implementign type. This is
/// often used when rolling up a type into an enclosing larger helpstring.
pub trait ShortHelpable
where
    Self::Output: std::fmt::Display,
{
    type Output;

    fn short_help(&self) -> Self::Output;
}

/// Helpable is for defining a method that outputs a helpstring for an
/// implementing type. This should be treated as a standalone helpstring not
/// meant to be composed with other sub-helpstrings.
pub trait Helpable
where
    Self::Output: std::fmt::Display,
{
    type Output;

    fn help(&self) -> Self::Output;
}

/// A marker trait to denote flag-like objects from terminal objects.
pub trait IsFlag {}

/// A constructor type to help with building flags. This should never be used
/// for anything but static method calls.
pub struct Flag;

impl IsFlag for Flag {}

impl Flag {
    /// Provides a convenient helper for generating an string evaluatable flag flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), "foo".to_string())),
    ///     Flag::expect_string("name", "n", "A name.")
    ///         .evaluate(&["test", "-n", "foo"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), "foo".to_string())),
    ///     FlagWithValue::new("name", "n", "A name.", StringValue)
    ///         .evaluate(&["test", "-n", "foo"][..])
    /// );
    /// ```
    pub fn expect_string(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<StringValue> {
        FlagWithValue::new(name, short_code, description, StringValue)
    }

    /// Provides a convenient helper for generating an StoreTrue flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..2), true)),
    ///     Flag::store_true("debug", "d", "Run command in debug mode.")
    ///         .evaluate(&["test", "-d"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..2), true)),
    ///     FlagWithValue::new("debug", "d", "Run command in debug mode.", ValueOnMatch::new(true))
    ///         .evaluate(&["test", "-d"][..])
    /// );
    /// ```
    pub fn store_true(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<ValueOnMatch<bool>> {
        FlagWithValue::new(name, short_code, description, ValueOnMatch::new(true))
    }

    /// Provides a convenient helper for generating an StoreFalse flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..2), false)),
    ///     Flag::store_false("no-wait", "n", "don't wait for a response.")
    ///         .evaluate(&["test", "-n"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..2), false)),
    ///     FlagWithValue::new("no-wait", "n", "don't wait for a response.", ValueOnMatch::new(false))
    ///         .evaluate(&["test", "-n"][..])
    /// );
    /// ```
    pub fn store_false(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<ValueOnMatch<bool>> {
        FlagWithValue::new(name, short_code, description, ValueOnMatch::new(false))
    }

    /// Provides a convenient helper for generating an ExpectI8Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_i8("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     FlagWithValue::new("timeout", "t", "A timeout.", I8Value)
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i8(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<I8Value> {
        FlagWithValue::new(name, short_code, description, I8Value)
    }

    /// Provides a convenient helper for generating an ExpectI16Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_i16("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     FlagWithValue::new("timeout", "t", "A timeout.", I16Value)
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i16(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<I16Value> {
        FlagWithValue::new(name, short_code, description, I16Value)
    }

    /// Provides a convenient helper for generating an ExpectI32Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_i32("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     FlagWithValue::new("timeout", "t", "A timeout.", I32Value)
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i32(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<I32Value> {
        FlagWithValue::new(name, short_code, description, I32Value)
    }

    /// Provides a convenient helper for generating an ExpectI64Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_i64("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     FlagWithValue::new("timeout", "t", "A timeout.", I64Value)
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i64(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<I64Value> {
        FlagWithValue::new(name, short_code, description, I64Value)
    }

    /// Provides a convenient helper for generating an ExpectU8Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_u8("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     FlagWithValue::new("timeout", "t", "A timeout.", U8Value)
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u8(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<U8Value> {
        FlagWithValue::new(name, short_code, description, U8Value)
    }

    /// Provides a convenient helper for generating an ExpectU16Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_u16("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     FlagWithValue::new("timeout", "t", "A timeout.", U16Value)
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u16(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<U16Value> {
        FlagWithValue::new(name, short_code, description, U16Value)
    }

    /// Provides a convenient helper for generating an ExpectU32Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_u32("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     FlagWithValue::new("timeout", "t", "A timeout.", U32Value)
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u32(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<U32Value> {
        FlagWithValue::new(name, short_code, description, U32Value)
    }

    /// Provides a convenient helper for generating an ExpectU64Value flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     Flag::expect_u64("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), 60)),
    ///     ExpectU64Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u64(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> FlagWithValue<U64Value> {
        FlagWithValue::new(name, short_code, description, U64Value)
    }

    /// Provides a convenient wrapper for generating `WithChoices` flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), "info".to_string())),
    ///     Flag::with_choices("log-level", "l", "A log level.", ["info".to_string(), "warn".to_string()], StringValue)
    ///         .evaluate(&["hello", "-l", "info"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(Value::new(Span::from_range(1..3), "info".to_string())),
    ///     WithChoices::new(
    ///         ["info".to_string(), "warn".to_string()],
    ///         FlagWithValue::new("log-level", "l", "A log level.", StringValue)
    ///     ).evaluate(&["hello", "-l", "info"][..])
    /// );
    /// ```
    pub fn with_choices<B, E, const N: usize>(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
        choices: [B; N],
        evaluator: E,
    ) -> WithChoices<B, FlagWithValue<E>, N> {
        WithChoices::new(
            choices,
            FlagWithValue::new(name, short_code, description, evaluator),
        )
    }
}

/// FlagHelpCollector provides a helper enum for collecting flag help strings
/// that are either derived from a single flag or joined flags.
pub enum FlagHelpCollector {
    Single(FlagHelpContext),
    Joined(Box<Self>, Box<Self>),
}

impl Default for FlagHelpCollector {
    fn default() -> Self {
        Self::Single(FlagHelpContext::default())
    }
}

impl std::fmt::Display for FlagHelpCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagHelpCollector::Single(fhc) => write!(f, "{}", fhc),
            FlagHelpCollector::Joined(lfhc, rfhc) => write!(f, "{}\n{}", lfhc, rfhc),
        }
    }
}

/// FlagHelpContext provides a type to store flag data that may be modified
/// through the course of generating a help string.
#[derive(Default)]
pub struct FlagHelpContext {
    name: &'static str,
    short_code: &'static str,
    description: &'static str,
    /// Additional String values to be appended after the description.
    modifiers: Vec<String>,
}

impl FlagHelpContext {
    /// Instantiates a new instance of FlagHelpContext.
    pub fn new(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
        modifiers: Vec<String>,
    ) -> Self {
        Self {
            name,
            short_code,
            description,
            modifiers,
        }
    }

    /// with_modifier returns an instances of FlagHelpContext with a provided
    /// modifier appended to the end of the modifiers vector.
    pub fn with_modifier(mut self, modifier: String) -> Self {
        self.modifiers.push(modifier);
        self
    }
}

impl std::fmt::Display for FlagHelpContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.is_empty() {
            write!(
                f,
                "    {:<16} {:<40}",
                format!("--{}, -{}", self.name, self.short_code),
                self.description,
            )
        } else {
            write!(
                f,
                "    {:<16} {:<40} [{}]",
                format!("--{}, -{}", self.name, self.short_code),
                self.description,
                self.modifiers
                    .iter()
                    .map(|modifier| format!("({})", modifier))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }
}

use core::ops::Range;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Span(Vec<usize>);

impl Span {
    pub const fn empty() -> Self {
        Span(vec![])
    }

    pub fn from_range(range: Range<usize>) -> Self {
        Self::from(range)
    }

    pub fn join(mut self, other: Span) -> Self {
        for v in other.0 {
            self.0.push(v)
        }

        self
    }
}

impl From<Range<usize>> for Span {
    fn from(src: Range<usize>) -> Self {
        let range = src.collect();
        Self(range)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value<T> {
    pub span: Span,
    pub value: T,
}

impl<T> Value<T> {
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }

    pub fn from_offset(self, offset: usize) -> Self {
        let adjusted_span_inner = self.span.0.iter().map(|v| *v + offset).collect();
        let span = Span(adjusted_span_inner);

        Self {
            span,
            value: self.value,
        }
    }

    pub fn unwrap(self) -> T {
        self.value
    }

    pub fn some(self) -> Option<T> {
        Some(self.value)
    }

    pub fn map<V, F>(self, map_fn: F) -> Value<V>
    where
        F: FnOnce(T) -> V,
    {
        let (span, value) = (self.span, self.value);
        Value::new(span, map_fn(value))
    }
}

/// Represents the result of an Evaluatable::evaluate call signifying whether
/// the call returned an error or correctly evaluated a flag to a type T.
pub type EvaluateResult<'a, T> = Result<Value<T>, CliError>;

/// A marker trait signifying that this implementation of Evaluatable is terminal.
pub trait TerminalEvaluatable<'a, A, B>: Evaluatable<'a, A, B> {}

/// Evaluatable provides methods for parsing and evaluating input values into a
/// corresponding concrete type.
pub trait Evaluatable<'a, A, B> {
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B>;

    fn join<E, C>(self, evaluator2: E) -> BoxedEvaluator<'a, A, (B, C)>
    where
        Self: Sized + BoxedEvaluatable<'a, A, B> + 'a,
        E: BoxedEvaluatable<'a, A, C> + 'a,
        A: Copy + 'a,
    {
        BoxedEvaluator::new(Join::<Self, E>::new(self, evaluator2))
    }
}

/// BoxedEvaluatable serves as a compound trait for the sake of combining the
/// Helpable and Evaluator traits.
pub trait BoxedEvaluatable<'a, A, B>:
    Evaluatable<'a, A, B> + ShortHelpable<Output = FlagHelpCollector>
{
}

impl<'a, A, B, T> BoxedEvaluatable<'a, A, B> for T where
    T: Evaluatable<'a, A, B> + ShortHelpable<Output = FlagHelpCollector> + 'a
{
}

/// BoxedEvaluator provides a wrapper for Evaluatable types.
pub struct BoxedEvaluator<'a, A, B> {
    evaluator: Box<dyn BoxedEvaluatable<'a, A, B> + 'a>,
}

impl<'a, A, B> IsFlag for BoxedEvaluator<'a, A, B> {}

impl<'a, A, B> BoxedEvaluator<'a, A, B> {
    pub fn new<E>(evaluator: E) -> Self
    where
        E: BoxedEvaluatable<'a, A, B> + 'a,
    {
        BoxedEvaluator {
            evaluator: Box::new(evaluator),
        }
    }
}

impl<'a, A, B> ShortHelpable for BoxedEvaluator<'a, A, B> {
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        self.evaluator.short_help()
    }
}

impl<'a, A, B> Evaluatable<'a, A, B> for BoxedEvaluator<'a, A, B> {
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B> {
        self.evaluator.evaluate(input)
    }
}

impl<'a, F, A, B> Evaluatable<'a, A, B> for F
where
    A: 'a,
    F: Fn(A) -> EvaluateResult<'a, B>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B> {
        self(input)
    }
}

/// Join provides a wrapper type for flag `Evaluators` allowing two evaluators
/// to be joined into a two return value. This join provides the basis for
/// compound or multiple flag values being passed upstream to a `Cmd`.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// let input = ["hello", "-n", "foo", "-l", "info"];
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..5), ("foo".to_string(), "info".to_string()))),
///     Join::new(
///         FlagWithValue::new("name", "n", "A name.", StringValue),
///         FlagWithValue::new("log-level", "l", "A given log level setting.", StringValue),
///     )
///     .evaluate(&input[..])
/// );
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..5), ("foo".to_string(), "info".to_string()))),
///     Flag::expect_string("name", "n", "A name.")
///         .join(FlagWithValue::new(
///             "log-level",
///             "l",
///             "A given log level setting.",
///             StringValue
///         ))
///         .evaluate(&input[..])
/// );
/// ```
#[derive(Debug)]
pub struct Join<E1, E2> {
    evaluator1: E1,
    evaluator2: E2,
}

impl<E1, E2> IsFlag for Join<E1, E2> {}

impl<E1, E2> Join<E1, E2> {
    /// Instantiates a new instance of Join with two given evaluators.
    pub fn new(evaluator1: E1, evaluator2: E2) -> Self {
        Self {
            evaluator1,
            evaluator2,
        }
    }
}

impl<'a, E1, E2, A, B, C> Evaluatable<'a, A, (B, C)> for Join<E1, E2>
where
    A: Copy + std::borrow::Borrow<A> + 'a,
    E1: Evaluatable<'a, A, B>,
    E2: Evaluatable<'a, A, C>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, (B, C)> {
        self.evaluator1
            .evaluate(input)
            .map_err(|e| e)
            .and_then(|e1_res| match self.evaluator2.evaluate(input) {
                Ok(e2_res) => {
                    let (e1_span, e1_val) = (e1_res.span, e1_res.value);
                    let (e2_span, e2_val) = (e2_res.span, e2_res.value);
                    let joined_span = e1_span.join(e2_span);

                    Ok(Value::new(joined_span, (e1_val, e2_val)))
                }
                Err(e) => Err(e),
            })
    }
}

impl<E1, E2> ShortHelpable for Join<E1, E2>
where
    E1: ShortHelpable<Output = FlagHelpCollector>,
    E2: ShortHelpable<Output = FlagHelpCollector>,
{
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        FlagHelpCollector::Joined(
            Box::new(self.evaluator1.short_help()),
            Box::new(self.evaluator2.short_help()),
        )
    }
}

/// A trait that signifies if a type can be assigned a default value. This
/// includes helper methods for assigning a type as optional and assigning a
/// default.
pub trait Defaultable
where
    Self: Sized,
{
    /// with_default returns a given type wrapped in a WithDefault with the
    /// provided default value. Functionally this is an alias for
    /// `WithDefault::new(self, default)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// FlagWithValue::new("name", "n", "A name.", StringValue).optional().with_default("foo".to_string());
    /// ```
    fn with_default<D>(self, default: D) -> WithDefault<D, Self> {
        WithDefault::new(default, self)
    }

    /// optional wraps a given type in an Optional struct. Functionally this
    /// is an alias for `Optional::new(self)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// FlagWithValue::new("name", "n", "A name.", StringValue).optional();
    /// ```
    fn optional(self) -> Optional<Self> {
        Optional::new(self)
    }
}

/// WithDefault takes an evaluator E and a default value B that agrees with the
/// return type of the Evaluator. This default is meant to wrap the enclosed
/// evaluator, returning the A success with the default value for any
/// evaluation that fails.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// let input = ["hello", "--log-level", "info"];
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(0..0), "foo".to_string())),
///     WithDefault::new(
///         "foo",
///         Optional::new(FlagWithValue::new("name", "n", "A name.", StringValue))
///     )
///     .evaluate(&input[..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(0..0), "foo".to_string())),
///     Flag::expect_string("name", "n", "A name.")
///         .optional()
///         .with_default("foo".to_string())
///         .evaluate(&input[..])
/// );
/// ```
#[derive(Debug)]
pub struct WithDefault<B, E> {
    default: B,
    evaluator: E,
}

impl<B, E> IsFlag for WithDefault<B, E> {}

impl<B, E> WithDefault<B, E> {
    /// Instantiates a new of WithDefault for a given type
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// WithDefault::<String, _>::new(
    ///     "foo",
    ///     Optional::new(FlagWithValue::new("name", "n", "A name.", StringValue))
    /// );
    /// ```
    pub fn new<D>(default: D, evaluator: E) -> Self
    where
        D: Into<B>,
    {
        Self {
            default: Into::<B>::into(default),
            evaluator,
        }
    }
}

impl<'a, E, A, B> Evaluatable<'a, A, B> for WithDefault<B, E>
where
    A: 'a,
    B: Clone,
    E: Evaluatable<'a, A, Option<B>>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B> {
        self.evaluator
            .evaluate(input)
            .map(|op| op.map(|opt| opt.unwrap_or_else(|| self.default.clone())))
    }
}

impl<B, E> ShortHelpable for WithDefault<B, E>
where
    B: Clone + std::fmt::Debug,
    E: ShortHelpable<Output = FlagHelpCollector> + Defaultable,
{
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        match self.evaluator.short_help() {
            FlagHelpCollector::Single(fhc) => FlagHelpCollector::Single(
                fhc.with_modifier(format!("default: {:?}", self.default.clone())),
            ),
            // this case should never be hit as joined is not defaultable
            fhcj @ FlagHelpCollector::Joined(_, _) => fhcj,
        }
    }
}

/// Optional wraps an evaluator, for the purpose of transforming the enclosed
/// evaluator from an `Evaluator<A, B>` to an `Evaluator<A, Option<B>>` where
/// the success state of the evaluation is capture in the value of the
/// `Option<B>`.
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// let input = ["hello", "-n", "foo"];
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..3), Some("foo".to_string()))),
///     Optional::new(FlagWithValue::new("name", "n", "A name.", StringValue)).evaluate(&input[..])
/// );
///
/// // validate boxed syntax works
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..3), Some("foo".to_string()))),
///     FlagWithValue::new("name", "n", "A name.", StringValue)
///         .optional()
///         .evaluate(&input[..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::empty(), None)),
///     Optional::new(FlagWithValue::new(
///         "log-level",
///         "l",
///         "A given log level setting.",
///         StringValue
///     ))
///     .evaluate(&input[..])
/// );
/// ```
#[derive(Debug)]
pub struct Optional<E> {
    evaluator: E,
}

impl<E> IsFlag for Optional<E> {}

impl<E> Defaultable for Optional<E> where E: Defaultable {}

impl<E> Optional<E> {
    /// Instantiates a new instance of Optional.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// Optional::new(FlagWithValue::new("name", "n", "A name.", StringValue));
    /// ```
    pub fn new(evaluator: E) -> Self {
        Self { evaluator }
    }
}

impl<'a, E, A, B> Evaluatable<'a, A, Option<B>> for Optional<E>
where
    A: 'a,
    E: Evaluatable<'a, A, B>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, Option<B>> {
        match self.evaluator.evaluate(input).ok() {
            Some(Value { span, value }) => Ok(Value::new(span, Some(value))),
            None => Ok(Value::new(Span::default(), None)),
        }
    }
}

impl<E> ShortHelpable for Optional<E>
where
    E: ShortHelpable<Output = FlagHelpCollector>,
{
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        match self.evaluator.short_help() {
            FlagHelpCollector::Single(fhc) => {
                FlagHelpCollector::Single(fhc.with_modifier("optional".to_string()))
            }
            // this case should never be hit as joined is not defaultable
            fhcj @ FlagHelpCollector::Joined(_, _) => fhcj,
        }
    }
}

/// WithChoices takes an evaluator E and a default value B that agrees with the
/// return type of the Evaluator. This default is meant to wrap the enclosed
/// evaluator, returning the A success with the default value for any
/// evaluation that fails.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// let input = ["hello", "--log-level", "info"];
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..3), "info".to_string())),
///     Flag::with_choices(
///         "log-level", "l", "logging level",
///         ["info".to_string(), "warn".to_string()],
///         StringValue
///     )
///     .evaluate(&input[..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..3), "info".to_string())),
///     WithChoices::new(
///         ["info".to_string(), "warn".to_string()],
///         FlagWithValue::new("log-level", "l", "logging level", StringValue)
///     )
///     .evaluate(&input[..])
/// );
///
/// assert!(
///     WithChoices::new(
///         ["error".to_string()],
///         FlagWithValue::new("log-level", "l", "logging level", StringValue)
///     )
///     .evaluate(&input[..]).is_err()
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::default(), "debug".to_string())),
///     WithDefault::new(
///         "debug".to_string(),
///         Optional::new(WithChoices::new(
///             ["error".to_string()],
///             FlagWithValue::new("log-level", "l", "logging level", StringValue)
///         ))
///     )
///     .evaluate(&input[..])
/// );
/// ```
#[derive(Debug)]
pub struct WithChoices<B, E, const N: usize> {
    choices: [B; N],
    evaluator: E,
}

impl<B, E, const N: usize> IsFlag for WithChoices<B, E, N> {}

#[allow(deprecated)]
impl<B, E, const N: usize> Defaultable for WithChoices<B, E, N> where E: Defaultable {}

impl<B, E, const N: usize> WithChoices<B, E, N> {
    /// Instantiates a new choices wrapper on an evaluator.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// WithChoices::new(
    ///     ["foo".to_string(), "bar".to_string()],
    ///     Optional::new(FlagWithValue::new("name", "n", "A name.", StringValue))
    /// );
    /// ```
    pub fn new(choices: [B; N], evaluator: E) -> Self {
        Self { choices, evaluator }
    }
}

impl<'a, E, A, B, const N: usize> Evaluatable<'a, A, B> for WithChoices<B, E, N>
where
    A: 'a,
    B: Clone + PartialEq,
    E: Evaluatable<'a, A, B>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B> {
        self.evaluator.evaluate(input).and_then(|op| {
            self.choices
                .iter()
                .any(|choice| choice == &op.value)
                .then(|| op)
                .ok_or(CliError::ValueEvaluation)
        })
    }
}

impl<B, E, const N: usize> ShortHelpable for WithChoices<B, E, N>
where
    B: Clone + std::fmt::Debug,
    E: ShortHelpable<Output = FlagHelpCollector> + Defaultable,
{
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        match self.evaluator.short_help() {
            FlagHelpCollector::Single(fhc) => {
                FlagHelpCollector::Single(fhc.with_modifier(format!("choices: {:?}", self.choices)))
            }
            // this case should never be hit as joined is not defaultable
            fhcj @ FlagHelpCollector::Joined(_, _) => fhcj,
        }
    }
}

/// ExpectStringValue represents a terminal flag type, returning the next string value passed.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///    Ok(Value::new(Span::from_range(1..3), "foo".to_string())),
///    ExpectStringValue::new("name", "n", "A name.").evaluate(&["hello", "--name", "foo"][..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..3), "foo".to_string())),
///     ExpectStringValue::new("name", "n", "A name.").evaluate(&["hello", "-n", "foo"][..])
/// );
/// ```
#[deprecated]
#[derive(Debug)]
pub struct ExpectStringValue {
    inner: FlagWithValue<StringValue>,
}

#[allow(deprecated)]
impl IsFlag for ExpectStringValue {}

#[allow(deprecated)]
impl ExpectStringValue {
    /// Instantiates a new instance of ExpectStringValue with a given flag name,
    /// shortcode and description.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// ExpectStringValue::new("name", "n", "A name.");
    /// ```
    #[allow(dead_code)]
    pub fn new(name: &'static str, short_code: &'static str, description: &'static str) -> Self {
        Self {
            inner: FlagWithValue::new(name, short_code, description, StringValue),
        }
    }
}

#[allow(deprecated)]
impl Defaultable for ExpectStringValue {}

#[allow(deprecated)]
impl<'a> Evaluatable<'a, &'a [&'a str], String> for ExpectStringValue {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, String> {
        self.inner.evaluate(input)
    }
}

#[allow(deprecated)]
impl ShortHelpable for ExpectStringValue {
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        self.inner.short_help()
    }
}

/// StoreTrue represents a terminal flag type, returning a boolean set to true if set.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///    Ok(Value::new(Span::from_range(1..2), true)),
///    StoreTrue::new("debug", "d", "Run in debug mode.").evaluate(&["hello", "--debug"][..])
/// );
///
/// assert_eq!(
///    Ok(Value::new(Span::from_range(1..2), true)),
///    StoreTrue::new("debug", "d", "Run in debug mode.").evaluate(&["hello", "-d"][..])
/// );
///
/// assert_eq!(
///    Ok(Value::new(Span::empty(), false)),
///    WithDefault::new(
///        false,
///        Optional::new(StoreTrue::new("debug", "d", "Run in debug mode."))
///    )
///    .evaluate(&["hello"][..])
/// );
/// ```
#[deprecated]
#[derive(Debug)]
pub struct StoreTrue {
    inner: FlagWithValue<ValueOnMatch<bool>>,
}

#[allow(deprecated)]
impl IsFlag for StoreTrue {}

#[allow(deprecated)]
impl Defaultable for StoreTrue {}

#[allow(deprecated)]
impl StoreTrue {
    /// Instantiates a new instance of StoreTrue with a given flag name,
    /// shortcode and description.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// StoreTrue::new("debug", "d", "Run a command in debug mode.");
    /// ```
    #[allow(dead_code)]
    pub fn new(name: &'static str, short_code: &'static str, description: &'static str) -> Self {
        Self {
            inner: FlagWithValue::new(name, short_code, description, ValueOnMatch::new(true)),
        }
    }
}

#[allow(deprecated)]
impl<'a> Evaluatable<'a, &'a [&'a str], bool> for StoreTrue {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, bool> {
        self.inner.evaluate(input)
    }
}

#[allow(deprecated)]
impl ShortHelpable for StoreTrue {
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        self.inner.short_help()
    }
}

/// StoreFalse represents a terminal flag type, returning a boolean set to false if set.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..2), false)),
///     StoreFalse::new("no-wait", "n", "don't wait for a response.").evaluate(&["hello", "--no-wait"][..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..2), false)),
///     StoreFalse::new("no-wait", "n", "don't wait for a response.").evaluate(&["hello", "-n"][..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::empty(), true)),
///     WithDefault::new(
///         true,
///         Optional::new(StoreFalse::new("no-wait", "n", "don't wait for a response."))
///     )
///     .evaluate(&["hello"][..])
/// );
/// ```
#[deprecated]
#[derive(Debug)]
pub struct StoreFalse {
    inner: FlagWithValue<ValueOnMatch<bool>>,
}

#[allow(deprecated)]
impl IsFlag for StoreFalse {}

#[allow(deprecated)]
impl Defaultable for StoreFalse {}

#[allow(deprecated)]
impl StoreFalse {
    /// Instantiates a new instance of StoreFalse with a given flag name,
    /// shortcode and description.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// StoreFalse::new("no-wait", "n", "don't wait for a response.");
    /// ```
    #[allow(dead_code)]
    pub fn new(name: &'static str, short_code: &'static str, description: &'static str) -> Self {
        Self {
            inner: FlagWithValue::new(name, short_code, description, ValueOnMatch::new(false)),
        }
    }
}

#[allow(deprecated)]
impl<'a> Evaluatable<'a, &'a [&'a str], bool> for StoreFalse {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, bool> {
        self.inner.evaluate(input)
    }
}

#[allow(deprecated)]
impl ShortHelpable for StoreFalse {
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        self.inner.short_help()
    }
}

// Integer types

macro_rules! generate_integer_evaluators {
    ($($name:tt, $value_name:tt, $primitive:ty,)*) => {
        $(
        #[deprecated]
        #[derive(Debug)]
        pub struct $name {
            inner: FlagWithValue<$value_name>,
        }

        #[allow(deprecated)]
        impl IsFlag for $name {}

        #[allow(deprecated)]
        impl Defaultable for $name {}

        #[allow(deprecated)]
        impl $name {
            #[allow(dead_code)]
            pub fn new(
                name: &'static str,
                short_code: &'static str,
                description: &'static str,
            ) -> Self {
                Self {
                    inner:FlagWithValue::new(name, short_code, description, $value_name),
                }
            }
        }

        #[allow(deprecated)]
        impl<'a> Evaluatable<'a, &'a [&'a str], $primitive> for $name {
            fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, $primitive> {
                self.inner.evaluate(input)
            }
        }

        #[allow(deprecated)]
        impl ShortHelpable for $name {
            type Output = FlagHelpCollector;

            fn short_help(&self) -> Self::Output {
                self.inner.short_help()
            }
        }

        /// Represents a Numeric argument
        #[derive(Debug, Clone, Copy)]
        pub struct $value_name;

        impl<'a> PositionalArgumentValue<'a, &'a [&'a str], $primitive> for $value_name {
            fn evaluate_at(&self, input: &'a [&'a str], pos: usize) -> EvaluateResult<'a, $primitive> {
                self.evaluate(&input[pos..])
            }
        }

        impl<'a> Evaluatable<'a, &'a [&'a str], $primitive> for $value_name {
            fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, $primitive> {
                let result = input
                    .get(0)
                    .and_then(|&v| v.parse::<$primitive>().ok())
                    .ok_or(CliError::ValueEvaluation);

               result.map(|matching_int| Value::new(Span::from_range(0..1), matching_int))
            }
        }

        impl<'a> TerminalEvaluatable<'a, &'a [&'a str], $primitive> for $value_name {}
    )*
    };
}

#[rustfmt::skip]
generate_integer_evaluators!(
    ExpectI8Value, I8Value, i8,
    ExpectI16Value, I16Value, i16,
    ExpectI32Value, I32Value, i32,
    ExpectI64Value, I64Value, i64,
    ExpectU8Value, U8Value, u8,
    ExpectU16Value, U16Value, u16,
    ExpectU32Value, U32Value, u32,
    ExpectU64Value, U64Value, u64,
);

/// Defines a marker trait for types that can be opened via the WithOpen
/// evaluator.
pub trait Openable {}

/// WithOpen represents an evaluator that can take a filepath as parsed by
/// `ExpectFilePath` and return an opened file handler for said path. Function
/// this works much like `WithDefault` in that it is an optional augmentation
/// for an existing evaluator.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
/// use std::fs::File;
///
/// assert!(
///     WithOpen::new(
///         ExpectFilePath::new("file", "f", "A file to open", true, false, true)
///     ).evaluate(&["hello", "--file", "/etc/hostname"][..]).is_ok()
/// );
///
/// assert!(
///     WithOpen::new(
///         ExpectFilePath::new("file", "f", "A file to open", true, false, true)
///     ).evaluate(&["hello", "-f", "/etc/hostname"][..]).is_ok()
/// );
///
/// assert!(
///     WithOpen::new(
///         ExpectFilePath::new("file", "f", "A file to open", true, false, true)
///     ).evaluate(&["hello"][..]).is_err()
/// );
/// ```
#[derive(Debug)]
pub struct WithOpen<E> {
    evaluator: E,
}

impl<E> IsFlag for WithOpen<E> {}

impl<E> WithOpen<E> {
    /// Instantiates a new of WithOpen for a given type
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// WithOpen::new(
    ///     ExpectFilePath::new("file", "f", "A file to open", true, false, true)
    /// );
    /// ```
    pub fn new(evaluator: E) -> Self {
        Self { evaluator }
    }
}

impl<'a, E> Evaluatable<'a, &'a [&'a str], std::fs::File> for WithOpen<E>
where
    E: Evaluatable<'a, &'a [&'a str], String> + Openable,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, std::fs::File> {
        self.evaluator.evaluate(input).and_then(|vfp| {
            std::fs::File::open(&vfp.value)
                .map_err(|e| {
                    CliError::FlagEvaluation(format!("unable to open file evaluator: {}", e))
                })
                .map(|f| Value::new(vfp.span, f))
        })
    }
}

impl<E> ShortHelpable for WithOpen<E>
where
    E: ShortHelpable<Output = FlagHelpCollector> + Defaultable,
{
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        match self.evaluator.short_help() {
            FlagHelpCollector::Single(fhc) => {
                FlagHelpCollector::Single(fhc.with_modifier("will_open".to_string()))
            }
            // this case should never be hit as joined is not defaultable
            fhcj => fhcj,
        }
    }
}

/// ExpectFilePath represents a terminal flag type, that parses and validates a
/// file exists in a path. Returning the file path as a String.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..3), "/etc/hostname".to_string())),
///     ExpectFilePath::new("file", "f", "A filepath to read", true, false, true).evaluate(&["hello", "--file", "/etc/hostname"][..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::empty(), "/etc/hostname".to_string())),
///     WithDefault::new(
///         "/etc/hostname".to_string(),
///         Optional::new(ExpectFilePath::new("file", "f", "A filepath to read", true, false, true))
///     )
///     .evaluate(&["hello"][..])
/// );
/// ```
#[deprecated]
#[derive(Debug)]
pub struct ExpectFilePath {
    inner: FlagWithValue<FileValue>,
}

#[allow(deprecated)]
impl IsFlag for ExpectFilePath {}

#[allow(deprecated)]
impl ExpectFilePath {
    /// Instantiates a new instance of ExpectFilePath with a given flag name,
    /// shortcode and description.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// ExpectFilePath::new("file", "f", "A file name.", true, false, true);
    /// ```
    #[allow(dead_code)]
    pub fn new(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
        readable: bool,
        writable: bool,
        exists: bool,
    ) -> Self {
        Self {
            inner: FlagWithValue::new(
                name,
                short_code,
                description,
                FileValue::new(readable, writable, exists),
            ),
        }
    }
}

#[allow(deprecated)]
impl Openable for ExpectFilePath {}

#[allow(deprecated)]
impl Defaultable for ExpectFilePath {}

#[allow(deprecated)]
impl<'a> Evaluatable<'a, &'a [&'a str], String> for ExpectFilePath {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, String> {
        self.inner.evaluate(input)
    }
}

#[allow(deprecated)]
impl ShortHelpable for ExpectFilePath {
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        self.inner.short_help()
    }
}

// Unit type

// This implementation exists mostly for cases where a Cmd, or SubCommands
// object has no flags associated with it.
impl<'a> Evaluatable<'a, &'a [&'a str], ()> for () {
    fn evaluate(&self, _: &'a [&'a str]) -> EvaluateResult<'a, ()> {
        Ok(Value::new(Span::from_range(0..1), ()))
    }
}

#[derive(Debug)]
pub struct FlagWithValue<V> {
    name: &'static str,
    short_code: &'static str,
    description: &'static str,
    value: V,
}

impl<V> IsFlag for FlagWithValue<V> {}

impl<V> FlagWithValue<V> {
    /// Instantiates a new instance of FlagWithValue with a given flag name,
    /// shortcode and description.
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// FlagWithValue::new("name", "n", "A name.", StringValue);
    /// ```
    #[allow(dead_code)]
    pub fn new(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
        value: V,
    ) -> Self {
        Self {
            name,
            short_code,
            description,
            value,
        }
    }
}

impl<V> Defaultable for FlagWithValue<V> {}

impl<V> Openable for FlagWithValue<V> where V: Openable {}

impl<'a, V, B> Evaluatable<'a, &'a [&'a str], B> for FlagWithValue<V>
where
    V: PositionalArgumentValue<'a, &'a [&'a str], B>,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, B> {
        input[..]
            .iter()
            .enumerate()
            .find(|(_, &arg)| {
                (arg == format!("{}{}", "--", self.name))
                    || (arg == format!("{}{}", "-", self.short_code))
            })
            // Only need the index.
            .map(|(idx, _)| idx)
            .and_then(|idx| {
                self.value
                    .evaluate_at(input, idx + 1)
                    .map(|val| val.from_offset(idx + 1))
                    .map(|v| {
                        let span = v.span;
                        let adjusted = Span::from_range(idx..idx + 1).join(span);
                        Value::new(adjusted, v.value)
                    })
                    .ok()
            })
            .ok_or_else(|| CliError::FlagEvaluation(self.name.to_string()))
    }
}

impl<V> ShortHelpable for FlagWithValue<V> {
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        FlagHelpCollector::Single(FlagHelpContext::new(
            self.name,
            self.short_code,
            self.description,
            Vec::new(),
        ))
    }
}

/// PositionalArgumentValue Provides a value type for evaluating positionally.
pub trait PositionalArgumentValue<'a, A, B>: Evaluatable<'a, A, B> {
    fn evaluate_at(&self, input: A, pos: usize) -> EvaluateResult<'a, B>;
}

/// Represents a String argument
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///    Ok(Value::new(Span::from_range(1..3), "foo".to_string())),
///    FlagWithValue::new("name", "n", "A name.", StringValue).evaluate(&["hello", "--name", "foo"][..])
/// );
///
/// assert_eq!(
///    Ok(Value::new(Span::from_range(1..3), "foo".to_string())),
///    FlagWithValue::new("name", "n", "A name.", StringValue).evaluate(&["hello", "-n", "foo"][..])
/// );
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StringValue;

impl<'a> PositionalArgumentValue<'a, &'a [&'a str], String> for StringValue {
    fn evaluate_at(&self, input: &'a [&'a str], pos: usize) -> EvaluateResult<'a, String> {
        self.evaluate(&input[pos..])
    }
}

impl<'a> Evaluatable<'a, &'a [&'a str], String> for StringValue {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, String> {
        input
            .get(0)
            .map(|v| Value::new(Span::from_range(0..1), v.to_string()))
            .ok_or(CliError::ValueEvaluation)
    }
}

impl<'a> TerminalEvaluatable<'a, &'a [&'a str], String> for StringValue {}

/// ValueOnMatch represents a terminal flag type, returning a given value on a match.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..2), false)),
///     FlagWithValue::new("no-wait", "n", "don't wait for a response.", ValueOnMatch::new(false))
///         .evaluate(&["hello", "--no-wait"][..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..2), false)),
///     FlagWithValue::new("no-wait", "n", "don't wait for a response.", ValueOnMatch::new(false))
///         .evaluate(&["hello", "-n"][..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::empty(), true)),
///     WithDefault::new(
///         true,
///         Optional::new(FlagWithValue::new("no-wait", "n", "don't wait for a response.", ValueOnMatch::new(false)))
///     )
///     .evaluate(&["hello"][..])
/// );
/// ```
#[derive(Debug)]
pub struct ValueOnMatch<V> {
    value: V,
}

impl<V> ValueOnMatch<V> {
    pub fn new(value: V) -> Self {
        Self { value }
    }
}

impl<'a, V: Clone> PositionalArgumentValue<'a, &'a [&'a str], V> for ValueOnMatch<V> {
    fn evaluate_at(&self, input: &'a [&'a str], pos: usize) -> EvaluateResult<'a, V> {
        self.evaluate(&input[pos..])
    }
}

impl<'a, V: Clone> Evaluatable<'a, &'a [&'a str], V> for ValueOnMatch<V> {
    fn evaluate(&self, _: &'a [&'a str]) -> EvaluateResult<'a, V> {
        Ok(Value::new(Span::empty(), self.value.clone()))
    }
}

impl<'a, V: Clone> TerminalEvaluatable<'a, &'a [&'a str], V> for ValueOnMatch<V> {}

/// FileValue represents a terminal flag type, that parses and validates a
/// file exists in a path. Returning the file path as a String.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///     Ok(Value::new(Span::from_range(1..3), "/etc/hostname".to_string())),
///     FlagWithValue::new("file", "f", "A filepath to read", FileValue::new(true, false, true))
///         .evaluate(&["hello", "--file", "/etc/hostname"][..])
/// );
///
/// assert_eq!(
///     Ok(Value::new(Span::empty(), "/etc/hostname".to_string())),
///     WithDefault::new(
///         "/etc/hostname".to_string(),
///         Optional::new(FlagWithValue::new("file", "f", "A filepath to read", FileValue::new(true, false, true)))
///     )
///     .evaluate(&["hello"][..])
/// );
/// ```
#[derive(Debug, Clone, Copy)]
pub struct FileValue {
    readable: bool,
    writable: bool,
    exists: bool,
}

impl IsFlag for FileValue {}

impl FileValue {
    /// Instantiates a new instance of FileArgumen.t
    ///
    /// # Example
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// FileValue::new(true, false, true);
    /// ```
    #[allow(dead_code)]
    pub fn new(readable: bool, writable: bool, exists: bool) -> Self {
        Self {
            readable,
            writable,
            exists,
        }
    }
}

impl Openable for FileValue {}

impl Defaultable for FileValue {}

impl<'a> PositionalArgumentValue<'a, &'a [&'a str], String> for FileValue {
    fn evaluate_at(&self, input: &'a [&'a str], pos: usize) -> EvaluateResult<'a, String> {
        self.evaluate(&input[pos..])
    }
}

impl<'a> Evaluatable<'a, &'a [&'a str], String> for FileValue {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, String> {
        use std::fs::OpenOptions;

        input
            .get(0)
            // check if the file exists with the corresponding flags.
            .and_then(|p| {
                OpenOptions::new()
                    .read(self.readable)
                    .write(self.writable)
                    .create(!self.exists)
                    .open(p)
                    .ok()
                    .map(|_| p)
            })
            .map(|&v| Value::new(Span::from_range(0..1), v.to_owned()))
            .ok_or(CliError::ValueEvaluation)
    }
}

impl<'a> TerminalEvaluatable<'a, &'a [&'a str], String> for FileValue {}

/// Returns all unused args from an evaluated source.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// let input = ["hello", "a", "-n", "foo", "b", "c", "1"];
///
/// let evaluated_res = Cmd::new("hello")
///     .with_flag(FlagWithValue::new("name", "n", "A name.", StringValue))
///     .evaluate(&input[..]);
///
/// let val_with_args = evaluated_res.map(|flags| {
///     let args = return_unused_args(&input[..], &flags.span);
///     (flags, args)
/// });
///
/// let expected_span = Span::from_range(0..1).join(Span::from_range(2..4));
/// let expected_args = vec!["a", "b", "c", "1"].iter().map(|v| v.to_string()).collect();
/// assert_eq!(
///     Ok((Value::new(expected_span, "foo".to_string()), expected_args)),
///     val_with_args
/// );
/// ```
pub fn return_unused_args<'a>(input: &'a [&'a str], matched_span: &Span) -> Vec<String> {
    let span = &matched_span.0;
    input
        .iter()
        .enumerate()
        .filter(|(offset, _)| !span.contains(offset))
        .map(|(_, v)| v.to_string())
        .collect()
}
