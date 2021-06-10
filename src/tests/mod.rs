use super::*;

#[test]
fn cmd_should_type_validate_handler() {
    assert_eq!(
        Ok(("foo".to_string(), "info".to_string())),
        Cmd::new("test")
            .description("a test cmd")
            .with_flags(
                Flag::expect_string("name", "n", "A name.")
                    .optional()
                    .with_default("foo".to_string())
                    .join(Flag::expect_string(
                        "log-level",
                        "l",
                        "A given log level setting.",
                    )),
            )
            .with_handler(|(l, r)| {
                format!("(Left: {}, Right: {})", &l, &r);
            })
            .evaluate(&["test", "-l", "info"][..])
    )
}

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
fn should_find_valid_string_flag() {
    assert_eq!(
        Ok("foo".to_string()),
        ExpectStringValue::new("name", "n", "A name.").evaluate(&["hello", "--name", "foo"][..])
    );

    assert_eq!(
        Ok("foo".to_string()),
        ExpectStringValue::new("name", "n", "A name.").evaluate(&["hello", "-n", "foo"][..])
    );
}

#[test]
fn should_find_valid_store_true_flag() {
    assert_eq!(
        Ok(true),
        StoreTrue::new("debug", "d", "Run in debug mode.").evaluate(&["hello", "--debug"][..])
    );

    assert_eq!(
        Ok(true),
        StoreTrue::new("debug", "d", "Run in debug mode.").evaluate(&["hello", "-d"][..])
    );

    // should appropriately default.
    assert_eq!(
        Ok(false),
        WithDefault::new(
            false,
            Optional::new(StoreTrue::new("debug", "d", "Run in debug mode."))
        )
        .evaluate(&["hello"][..])
    );
}

#[test]
fn should_find_valid_store_false_flag() {
    assert_eq!(
        Ok(false),
        StoreFalse::new("no-wait", "n", "don't wait for a response.")
            .evaluate(&["hello", "--no-wait"][..])
    );

    assert_eq!(
        Ok(false),
        StoreFalse::new("no-wait", "n", "don't wait for a response.")
            .evaluate(&["hello", "-n"][..])
    );

    // should appropriately default.
    assert_eq!(
        Ok(true),
        WithDefault::new(
            true,
            Optional::new(StoreFalse::new(
                "no-wait",
                "n",
                "don't wait for a response."
            ))
        )
        .evaluate(&["hello"][..])
    );
}

#[test]
fn should_generate_expected_helpstring_for_given_string_check() {
    assert_eq!(
        "    --name, -n       A name.                                 ".to_string(),
        format!("{}", ExpectStringValue::new("name", "n", "A name.").help())
    )
}

#[test]
fn should_find_joined_evaluators() {
    let input = vec!["hello", "-n", "foo", "-l", "info"];
    assert_eq!(
        Ok(("foo".to_string(), "info".to_string())),
        Join::new(
            ExpectStringValue::new("name", "n", "A name."),
            ExpectStringValue::new("log-level", "l", "A given log level setting."),
        )
        .evaluate(&input[..])
    );

    assert_eq!(
        Ok(("foo".to_string(), "info".to_string())),
        Flag::expect_string("name", "n", "A name.")
            .join(ExpectStringValue::new(
                "log-level",
                "l",
                "A given log level setting."
            ))
            .evaluate(&input[..])
    );
}

#[test]
fn should_optionally_match_a_value() {
    let input = vec!["hello", "-n", "foo"];

    assert_eq!(
        Ok(Some("foo".to_string())),
        Optional::new(ExpectStringValue::new("name", "n", "A name.")).evaluate(&input[..])
    );

    // validate boxed syntax works
    assert_eq!(
        Ok(Some("foo".to_string())),
        ExpectStringValue::new("name", "n", "A name.")
            .optional()
            .evaluate(&input[..])
    );

    assert_eq!(
        Ok(None),
        Optional::new(ExpectStringValue::new(
            "log-level",
            "l",
            "A given log level setting."
        ))
        .evaluate(&input[..])
    );
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
        .help()
        .to_string()
    )
}

#[test]
fn should_default_an_optional_match_when_assigned() {
    let input = vec!["hello", "--log-level", "info"];

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
            .help()
            .to_string()
        )
}
