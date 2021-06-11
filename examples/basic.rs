use scrap::prelude::v1::*;
use std::env;

fn main() {
    let raw_args: Vec<String> = env::args().into_iter().collect::<Vec<String>>();
    let args = raw_args.iter().map(|a| a.as_str()).collect::<Vec<&str>>();

    let cmd = scrap::Cmd::new("basic")
        .description("this is a test")
        .author("John Doe <jdoe@example.com>")
        .version("1.2.3")
        .with_flag(
            scrap::Flag::store_true("version", "v", "output the version of the command.")
                .optional()
                .with_default(false),
        )
        .with_flag(
            scrap::Flag::store_true("test", "t", "a test flag.")
                .optional()
                .with_default(false),
        )
        .with_handler(|(version, test)| println!("Version: {}\nTest: {}", version, test));

    println!(
        "Spasm Basic Example\n\nExample Help Output:\n{}\n",
        ["-"].iter().cycle().take(40).copied().collect::<String>()
    );
    println!("{}\n", cmd.help());
    println!(
        "Running Dispatcher:\n{}\n",
        ["-"].iter().cycle().take(40).copied().collect::<String>()
    );

    cmd.evaluate(&args[..])
        .map(|flag_values| cmd.dispatch(flag_values))
        .expect("Flags should evaluate correctly");
}
