use crate::flag::{Action, Flag, Value};
use parcel::prelude::v1::*;
use parcel::MatchStatus;

#[test]
fn should_set_flag_defaults_on_new() {
    assert_eq!(
        Flag::default(),
        Flag::new().name("").short_code("").help_string("")
    );
}

#[test]
fn should_generate_correct_help_message_based_off_passed_arguments() {
    assert_eq!(
        "--version, -v: print command version",
        format!(
            "{}",
            Flag::new()
                .name("version")
                .short_code("v")
                .help_string("print command version")
                .action(Action::StoreTrue)
        )
    );

    assert_eq!(
        "--version: print command version",
        format!(
            "{}",
            Flag::new()
                .name("version")
                .help_string("print command version")
                .action(Action::StoreTrue)
        )
    );

    assert_eq!(
        "--version, -v",
        format!(
            "{}",
            Flag::new()
                .name("version")
                .short_code("v")
                .action(Action::StoreTrue)
        )
    );
}

#[test]
fn should_match_parse_flags_that_match_store_true_actions() {
    let input = "--version";
    let short_input = "-v";
    let flag = Flag::new()
        .name("version")
        .short_code("v")
        .help_string("print command version")
        .action(Action::StoreTrue);

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            ("version".to_string(), Value::Bool(true))
        ))),
        flag.clone().parse(&input)
    );

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            ("v".to_string(), Value::Bool(true))
        ))),
        flag.clone().parse(&short_input)
    );
}

#[test]
fn should_match_parse_flags_that_match_store_false_actions() {
    let input = "--no-ask";
    let short_input = "-n";
    let flag = Flag::new()
        .name("no-ask")
        .short_code("n")
        .help_string("don't prompt user for input")
        .action(Action::StoreFalse);

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            ("no-ask".to_string(), Value::Bool(false))
        ))),
        flag.clone().parse(&input)
    );

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            ("n".to_string(), Value::Bool(false))
        ))),
        flag.clone().parse(&short_input)
    );
}
