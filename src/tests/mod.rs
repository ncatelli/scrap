use super::*;

#[test]
fn cmd_should_dispatch_a_valid_handler() {
    let cmd = Cmd::new("test")
        .description("a test cmd")
        .with_flags(
            Flag::expect_string("name", "n", "A name.")
                .optional()
                .with_default("foo".to_string())
                .join(
                    Flag::store_true("debug", "d", "run command in debug mode.")
                        .optional()
                        .with_default(false),
                ),
        )
        .with_handler(|(l, debug)| {
            format!("(Left: {}, Right: {})", &l, debug);
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
                .with_flags(WithDefault::<String, _>::new(
                    "foo",
                    Optional::new(ExpectStringValue::new("name", "n", "A name.")),
                ),)
                .help()
                .to_string()
        )
}

#[test]
fn should_generate_expected_helpstring_for_given_string_check() {
    assert_eq!(
        "    --name, -n       A name.                                 ".to_string(),
        format!(
            "{}",
            ExpectStringValue::new("name", "n", "A name.").short_help()
        )
    )
}

#[test]
fn should_generate_expected_helpstring_for_optional_flag() {
    assert_eq!(
        "    --log-level, -l  A given log level setting.               [(optional)]".to_string(),
        Optional::new(ExpectStringValue::new(
            "log-level",
            "l",
            "A given log level setting."
        ))
        .short_help()
        .to_string()
    )
}

#[test]
fn should_default_an_optional_match_when_assigned() {
    let input = ["hello", "--log-level", "info"];

    assert_eq!(
        Ok("foo".to_string()),
        WithDefault::new(
            "foo",
            Optional::new(ExpectStringValue::new("name", "n", "A name."))
        )
        .evaluate(&input[..])
    );

    assert_eq!(
        Ok("foo".to_string()),
        Flag::expect_string("name", "n", "A name.")
            .optional()
            .with_default("foo".to_string())
            .evaluate(&input[..])
    );
}

#[test]
fn should_generate_expected_helpstring_for_optional_with_default_flag() {
    assert_eq!(
            "    --name, -n       A name.                                  [(optional), (default: \"foo\")]".to_string(),
            WithDefault::<String, _>::new(
                "foo",
                Optional::new(ExpectStringValue::new("name", "n", "A name."))
            )
            .short_help()
            .to_string()
        )
}
