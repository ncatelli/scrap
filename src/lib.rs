pub mod prelude;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum CliError {
    AmbiguousCommand,
    FlagEvaluation(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AmbiguousCommand => write!(f, "ambiguous command"),
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
///     Ok(Either::Left("test".to_string())),
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
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<B> {
        match input
            .get(0)
            .map(|&bin| std::path::Path::new(bin).file_name())
        {
            Some(Some(name)) if name == self.name => self.commands.evaluate(&input[1..]),
            _ => Err(CliError::AmbiguousCommand),
        }
    }
}

impl<'a, C, A, B, R> Dispatchable<A, B, R> for CmdGroup<C>
where
    C: Evaluatable<'a, A, B> + Dispatchable<A, B, R>,
{
    fn dispatch(self, flag_values: B) -> R {
        self.commands.dispatch(flag_values)
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
///     Ok(Either::Left("test".to_string())),
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
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<Either<B, C>> {
        match (self.left.evaluate(&input), self.right.evaluate(&input)) {
            (Ok(b), Err(_)) => Ok(Either::Left(b)),
            (Err(_), Ok(c)) => Ok(Either::Right(c)),
            _ => Err(CliError::AmbiguousCommand),
        }
    }
}

impl<'a, C1, C2, A, B, C, R> Dispatchable<A, Either<B, C>, R> for OneOf<C1, C2>
where
    C1: Evaluatable<'a, A, B> + Dispatchable<A, B, R>,
    C2: Evaluatable<'a, A, C> + Dispatchable<A, C, R>,
{
    fn dispatch(self, flag_values: Either<B, C>) -> R {
        match flag_values {
            Either::Left(b) => self.left.dispatch(b),
            Either::Right(c) => self.right.dispatch(c),
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
///     Ok(("foo".to_string(), "info".to_string())),
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

    /// Returns Cmd with the handler set to the provided function.
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
    F: Evaluatable<'a, &'a [&'a str], B>,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<B> {
        match input
            .get(0)
            .map(|&bin| std::path::Path::new(bin).file_name())
        {
            Some(Some(name)) if name == self.name => self.flags.evaluate(&input[1..]),
            _ => Err(CliError::AmbiguousCommand),
        }
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
            self.flags.short_help().to_string()
        )
    }
}

impl<'a, T, H, A, B, R> Dispatchable<A, B, R> for Cmd<T, H>
where
    T: Evaluatable<'a, A, B>,
    H: Fn(B) -> R,
{
    fn dispatch(self, flag_values: B) -> R {
        (self.handler)(flag_values)
    }
}

pub trait Dispatchable<A, B, R> {
    fn dispatch(self, flag_values: B) -> R;
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
    /// Provides a convenient helper for generating an ExpectStringValue flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrap::prelude::v1::*;
    /// use scrap::*;
    ///
    /// assert_eq!(
    ///     Ok("foo".to_string()),
    ///     Flag::expect_string("name", "n", "A name.")
    ///         .evaluate(&["test", "-n", "foo"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok("foo".to_string()),
    ///     ExpectStringValue::new("name", "n", "A name.")
    ///         .evaluate(&["test", "-n", "foo"][..])
    /// );
    /// ```
    pub fn expect_string(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectStringValue {
        ExpectStringValue::new(name, short_code, description)
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
    ///     Ok(true),
    ///     Flag::store_true("debug", "d", "Run command in debug mode.")
    ///         .evaluate(&["test", "-d"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(true),
    ///     StoreTrue::new("debug", "d", "Run command in debug mode.")
    ///         .evaluate(&["test", "-d"][..])
    /// );
    /// ```
    pub fn store_true(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> StoreTrue {
        StoreTrue::new(name, short_code, description)
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
    ///     Ok(false),
    ///     Flag::store_false("no-wait", "n", "don't wait for a response.")
    ///         .evaluate(&["test", "-n"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(false),
    ///     StoreFalse::new("no-wait", "n", "don't wait for a response." )
    ///         .evaluate(&["test", "-n"][..])
    /// );
    /// ```
    pub fn store_false(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> StoreFalse {
        StoreFalse::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_i8("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectI8Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i8(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectI8Value {
        ExpectI8Value::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_i16("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectI16Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i16(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectI16Value {
        ExpectI16Value::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_i32("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectI32Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i32(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectI32Value {
        ExpectI32Value::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_i64("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectI64Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_i64(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectI64Value {
        ExpectI64Value::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_u8("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectU8Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u8(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectU8Value {
        ExpectU8Value::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_u16("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectU16Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u16(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectU16Value {
        ExpectU16Value::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_u32("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectU32Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u32(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectU32Value {
        ExpectU32Value::new(name, short_code, description)
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
    ///     Ok(60),
    ///     Flag::expect_u64("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    ///
    /// assert_eq!(
    ///     Ok(60),
    ///     ExpectU64Value::new("timeout", "t", "A timeout.")
    ///         .evaluate(&["test", "-t", "60"][..])
    /// );
    /// ```
    pub fn expect_u64(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectU64Value {
        ExpectU64Value::new(name, short_code, description)
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

/// Represents the result of an Evaluatable::evaluate call signifying whether
/// the call returned an error or correctly evaluated a flag to a type T.
pub type EvaluateResult<'a, T> = Result<T, CliError>;

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
///     Ok(("foo".to_string(), "info".to_string())),
///     Join::new(
///         ExpectStringValue::new("name", "n", "A name."),
///         ExpectStringValue::new("log-level", "l", "A given log level setting."),
///     )
///     .evaluate(&input[..])
/// );
/// assert_eq!(
///     Ok(("foo".to_string(), "info".to_string())),
///     Flag::expect_string("name", "n", "A name.")
///         .join(ExpectStringValue::new(
///             "log-level",
///             "l",
///             "A given log level setting."
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
                Ok(e2_res) => Ok((e1_res, e2_res)),
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
    /// ExpectStringValue::new("name", "n", "A name.").optional().with_default("foo".to_string());
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
    /// ExpectStringValue::new("name", "n", "A name.").optional();
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
///     Ok("foo".to_string()),
///     WithDefault::new(
///         "foo",
///         Optional::new(ExpectStringValue::new("name", "n", "A name."))
///     )
///     .evaluate(&input[..])
/// );
///
/// assert_eq!(
///     Ok("foo".to_string()),
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
    ///     Optional::new(ExpectStringValue::new("name", "n", "A name."))
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
            .map(|op| op.unwrap_or_else(|| self.default.clone()))
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
///     Ok(Some("foo".to_string())),
///     Optional::new(ExpectStringValue::new("name", "n", "A name.")).evaluate(&input[..])
/// );
///
/// // validate boxed syntax works
/// assert_eq!(
///     Ok(Some("foo".to_string())),
///     ExpectStringValue::new("name", "n", "A name.")
///         .optional()
///         .evaluate(&input[..])
/// );
///
/// assert_eq!(
///     Ok(None),
///     Optional::new(ExpectStringValue::new(
///         "log-level",
///         "l",
///         "A given log level setting."
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
    /// Optional::new(ExpectStringValue::new("name", "n", "A name."));
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
        Ok(self.evaluator.evaluate(input).ok())
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

/// ExpectStringValue represents a terminal flag type, returning the next string value passed.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///    Ok("foo".to_string()),
///    ExpectStringValue::new("name", "n", "A name.").evaluate(&["hello", "--name", "foo"][..])
/// );
///
/// assert_eq!(
///     Ok("foo".to_string()),
///     ExpectStringValue::new("name", "n", "A name.").evaluate(&["hello", "-n", "foo"][..])
/// );
/// ```
#[derive(Debug)]
pub struct ExpectStringValue {
    name: &'static str,
    short_code: &'static str,
    description: &'static str,
}

impl IsFlag for ExpectStringValue {}

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
            name,
            short_code,
            description,
        }
    }
}

impl Defaultable for ExpectStringValue {}

impl<'a> Evaluatable<'a, &'a [&'a str], String> for ExpectStringValue {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, String> {
        input[..]
            .iter()
            .enumerate()
            .find(|(_, &arg)| {
                (arg == format!("{}{}", "--", self.name))
                    || (arg == format!("{}{}", "-", self.short_code))
            })
            // Only need the index.
            .map(|(idx, _)| idx)
            .and_then(|idx| input[..].get(idx + 1).map(|&v| v.to_string()))
            .ok_or_else(|| CliError::FlagEvaluation(self.name.to_string()))
    }
}

impl ShortHelpable for ExpectStringValue {
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

/// StoreTrue represents a terminal flag type, returning a boolean set to true if set.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///    Ok(true),
///    StoreTrue::new("debug", "d", "Run in debug mode.").evaluate(&["hello", "--debug"][..])
/// );
///
/// assert_eq!(
///     Ok(true),
///     StoreTrue::new("debug", "d", "Run in debug mode.").evaluate(&["hello", "-d"][..])
/// );
///
/// assert_eq!(
///     Ok(false),
///     WithDefault::new(
///         false,
///         Optional::new(StoreTrue::new("debug", "d", "Run in debug mode."))
///     )
///     .evaluate(&["hello"][..])
/// );
/// ```
#[derive(Debug)]
pub struct StoreTrue {
    name: &'static str,
    short_code: &'static str,
    description: &'static str,
}

impl IsFlag for StoreTrue {}
impl Defaultable for StoreTrue {}

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
            name,
            short_code,
            description,
        }
    }
}

impl<'a> Evaluatable<'a, &'a [&'a str], bool> for StoreTrue {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, bool> {
        input[..]
            .iter()
            .enumerate()
            .find(|(_, &arg)| {
                (arg == format!("{}{}", "--", self.name))
                    || (arg == format!("{}{}", "-", self.short_code))
            })
            .map(|_| true)
            .ok_or_else(|| CliError::FlagEvaluation(self.name.to_string()))
    }
}

impl ShortHelpable for StoreTrue {
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

/// StoreFalse represents a terminal flag type, returning a boolean set to false if set.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///     Ok(false),
///     StoreFalse::new("no-wait", "n", "don't wait for a response.").evaluate(&["hello", "--no-wait"][..])
/// );
///
/// assert_eq!(
///     Ok(false),
///     StoreFalse::new("no-wait", "n", "don't wait for a response.").evaluate(&["hello", "-n"][..])
/// );
///
/// assert_eq!(
///     Ok(true),
///     WithDefault::new(
///         true,
///         Optional::new(StoreFalse::new("no-wait", "n", "don't wait for a response."))
///     )
///     .evaluate(&["hello"][..])
/// );
/// ```
#[derive(Debug)]
pub struct StoreFalse {
    name: &'static str,
    short_code: &'static str,
    description: &'static str,
}

impl IsFlag for StoreFalse {}
impl Defaultable for StoreFalse {}

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
            name,
            short_code,
            description,
        }
    }
}

impl<'a> Evaluatable<'a, &'a [&'a str], bool> for StoreFalse {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, bool> {
        input[..]
            .iter()
            .enumerate()
            .find(|(_, &arg)| {
                (arg == format!("{}{}", "--", self.name))
                    || (arg == format!("{}{}", "-", self.short_code))
            })
            .map(|_| false)
            .ok_or_else(|| CliError::FlagEvaluation(self.name.to_string()))
    }
}

impl ShortHelpable for StoreFalse {
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

// Integer types

macro_rules! generate_integer_evaluators {
    ($($name:tt, $primitive:ty,)*) => {
        $(
        #[derive(Debug)]
        pub struct $name {
            name: &'static str,
            short_code: &'static str,
            description: &'static str,
        }

        impl IsFlag for $name {}
        impl Defaultable for $name {}

        impl $name {
            #[allow(dead_code)]
            pub fn new(
                name: &'static str,
                short_code: &'static str,
                description: &'static str,
            ) -> Self {
                Self {
                    name,
                    short_code,
                    description,
                }
            }
        }

        impl<'a> Evaluatable<'a, &'a [&'a str], $primitive> for $name {
            fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, $primitive> {
                input[..]
                    .iter()
                    .enumerate()
                    .find(|(_, &arg)| {
                        (arg == format!("{}{}", "--", self.name))
                            || (arg == format!("{}{}", "-", self.short_code))
                    })
                    // Only need the index.
                    .map(|(idx, _)| idx)
                    .and_then(|idx| input[..].get(idx + 1).and_then(|&v| v.parse::<$primitive>().ok()))
                    .ok_or_else(|| CliError::FlagEvaluation(self.name.to_string()))
            }
        }

        impl ShortHelpable for $name {
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
    )*
    };
}

#[rustfmt::skip]
generate_integer_evaluators!(
    ExpectI8Value, i8,
    ExpectI16Value, i16,
    ExpectI32Value, i32,
    ExpectI64Value, i64,
    ExpectU8Value, u8,
    ExpectU16Value, u16,
    ExpectU32Value, u32,
    ExpectU64Value, u64,
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
///         ExpectFilePath::new("file", "f", "A file to open")
///     ).evaluate(&["hello", "--file", "/etc/hostname"][..]).is_ok()
/// );
///
/// assert!(
///     WithOpen::new(
///         ExpectFilePath::new("file", "f", "A file to open")
///     ).evaluate(&["hello", "-f", "/etc/hostname"][..]).is_ok()
/// );
///
/// assert!(
///     WithOpen::new(
///         ExpectFilePath::new("file", "f", "A file to open")
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
    ///     ExpectFilePath::new("file", "f", "A file to open")
    /// );
    /// ```
    pub fn new(evaluator: E) -> Self {
        Self { evaluator }
    }
}

impl<'a, E, A> Evaluatable<'a, A, std::fs::File> for WithOpen<E>
where
    A: 'a,
    E: Evaluatable<'a, A, String> + Openable,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, std::fs::File> {
        self.evaluator.evaluate(input).and_then(|fp| {
            std::fs::File::open(&fp).map_err(|e| {
                CliError::FlagEvaluation(format!("unable to open file evaluator: {}", e))
            })
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
                FlagHelpCollector::Single(fhc.with_modifier(format!("will_open")))
            }
            // this case should never be hit as joined is not defaultable
            fhcj @ FlagHelpCollector::Joined(_, _) => fhcj,
        }
    }
}

/// ExpectFilePath represents a terminal flag type, that parses and validates a
/// file exists in a path. Returning the file path as a Rtring.
///
/// # Example
///
/// ```
/// use scrap::prelude::v1::*;
/// use scrap::*;
///
/// assert_eq!(
///     Ok("/etc/hostname".to_string()),
///     ExpectFilePath::new("file", "f", "A filepath to read").evaluate(&["hello", "--file", "/etc/hostname"][..])
/// );
///
/// assert_eq!(
///     Ok("/etc/hostname".to_string()),
///     WithDefault::new(
///         "/etc/hostname".to_string(),
///         Optional::new(ExpectFilePath::new("file", "f", "A filepath to read"))
///     )
///     .evaluate(&["hello"][..])
/// );
/// ```
#[derive(Debug)]
pub struct ExpectFilePath {
    name: &'static str,
    short_code: &'static str,
    description: &'static str,
}

impl IsFlag for ExpectFilePath {}

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
    /// ExpectFilePath::new("file", "f", "A file name.");
    /// ```
    #[allow(dead_code)]
    pub fn new(name: &'static str, short_code: &'static str, description: &'static str) -> Self {
        Self {
            name,
            short_code,
            description,
        }
    }
}

impl Openable for ExpectFilePath {}

impl Defaultable for ExpectFilePath {}

impl<'a> Evaluatable<'a, &'a [&'a str], String> for ExpectFilePath {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, String> {
        use std::path::Path;

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
                input[..]
                    .get(idx + 1)
                    .and_then(|&p| Path::new(p).is_file().then(|| p))
                    .map(|v| v.to_owned())
            })
            .ok_or_else(|| CliError::FlagEvaluation(self.name.to_string()))
    }
}

impl ShortHelpable for ExpectFilePath {
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

// Unit type

// This implementation exists mostly for cases where a Cmd, or SubCommands
// object has no flags associated with it.
impl<'a> Evaluatable<'a, &'a [&'a str], ()> for () {
    fn evaluate(&self, _: &'a [&'a str]) -> EvaluateResult<'a, ()> {
        Ok(())
    }
}
