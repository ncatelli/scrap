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
