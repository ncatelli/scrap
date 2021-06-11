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
            scrap::Flag::store_true("help", "h", "output help information.")
                .optional()
                .with_default(false),
        )
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
        .with_handler(|((_, version), test)| println!("Version: {}\nTest: {}", version, test));

    cmd.evaluate(&args[..])
        .map(|((help, version), test)| {
            if help {
                println!("{}", cmd.help())
            } else {
                cmd.dispatch(((help, version), test))
            }
        })
        .expect("Flags should evaluate correctly");
}
