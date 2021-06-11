pub mod prelude;

#[cfg(test)]
mod tests;

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
///     .with_flags(
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
///     .with_flags(
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
///     CmdGroup::new("testgroup").with_commands(commands)
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
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            description: "",
            author: "",
            version: "",
            commands: (),
        }
    }

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
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub fn author(mut self, author: &'static str) -> Self {
        self.author = author;
        self
    }

    pub fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }

    pub fn with_commands<NC>(self, commands: NC) -> CmdGroup<NC> {
        CmdGroup {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            commands,
        }
    }
}

impl<C> CmdGroup<C>
where
    C: IsCmd,
{
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
            _ => Err(format!("no match for command: {}", &self.name)),
        }
    }
}

impl<'a, C, A, B> Dispatchable<A, B, ()> for CmdGroup<C>
where
    C: Evaluatable<'a, A, B> + Dispatchable<A, B, ()>,
{
    fn dispatch(self, flag_values: B) {
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
///     .with_flags(
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
///     .with_flags(
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
    command1: C1,
    command2: C2,
}

impl<C1, C2> OneOf<C1, C2> {
    pub fn new(command1: C1, command2: C2) -> Self {
        Self { command1, command2 }
    }
}

impl<'a, C1, C2, B, C> Evaluatable<'a, &'a [&'a str], Either<B, C>> for OneOf<C1, C2>
where
    C1: Evaluatable<'a, &'a [&'a str], B>,
    C2: Evaluatable<'a, &'a [&'a str], C>,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<Either<B, C>> {
        match (
            self.command1.evaluate(&input),
            self.command2.evaluate(&input),
        ) {
            (Ok(b), Err(_)) => Ok(Either::Left(b)),
            (Err(_), Ok(c)) => Ok(Either::Right(c)),
            _ => Err("ambiguous command.".to_string()),
        }
    }
}

impl<'a, C1, C2, A, B, C> Dispatchable<A, Either<B, C>, ()> for OneOf<C1, C2>
where
    C1: Evaluatable<'a, A, B> + Dispatchable<A, B, ()>,
    C2: Evaluatable<'a, A, C> + Dispatchable<A, C, ()>,
{
    fn dispatch(self, flag_values: Either<B, C>) {
        match flag_values {
            Either::Left(b) => self.command1.dispatch(b),
            Either::Right(c) => self.command2.dispatch(c),
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
        format!(
            "{}\n{}",
            self.command1.short_help(),
            self.command2.short_help()
        )
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
///         .with_flags(
///             Flag::expect_string("name", "n", "A name.")
///                 .optional()
///                 .with_default("foo".to_string())
///                 .join(Flag::expect_string(
///                     "log-level",
///                     "l",
///                     "A given log level setting.",
///                 )),
///         )
///         .with_handler(|(l, r)| {
///             format!("(Left: {}, Right: {})", &l, &r);
///         })
///         .evaluate(&["test", "-l", "info"][..])
/// )
/// ```
#[derive(Debug, Default)]
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
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    pub fn author(mut self, author: &'static str) -> Self {
        self.author = author;
        self
    }

    pub fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }

    pub fn with_handler<'a, A, B, NH>(self, handler: NH) -> Cmd<T, NH>
    where
        T: Evaluatable<'a, A, B>,
        NH: Fn(B),
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

// Cmd has no flags
impl<'a, H> Evaluatable<'a, &'a [&'a str], ()> for Cmd<(), H> {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<()> {
        match input
            .get(0)
            .map(|&bin| std::path::Path::new(bin).file_name())
        {
            Some(Some(name)) if name == self.name => Ok(()),
            _ => Err(format!("no match for command: {}", &self.name)),
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
            _ => Err(format!("no match for command: {}", &self.name)),
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

impl<'a, T, H, A, B> Dispatchable<A, B, ()> for Cmd<T, H>
where
    T: Evaluatable<'a, A, B>,
    H: Fn(B),
{
    fn dispatch(self, flag_values: B) {
        (self.handler)(flag_values)
    }
}

pub trait Dispatchable<A, B, R> {
    fn dispatch(self, flag_values: B) -> R;
}

/// Provides short summary help descriptions
pub trait ShortHelpable
where
    Self::Output: std::fmt::Display,
{
    type Output;

    fn short_help(&self) -> Self::Output;
}

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
    pub fn expect_string(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> ExpectStringValue {
        ExpectStringValue::new(name, short_code, description)
    }

    pub fn store_true(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> StoreTrue {
        StoreTrue::new(name, short_code, description)
    }

    pub fn store_false(
        name: &'static str,
        short_code: &'static str,
        description: &'static str,
    ) -> StoreFalse {
        StoreFalse::new(name, short_code, description)
    }
}

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

#[derive(Default)]
pub struct FlagHelpContext {
    name: &'static str,
    short_code: &'static str,
    description: &'static str,
    /// Additional String values to be appended after the description.
    modifiers: Vec<String>,
}

impl FlagHelpContext {
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
pub type EvaluateResult<'a, T> = Result<T, String>;

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
            .map_err(|e| format!("failed to map left side: {}", e))
            .and_then(|e1_res| match self.evaluator2.evaluate(input) {
                Ok(e2_res) => Ok((e1_res, e2_res)),
                Err(e) => Err(format!("failed to map right side: {}", e)),
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
    fn with_default<D>(self, default: D) -> WithDefault<D, Self> {
        WithDefault::new(default, self)
    }

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
/// use scrap::ExpectStringValue;
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
            .ok_or_else(|| "No matching value".to_string())
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
            .ok_or_else(|| "No matching value".to_string())
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
///     StoreFalse::new("no-wait", "n", "don't wait for a response.").evaluate(&["hello", "--no-wait"][..])
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
            .ok_or_else(|| "No matching value".to_string())
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
