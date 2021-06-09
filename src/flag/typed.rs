pub type EvaluateResult<'a, V> = Result<V, String>;

#[derive(Debug, Default)]
pub struct Cmd<F, H> {
    name: &'static str,
    description: &'static str,
    author: &'static str,
    version: &'static str,
    flags: F,
    handler: H,
}

impl Cmd<(), Box<dyn Fn() -> ()>> {
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
        T: Evaluator<'a, A, B>,
        NH: Fn(B),
    {
        Cmd {
            name: self.name,
            description: self.description,
            author: self.author,
            version: self.version,
            flags: self.flags,
            handler: handler,
        }
    }
}

impl<'a, F, H, B> Evaluator<'a, &'a [&'a str], B> for Cmd<F, H>
where
    F: Evaluator<'a, &'a [&'a str], B>,
{
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<B> {
        match input.get(0) {
            Some(&name) if name == self.name => self.flags.evaluate(&input[1..]),
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
            "{}:\n{}\n{}",
            self.name,
            self.description,
            self.flags.help().to_string()
        )
    }
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
                "--{}, -{}\t{}",
                self.name, self.short_code, self.description,
            )
        } else {
            write!(
                f,
                "--{}, -{}\t{}\t[{}]",
                self.name,
                self.short_code,
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

pub trait Evaluator<'a, A, B> {
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B>;

    fn join<E, C>(self, evaluator2: E) -> BoxedEvaluator<'a, A, (B, C)>
    where
        Self: Sized + Evaluator<'a, A, B> + 'a,
        E: Evaluator<'a, A, C> + 'a,
        A: Copy + 'a,
    {
        BoxedEvaluator::new(Join::<Self, E>::new(self, evaluator2))
    }
}

pub struct BoxedEvaluator<'a, A, B> {
    evaluator: Box<dyn Evaluator<'a, A, B> + 'a>,
}

impl<'a, A, B> BoxedEvaluator<'a, A, B> {
    pub fn new<E>(evaluator: E) -> Self
    where
        E: Evaluator<'a, A, B> + 'a,
    {
        BoxedEvaluator {
            evaluator: Box::new(evaluator),
        }
    }
}

impl<'a, A, B> Evaluator<'a, A, B> for BoxedEvaluator<'a, A, B> {
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B> {
        self.evaluator.evaluate(input)
    }
}

impl<'a, F, A, B> Evaluator<'a, A, B> for F
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

impl<'a, E1, E2, A, B, C> Evaluator<'a, A, (B, C)> for Join<E1, E2>
where
    A: Copy + std::borrow::Borrow<A> + 'a,
    E1: Evaluator<'a, A, B>,
    E2: Evaluator<'a, A, C>,
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

/// A trait that signifies if a type can be assigned a default value.
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

impl<'a, E, A, B> Evaluator<'a, A, B> for WithDefault<B, E>
where
    A: 'a,
    B: Clone,
    E: Evaluator<'a, A, Option<B>>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B> {
        self.evaluator
            .evaluate(input)
            .map(|op| op.unwrap_or(self.default.clone()))
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

impl<'a, E, A, B> Evaluator<'a, A, Option<B>> for Optional<E>
where
    A: 'a,
    E: Evaluator<'a, A, B>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, Option<B>> {
        Ok(self.evaluator.evaluate(input).map_or(None, |b| Some(b)))
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

