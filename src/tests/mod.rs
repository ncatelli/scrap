use crate::App;

mod parser;

#[test]
fn should_set_app_defaults_on_new() {
    assert_eq!(
        App::default(),
        App::new().name("").description("").author("").version("")
    );
}

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
