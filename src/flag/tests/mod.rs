use crate::flag::{Action, Flag};
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

    assert_eq!(
        Ok(MatchStatus::Match((
            &input[input.len()..],
            "version".to_string()
        ))),
        Flag::new()
            .name("version")
            .short_code("v")
            .help_string("print command version")
            .action(Action::StoreTrue)
            .parse(&input)
    );
}
