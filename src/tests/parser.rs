use crate::flag::{FlagOrValue, Value};
use crate::parsers::ArgumentParser;
use parcel::prelude::v1::*;
use parcel::MatchStatus;

#[test]
fn should_parse_flag_into_flag_or_value() {
    let input = "--no-ask";
    let short_input = "-n";

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            FlagOrValue::Flag("no-ask".to_string())
        ))),
        ArgumentParser::new().parse(input)
    );

    assert_eq!(
        Ok(MatchStatus::Match((
            &short_input[short_input.len()..],
            FlagOrValue::Flag("n".to_string())
        ))),
        ArgumentParser::new().parse(short_input)
    );
}

#[test]
fn should_parse_string_value_into_flag_or_value() {
    let input = "test";

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            FlagOrValue::Value(Value::Str("test".to_string()))
        ))),
        ArgumentParser::new().parse(input)
    );
}

#[test]
fn should_parse_float_value_into_flag_or_value() {
    let input = "123.45";

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            FlagOrValue::Value(Value::Float(123.45))
        ))),
        ArgumentParser::new().parse(input)
    );
}

#[test]
fn should_parse_integer_value_into_flag_or_value() {
    let input = "123";

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            FlagOrValue::Value(Value::Integer(123))
        ))),
        ArgumentParser::new().parse(input)
    );
}

#[test]
fn should_parse_flag_or_value_from_string_vector_input() {
    let input = vec![
        "--size".to_string(),
        "1024".to_string(),
        "--no-ask".to_string(),
        "test".to_string(),
    ];

    assert_eq!(
        Ok(MatchStatus::Match((
            vec![],
            vec![
                FlagOrValue::Flag("size".to_string()),
                FlagOrValue::Value(Value::Integer(1024)),
                FlagOrValue::Flag("no-ask".to_string()),
                FlagOrValue::Value(Value::Str("test".to_string())),
            ]
        ))),
        ArgumentParser::new().parse(input)
    );
}
