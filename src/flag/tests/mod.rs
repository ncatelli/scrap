use crate::flag::Action;
use crate::flag::{Flag, Value};

#[test]
fn should_set_flag_defaults_on_new() {
    assert_eq!(
        crate::flag::Flag::default(),
        Flag::new().name("").short_code("").help_string("")
    );
}

#[test]
fn should_set_false_default_value_on_flag_with_storetrue_action() {
    assert_eq!(
        Some(Value::Bool(false)),
        Flag::new()
            .name("test")
            .action(Action::StoreTrue)
            .default_value
    );
}

#[test]
fn should_set_true_default_value_on_flag_with_storefalse_action() {
    assert_eq!(
        Some(Value::Bool(true)),
        Flag::new()
            .name("test")
            .action(Action::StoreFalse)
            .default_value
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
