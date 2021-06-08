pub type EvaluateResult<'a, V> = Result<V, String>;

#[derive(Debug)]
pub struct Cmd<F> {
    name: &'static str,
    descriptions: &'static str,
    flags: F,
}

impl<F> Cmd<F> {
    pub fn new(name: &'static str, descriptions: &'static str, flags: F) -> Self {
        Self {
            name,
            descriptions,
            flags,
        }
    }
}

impl<'a, F, A, B> Evaluator<'a, A, B> for Cmd<F>
where
    F: Evaluator<'a, A, B>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<B> {
        self.flags.evaluate(input)
    }
}

impl<F> Helpable for Cmd<F>
where
    F: std::fmt::Display,
{
    type Output = String;

    fn help(&self) -> Self::Output {
        let fhelp = self.flags.to_string();
        CmdHelpContext::new(self.name, self.descriptions, fhelp).to_string()
    }
}

#[derive(Default)]
pub struct CmdHelpContext<F> {
    name: &'static str,
    description: &'static str,
    /// Additional String values to be appended after the description.
    flags: F,
}

impl<F> CmdHelpContext<F> {
    pub fn new(name: &'static str, description: &'static str, flags: F) -> Self {
        Self {
            name,
            description,
            flags,
        }
    }
}

impl<F> std::fmt::Display for CmdHelpContext<F>
where
    F: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:\n{}\n{}", self.name, self.description, self.flags)
    }
}

pub trait Helpable
where
    Self::Output: std::fmt::Display,
{
    type Output;

    fn help(&self) -> Self::Output;
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CmdHelpCollector {
    tab_depth: usize,
    inner: Vec<String>,
}

impl CmdHelpCollector {
    pub fn new(tab_depth: usize, inner: Vec<String>) -> Self {
        Self { tab_depth, inner }
    }
}

impl std::fmt::Display for CmdHelpCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tab_str: String = "\t"
            .chars()
            .into_iter()
            .cycle()
            .take(self.tab_depth)
            .collect();

        write!(
            f,
            "{}",
            self.inner
                .iter()
                .map(|hs| format!("{}{}", &tab_str, hs))
                .collect::<Vec<String>>()
                .join("\n")
        )
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

    fn optional(self) -> BoxedEvaluator<'a, A, Option<B>>
    where
        Self: Sized + 'a,
        A: 'a,
    {
        BoxedEvaluator::new(Optional::new(self))
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
    E1: Helpable,
    E2: Helpable,
{
    type Output = CmdHelpCollector;

    fn help(&self) -> Self::Output {
        CmdHelpCollector::new(
            1,
            vec![
                format!("{}", self.evaluator1.help()),
                format!("{}", self.evaluator2.help()),
            ],
        )
    }
}

#[derive(Debug)]
pub struct Optional<E> {
    evaluator: E,
}

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
    E: Helpable<Output = FlagHelpContext>,
{
    type Output = FlagHelpContext;

    fn help(&self) -> Self::Output {
        self.evaluator.help().with_modifier("optional".to_string())
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
    E: Helpable + Helpable<Output = FlagHelpContext>,
{
    type Output = FlagHelpContext;

    fn help(&self) -> Self::Output {
        self.evaluator
            .help()
            .with_modifier(format!("Default: {:?}", self.default.clone()))
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
    type Output = FlagHelpContext;

    fn help(&self) -> Self::Output {
        FlagHelpContext::new(self.name, self.short_code, self.description, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_evaluate_command_with_valid_sub_flags() {
        assert_eq!(
            Ok("foo".to_string()),
            Cmd::new(
                "test",
                "a test cmd",
                WithDefault::<String, _>::new(
                    "foo",
                    Optional::new(ExpectStringValue::new("name", "n", "A name.")),
                ),
            )
            .evaluate(&vec!["test"][..])
        )
    }

    #[test]
    fn should_generate_expected_helpstring_for_given_command() {
        assert_eq!(
            "test:\na test cmd\n--name, -n\tA name.\t[(optional), (Default: \"foo\")]".to_string(),
            Cmd::new(
                "test",
                "a test cmd",
                WithDefault::<String, _>::new(
                    "foo",
                    Optional::new(ExpectStringValue::new("name", "n", "A name.")),
                ),
            )
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
    }

    #[test]
    fn should_generate_expected_helpstring_for_join_evaluator() {
        assert_eq!(
            CmdHelpCollector::new(
                1,
                vec![
                    "--name, -n\tA name.".to_string(),
                    "--log-level, -l\tA given log level setting.".to_string()
                ]
            ),
            Join::new(
                ExpectStringValue::new("name", "n", "A name."),
                ExpectStringValue::new("log-level", "l", "A given log level setting.")
            )
            .help()
        );

        assert_eq!(
            "\t--name, -n\tA name.\n\t--log-level, -l\tA given log level setting.".to_string(),
            Join::new(
                ExpectStringValue::new("name", "n", "A name."),
                ExpectStringValue::new("log-level", "l", "A given log level setting.")
            )
            .help()
            .to_string()
        )
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
    }

    #[test]
    fn should_generate_expected_helpstring_for_optional_with_default_flag() {
        assert_eq!(
            "--name, -n\tA name.\t[(optional), (Default: \"foo\")]".to_string(),
            WithDefault::<String, _>::new(
                "foo",
                Optional::new(ExpectStringValue::new("name", "n", "A name."))
            )
            .help()
            .to_string()
        )
    }
}
