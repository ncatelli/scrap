use crate::App;

#[test]
fn should_match_expected_help_message() {
    assert_eq!(
        "this is a test\nUsage: example [OPTIONS] [SUBCOMMAND]",
        format!(
            "{}",
            App::new()
                .name("example")
                .description("this is a test")
                .author("John Doe <jdoe@example.com>")
                .version("1.2.3")
        )
    );
}
