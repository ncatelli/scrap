use scrap::prelude::v1::*;
use scrap::*;
use std::env;

fn main() {
    let raw_args: Vec<String> = env::args().into_iter().collect::<Vec<String>>();
    let args = raw_args.iter().map(|a| a.as_str()).collect::<Vec<&str>>();

    // flag definitions
    let name = Flag::expect_string("name", "n", "A name.")
        .optional()
        .with_default("foo".to_string());
    let debug = Flag::store_true("debug", "d", "Run command in debug mode.")
        .optional()
        .with_default(false);

    let cmd_group = CmdGroup::new("subcommands")
        .with_command(
            Cmd::new("test_one")
                .description("first test cmd")
                .with_flag(name)
                .with_handler(|name| {
                    println!("name: {}", &name);
                }),
        )
        .with_command(
            Cmd::new("test_two")
                .description("a test cmd")
                .with_flag(debug)
                .with_handler(|debug| {
                    println!("debug: {}", &debug);
                }),
        );

    let help_string = cmd_group.help();
    let eval_res = cmd_group
        .evaluate(&args[..])
        .and_then(|flag_values| match flag_values {
            MatchStatus::Match(_, v) => Ok(cmd_group.dispatch(v)),
            MatchStatus::NoMatch(_) => Err(CliError::AmbiguousCommand),
        });

    match eval_res {
        Ok(_) => (),
        Err(_) => println!("{}", &help_string),
    }
}
