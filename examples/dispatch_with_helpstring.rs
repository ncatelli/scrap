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
    )
    .optional();

    // `Cmd` defines the named command, combining metadata without our above defined command.
    let cmd = scrap::Cmd::new("dispatch_with_helpstring")
        .description("A minimal example cli.")
        .author("John Doe <jdoe@example.com>")
        .version("1.2.3")
        .with_flag(help)
        .with_flag(direction)
        // Finally a handler is defined with its signature being a product of
        // the cli's helpstring and defined flags.
        .with_helpstring_handler(|help_string, (help_flag_set, optional_direction)| {
            match (help_flag_set, optional_direction) {
                (false, Some(direction)) => println!("You chose {}.", direction),
                _ => println!("{}", help_string),
            }
        });

    // Evaluate attempts to parse the input, evaluating all commands and flags
    // into concrete types which can be passed to `dispatch`, the defined
    // handler.
    let _ = cmd
        .evaluate(&args[..])
        .map_err(|e| e.to_string())
        .map(|flag_values| cmd.dispatch_with_helpstring(flag_values))
        .map_err(|e| println!("{}", e));
}
