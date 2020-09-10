use crate::flag::{Action, Flag, Value, ValueType};
use crate::{App, Config};

mod parser;

macro_rules! to_string_vec {
    ($str_vec:expr) => {
        $str_vec
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
}

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

#[test]
fn should_parse_raw_input_vec_to_config() {
    let input = to_string_vec!(vec!["example", "--version", "-s", "1024"]);
    let mut expected_config = Config::new();
    expected_config.insert("version".to_string(), Value::Bool(true));
    expected_config.insert("size".to_string(), Value::Integer(1024));

    assert_eq!(
        Ok(expected_config),
        App::new()
            .name("example")
            .description("this is a test")
            .author("John Doe <jdoe@example.com>")
            .version("1.2.3")
            .flag(
                Flag::new()
                    .name("version")
                    .short_code("v")
                    .action(Action::StoreTrue)
                    .value_type(ValueType::Bool)
            )
            .flag(
                Flag::new()
                    .name("size")
                    .short_code("s")
                    .action(Action::ExpectSingleValue)
                    .value_type(ValueType::Integer)
            )
            .parse(input)
    );
}

#[test]
fn should_set_default_values_on_unprovided_values() {
    let input = to_string_vec!(vec!["example", "--version"]);
    let mut expected_config = Config::new();
    expected_config.insert("version".to_string(), Value::Bool(true));
    expected_config.insert("size".to_string(), Value::Integer(1024));

    assert_eq!(
        Ok(expected_config),
        App::new()
            .name("example")
            .description("this is a test")
            .author("John Doe <jdoe@example.com>")
            .version("1.2.3")
            .flag(
                Flag::new()
                    .name("version")
                    .short_code("v")
                    .action(Action::StoreTrue)
                    .value_type(ValueType::Bool)
            )
            .flag(
                Flag::new()
                    .name("size")
                    .short_code("s")
                    .action(Action::ExpectSingleValue)
                    .value_type(ValueType::Integer)
                    .default_value(Value::Integer(1024))
            )
            .parse(input)
    );
}
