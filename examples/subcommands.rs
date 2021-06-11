use scrap::prelude::v1::*;
use scrap::*;
use std::env;

fn main() {
    let raw_args: Vec<String> = env::args().into_iter().collect::<Vec<String>>();
    let args = raw_args.iter().map(|a| a.as_str()).collect::<Vec<&str>>();

    let cmd_group = CmdGroup::new("subcommands").with_commands(OneOf::new(
        Cmd::new("test_one")
            .description("first test cmd")
            .with_flags(
                Flag::expect_string("name", "n", "A name.")
                    .optional()
                    .with_default("foo".to_string()),
            )
            .with_handler(|name| {
                println!("name: {}", &name);
            }),
        Cmd::new("test_two")
            .description("a test cmd")
            .with_flags(
                Flag::store_true("debug", "d", "Run command in debug mode.")
                    .optional()
                    .with_default(false),
            )
            .with_handler(|debug| {
                println!("debug: {}", &debug);
            }),
    ));

    println!(
        "Spasm Subcommands Example\n\nExample Help Output:\n{}\n",
        ["-"].iter().cycle().take(40).copied().collect::<String>()
    );
    println!("{}\n", cmd_group.help());
    println!(
        "Running Dispatcher:\n{}\n",
        ["-"].iter().cycle().take(40).copied().collect::<String>()
    );

    cmd_group
        .evaluate(&args[..])
        .map(|flag_values| cmd_group.dispatch(flag_values))
        .expect("Flags should evaluate correctly");
}
