use super::*;

#[test]
fn cmd_should_dispatch_a_valid_handler() {
    let cmd = Cmd::new("test")
        .description("a test cmd")
        .with_flag(
            Flag::expect_string("name", "n", "A name.")
                .optional()
                .with_default("foo".to_string()),
        )
        .with_flag(
            Flag::store_true("debug", "d", "run command in debug mode.")
                .optional()
                .with_default(false),
        )
        .with_handler(|(n, debug)| {
            format!("(Left: {}, Right: {})", &n, debug);
        });

    assert_eq!(
        Ok(()),
        cmd.evaluate(&["test", "-l", "info"][..])
            .map(|flag_values| cmd.dispatch(flag_values))
    );
}

#[test]
fn should_generate_expected_helpstring_for_given_command() {
    assert_eq!("Usage: test [OPTIONS]\na test cmd\nFlags:\n    --name, -n       A name.                                  [(optional), (default: \"foo\")]"
            .to_string(),
            Cmd::new("test")
                .description("a test cmd")
                .with_flag(WithDefault::<String, _>::new(
                    "foo",
                    Optional::new(FlagWithValue::new("name", "n", "A name.", StringValue)),
                ),)
                .help()
        )
}

#[test]
fn should_generate_expected_helpstring_for_given_string_check() {
    assert_eq!(
        "    --name, -n       A name.                                 ".to_string(),
        format!(
            "{}",
            FlagWithValue::new("name", "n", "A name.", StringValue).short_help()
        )
    )
}

#[test]
fn should_generate_expected_helpstring_for_optional_flag() {
    assert_eq!(
        "    --log-level, -l  A given log level setting.               [(optional)]".to_string(),
        Optional::new(FlagWithValue::new(
            "log-level",
            "l",
            "A given log level setting.",
            StringValue
        ))
        .short_help()
        .to_string()
    )
}

#[test]
fn should_generate_expected_helpstring_for_optional_with_default_flag() {
    assert_eq!(
            "    --name, -n       A name.                                  [(optional), (default: \"foo\")]".to_string(),
            WithDefault::<String, _>::new(
                "foo",
                Optional::new(FlagWithValue::new("name", "n", "A name.", StringValue))
            )
            .short_help()
            .to_string()
        )
}
