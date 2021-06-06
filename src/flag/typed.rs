pub type EvaluateResult<'a, V> = Result<V, String>;

pub trait Evaluator<'a, A, B> {
    fn evaluate(&self, input: A) -> EvaluateResult<'a, B>;
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

#[derive(Debug)]
pub struct ExpectStringValue {
    name: &'static str,
    short_code: &'static str,
}

impl ExpectStringValue {
    #[allow(dead_code)]
    pub fn new(name: &'static str, short_code: &'static str) -> Self {
        Self { name, short_code }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_evaluator_should_find_valid_string_flag() {
        let input_long = vec!["hello", "--name", "foo"];
        let input_short = vec!["hello", "-n", "foo"];

        assert_eq!(
            Ok("foo".to_string()),
            ExpectStringValue::new("name", "n").evaluate(&input_long[..])
        );

        assert_eq!(
            Ok("foo".to_string()),
            ExpectStringValue::new("name", "n").evaluate(&input_short[..])
        );
    }

    #[test]
    fn assert_evaluator_should_find_joined_evaluators() {
        let input = vec!["hello", "-n", "foo", "-l", "info"];
        assert_eq!(
            Ok(("foo".to_string(), "info".to_string())),
            Join::new(
                ExpectStringValue::new("name", "n"),
                ExpectStringValue::new("log-level", "l"),
            )
            .evaluate(&input[..])
        );
    }

    #[test]
    fn assert_evaluator_should_optionally_match_a_value() {
        let input = vec!["hello", "-n", "foo"];

        assert_eq!(
            Ok(Some("foo".to_string())),
            Optional::new(ExpectStringValue::new("name", "n")).evaluate(&input[..])
        );

        assert_eq!(
            Ok(None),
            Optional::new(ExpectStringValue::new("log-level", "l")).evaluate(&input[..])
        );
    }

    #[test]
    fn assert_evaluator_should_default_an_optional_match_when_assigned() {
        let input = vec!["hello", "--log-level", "info"];

        assert_eq!(
            Ok("foo".to_string()),
            WithDefault::new("foo", Optional::new(ExpectStringValue::new("name", "n")))
                .evaluate(&input[..])
        );
    }
}