impl<'a> Evaluator<'a, &'a [&'a str], String> for ExpectStringValue {
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
            .ok_or("No matching Value".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_should_type_validate_handler() {
        assert_eq!(
            Ok(("foo".to_string(), "info".to_string())),
            Cmd::new("test")
                .description("a test cmd")
                .with_flags(
                    Flag::expect_string("name", "n", "A name.")
                        .optional()
                        .with_default("foo".to_string())
                        .join(Flag::expect_string(
                            "log-level",
                            "l",
                            "A given log level setting.",
                        )),
                )
                .with_handler(|(l, r)| {
                    println!("(Left: {}, Right: {})", &l, &r);
                })
                .evaluate(&["test", "-l", "info"][..])
        )
    }

    #[test]
    fn should_evaluate_command_with_valid_sub_flags() {
        assert_eq!(
            Ok("foo".to_string()),
            Cmd::new("test")
                .description("a test cmd")
                .with_flags(
                    Flag::expect_string("name", "n", "A name.")
                        .optional()
                        .with_default("foo".to_string())
                )
                .evaluate(&["test"][..])
        );

        assert_eq!(
            Ok(("foo".to_string(), "info".to_string())),
            Cmd::new("test")
                .description("a test cmd")
                .with_flags(
                    Flag::expect_string("name", "n", "A name.")
                        .optional()
                        .with_default("foo".to_string())
                        .join(Flag::expect_string(
                            "log-level",
                            "l",
                            "A given log level setting."
                        ))
                )
                .evaluate(&["test", "-l", "info"][..])
        );
    }

    #[test]
    fn should_generate_expected_helpstring_for_given_command() {
        assert_eq!(
            "test:\na test cmd\n--name, -n\tA name.\t[(optional), (default: \"foo\")]".to_string(),
            Cmd::new("test")
                .description("a test cmd")
                .with_flags(WithDefault::<String, _>::new(
                    "foo",
                    Optional::new(ExpectStringValue::new("name", "n", "A name.")),
                ),)
                .help()
                .to_string()
        )
    }

    #[test]
    fn should_find_valid_string_flag() {
        let input_long = vec!["hello", "--name", "foo"];
        let input_short = vec!["hello", "-n", "foo"];

        assert_eq!(
            Ok("foo".to_string()),
            ExpectStringValue::new("name", "n", "A name.").evaluate(&input_long[..])
        );

        assert_eq!(
            Ok("foo".to_string()),
            ExpectStringValue::new("name", "n", "A name.").evaluate(&input_short[..])
        );
    }

    #[test]
    fn should_generate_expected_helpstring_for_given_string_check() {
        assert_eq!(
            "--name, -n\tA name.".to_string(),
            format!("{}", ExpectStringValue::new("name", "n", "A name.").help())
        )
    }

    #[test]
    fn should_find_joined_evaluators() {
        let input = vec!["hello", "-n", "foo", "-l", "info"];
        assert_eq!(
            Ok(("foo".to_string(), "info".to_string())),
            Join::new(
                ExpectStringValue::new("name", "n", "A name."),
                ExpectStringValue::new("log-level", "l", "A given log level setting."),
            )
            .evaluate(&input[..])
        );

        assert_eq!(
            Ok(("foo".to_string(), "info".to_string())),
            Flag::expect_string("name", "n", "A name.")
                .join(ExpectStringValue::new(
                    "log-level",
                    "l",
                    "A given log level setting."
                ))
                .evaluate(&input[..])
        );
    }

    #[test]
    fn should_optionally_match_a_value() {
        let input = vec!["hello", "-n", "foo"];

        assert_eq!(
            Ok(Some("foo".to_string())),
            Optional::new(ExpectStringValue::new("name", "n", "A name.")).evaluate(&input[..])
        );

        // validate boxed syntax works
        assert_eq!(
            Ok(Some("foo".to_string())),
            ExpectStringValue::new("name", "n", "A name.")
                .optional()
                .evaluate(&input[..])
        );

        assert_eq!(
            Ok(None),
            Optional::new(ExpectStringValue::new(
                "log-level",
                "l",
                "A given log level setting."
            ))
            .evaluate(&input[..])
        );
    }

    #[test]
    fn should_generate_expected_helpstring_for_optional_flag() {
        assert_eq!(
            "--log-level, -l\tA given log level setting.\t[(optional)]".to_string(),
            Optional::new(ExpectStringValue::new(
                "log-level",
                "l",
                "A given log level setting."
            ))
            .help()
            .to_string()
        )
    }

    #[test]
    fn should_default_an_optional_match_when_assigned() {
        let input = vec!["hello", "--log-level", "info"];

        assert_eq!(
            Ok("foo".to_string()),
            WithDefault::new(
                "foo",
                Optional::new(ExpectStringValue::new("name", "n", "A name."))
            )
            .evaluate(&input[..])
        );

        assert_eq!(
            Ok("foo".to_string()),
            Flag::expect_string("name", "n", "A name.")
                .optional()
                .with_default("foo".to_string())
                .evaluate(&input[..])
        );
    }

    #[test]
    fn should_generate_expected_helpstring_for_optional_with_default_flag() {
        assert_eq!(
            "--name, -n\tA name.\t[(optional), (default: \"foo\")]".to_string(),
            WithDefault::<String, _>::new(
                "foo",
                Optional::new(ExpectStringValue::new("name", "n", "A name."))
            )
            .help()
            .to_string()
        )
    }
}
