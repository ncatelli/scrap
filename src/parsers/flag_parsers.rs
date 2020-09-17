use crate::flag::{Flag, FlagOrValue, Value, ValueType};
use parcel::{MatchStatus, Parser};

pub fn match_flag<'a>(expected: Flag) -> impl Parser<'a, &'a [FlagOrValue], Flag> {
    move |input: &'a [FlagOrValue]| match input.get(0) {
        Some(FlagOrValue::Flag(name)) if name == &expected.name || name == &expected.short_code => {
            Ok(MatchStatus::Match((&input[1..], expected.clone())))
        }
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn match_any_flag<'a>() -> impl Parser<'a, &'a [FlagOrValue], Flag> {
    move |input: &'a [FlagOrValue]| match input.get(0) {
        Some(FlagOrValue::Flag(name)) => {
            Ok(MatchStatus::Match((&input[1..], Flag::new().name(name))))
        }
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn match_value_type<'a>(expected: ValueType) -> impl Parser<'a, &'a [FlagOrValue], Value> {
    move |input: &'a [FlagOrValue]| match (expected, input.get(0)) {
        (ValueType::Any, Some(FlagOrValue::Value(v))) => {
            Ok(MatchStatus::Match((&input[1..], v.clone())))
        }
        (ValueType::Bool, Some(FlagOrValue::Value(Value::Bool(bv)))) => {
            Ok(MatchStatus::Match((&input[1..], Value::Bool(*bv))))
        }
        (ValueType::Str, Some(FlagOrValue::Value(Value::Str(sv)))) => {
            Ok(MatchStatus::Match((&input[1..], Value::Str(sv.clone()))))
        }
        (ValueType::Integer, Some(FlagOrValue::Value(Value::Integer(iv)))) => {
            Ok(MatchStatus::Match((&input[1..], Value::Integer(*iv))))
        }
        (ValueType::Float, Some(FlagOrValue::Value(Value::Float(fv)))) => {
            Ok(MatchStatus::Match((&input[1..], Value::Float(*fv))))
        }

        _ => Ok(MatchStatus::NoMatch(input)),
    }
}
