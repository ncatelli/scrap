use scrap::prelude::v1::*;
use std::env;

fn main() {
    let raw_args: Vec<String> = env::args().into_iter().collect::<Vec<String>>();
    let args = raw_args.iter().map(|a| a.as_str()).collect::<Vec<&str>>();

    // The `Flag` type defines helpers for generating various common flag
    // evaluators.
    // Shown below, the `help` flag represents common boolean flag with default
    // a default value.
    let help = scrap::Flag::store_true("help", "h", "output help information.")
        .optional()
        .with_default(false);
    // `direction` provides a flag with a finite set of choices, matching a
    // string value.
    let direction = scrap::Flag::with_choices(
        "direction",
        "d",
        "a cardinal direction.",
        [
            "north".to_string(),
            "south".to_string(),
            "east".to_string(),
            "west".to_string(),
        ],
        scrap::StringValue,
    );

    // `Cmd` defines the named command, combining metadata without our above defined command.
    let cmd = scrap::Cmd::new("basic")
        .description("A minimal example cli.")
        .author("John Doe <jdoe@example.com>")
        .version("1.2.3")
        .with_flag(help)
        .with_flag(direction)
        // Finally a handler is defined with its signature being a product of
        // the cli's defined flags.
        .with_handler(|(_, direction)| println!("You chose {}.", direction));

    // The help method generates a help command based on the output rendered
    // from all defined flags.
    let help_string = cmd.help();

    // Evaluate attempts to parse the input, evaluating all commands and flags
    // into concrete types which can be passed to `dispatch`, the defined
    // handler.
    let res =
        cmd.evaluate(&args[..])
            .map_err(|e| e.to_string())
            .and_then(|Value { span, value }| match value {
                (help, direction) if !help => {
                    cmd.dispatch(Value::new(span, (help, direction)));
                    Ok(())
                }
                _ => Err("output help".to_string()),
            });

    match res {
        Ok(_) => (),
        Err(_) => println!("{}", help_string),
    }
}
