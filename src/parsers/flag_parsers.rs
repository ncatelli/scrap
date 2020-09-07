use crate::flag::Flag;
use crate::flag::FlagOrValue;
use crate::flag::Value;
use parcel::{MatchStatus, Parser};

pub fn match_flag<'a>(expected: &'a Flag) -> impl Parser<'a, &'a [FlagOrValue], Flag> {
    move |input: &'a [FlagOrValue]| match input.get(0) {
        Some(FlagOrValue::Flag(name)) if name == &expected.name => {
            Ok(MatchStatus::Match((&input[1..], expected.clone())))
        }
        _ => Ok(MatchStatus::NoMatch(input)),
    }
}

pub fn match_string_value<'a>(expected: &'a Value) -> impl Parser<'a, &'a [FlagOrValue], Value> {
    move |input: &'a [FlagOrValue]| {
        let ev = match expected {
            Value::Str(v) => Ok(v),
            _ => Err("invalid value"),
        };

        match (ev, input.get(0)) {
            (Ok(ev), Some(FlagOrValue::Value(Value::Str(sv)))) if sv == ev => {
                Ok(MatchStatus::Match((&input[1..], expected.clone())))
            }
            _ => Ok(MatchStatus::NoMatch(input)),
        }
    }
}

pub fn match_integer_value<'a>(expected: &'a Value) -> impl Parser<'a, &'a [FlagOrValue], Value> {
    move |input: &'a [FlagOrValue]| {
        let ev = match expected {
            Value::Integer(v) => Ok(v),
            _ => Err("invalid value"),
        };
        match (ev, input.get(0)) {
            (Ok(ev), Some(FlagOrValue::Value(Value::Integer(iv)))) if iv == ev => {
                Ok(MatchStatus::Match((&input[1..], expected.clone())))
            }
            _ => Ok(MatchStatus::NoMatch(input)),
        }
    }
}

pub fn match_float_value<'a>(expected: &'a Value) -> impl Parser<'a, &'a [FlagOrValue], Value> {
    move |input: &'a [FlagOrValue]| {
        let ev = match expected {
            Value::Float(v) => Ok(v),
            _ => Err("invalid value"),
        };
        match (ev, input.get(0)) {
            (Ok(ev), Some(FlagOrValue::Value(Value::Float(fv)))) if fv == ev => {
                Ok(MatchStatus::Match((&input[1..], expected.clone())))
            }
            _ => Ok(MatchStatus::NoMatch(input)),
        }
    }
}

pub fn match_bool_value<'a>(expected: &'a Value) -> impl Parser<'a, &'a [FlagOrValue], Value> {
    move |input: &'a [FlagOrValue]| {
        let ev = match expected {
            Value::Bool(v) => Ok(v),
            _ => Err("invalid value"),
        };
        match (ev, input.get(0)) {
            (Ok(ev), Some(FlagOrValue::Value(Value::Bool(bv)))) if bv == ev => {
                Ok(MatchStatus::Match((&input[1..], expected.clone())))
            }
            _ => Ok(MatchStatus::NoMatch(input)),
        }
    }
}
