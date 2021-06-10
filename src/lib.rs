pub mod prelude;

#[cfg(test)]
mod tests;

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
}

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

#[derive(Debug, Default)]
pub struct Cmd<F, H> {
    name: &'static str,
    description: &'static str,
    author: &'static str,
    version: &'static str,
    flags: F,
    handler: H,
}

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

    pub fn with_flags<F>(self, flags: F) -> Cmd<F, H> {
        Cmd {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            flags,
            handler: self.handler,
        }
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

impl<F, H> Helpable for Cmd<F, H>
where
    F: Helpable<Output = FlagHelpCollector>,
{
    type Output = String;

    fn help(&self) -> Self::Output {
        format!(
            "Usage: {} [OPTIONS]\n{}\nFlags:\n{}",
            self.name,
            self.description,
            self.flags.help().to_string()
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

pub trait Helpable
where
    Self::Output: std::fmt::Display,
{
    type Output;

    fn help(&self) -> Self::Output;
}

/// A constructor type to help with building flags. This should never be used
/// for anything but static method calls.
pub struct Flag;

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
    Evaluatable<'a, A, B> + Helpable<Output = FlagHelpCollector>
{
}

impl<'a, A, B, T> BoxedEvaluatable<'a, A, B> for T where
    T: Evaluatable<'a, A, B> + Helpable<Output = FlagHelpCollector> + 'a
{
}

/// BoxedEvaluator provides a wrapper for Evaluatable types.
pub struct BoxedEvaluator<'a, A, B> {
    evaluator: Box<dyn BoxedEvaluatable<'a, A, B> + 'a>,
}

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

impl<'a, A, B> Helpable for BoxedEvaluator<'a, A, B> {
    type Output = FlagHelpCollector;

    fn help(&self) -> Self::Output {
        self.evaluator.help()
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

#[derive(Debug)]
pub struct Join<E1, E2> {
    evaluator1: E1,
    evaluator2: E2,
}

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

impl<E1, E2> Helpable for Join<E1, E2>
where
    E1: Helpable<Output = FlagHelpCollector>,
    E2: Helpable<Output = FlagHelpCollector>,
{
    type Output = FlagHelpCollector;

    fn help(&self) -> Self::Output {
        FlagHelpCollector::Joined(
            Box::new(self.evaluator1.help()),
            Box::new(self.evaluator2.help()),
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
#[derive(Debug)]
pub struct WithDefault<B, E> {
    default: B,
    evaluator: E,
}

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

impl<B, E> Helpable for WithDefault<B, E>
where
    B: Clone + std::fmt::Debug,
    E: Helpable + Helpable<Output = FlagHelpCollector> + Defaultable,
{
    type Output = FlagHelpCollector;

    fn help(&self) -> Self::Output {
        match self.evaluator.help() {
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
// `Option<B>`.
#[derive(Debug)]
pub struct Optional<E> {
    evaluator: E,
}

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

impl<E> Helpable for Optional<E>
where
    E: Helpable<Output = FlagHelpCollector>,
{
    type Output = FlagHelpCollector;

    fn help(&self) -> Self::Output {
        match self.evaluator.help() {
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

impl Helpable for ExpectStringValue {
    type Output = FlagHelpCollector;

    fn help(&self) -> Self::Output {
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
/// use scrap::{StoreTrue, WithDefault, Optional};
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

impl Helpable for StoreTrue {
    type Output = FlagHelpCollector;

    fn help(&self) -> Self::Output {
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
/// use scrap::{StoreFalse, WithDefault, Optional};
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

impl Helpable for StoreFalse {
    type Output = FlagHelpCollector;

    fn help(&self) -> Self::Output {
        FlagHelpCollector::Single(FlagHelpContext::new(
            self.name,
            self.short_code,
            self.description,
            Vec::new(),
        ))
    }
}
