extern crate scrap;
use scrap::prelude::v1::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().into_iter().collect();

    let res = Cmd::new()
        .name("subcommands")
        .description("this is a test")
        .author("John Doe <jdoe@example.com>")
        .version("1.2.3")
        .flag(
            Flag::new()
                .name("version")
                .short_code("v")
                .action(Action::StoreTrue)
                .value_type(ValueType::Bool),
        )
        .handler(Box::new(|c| {
            println!("root dispatched with config: {:?}", c);
            Ok(0)
        }))
        .subcommand(
            Cmd::new()
                .name("run")
                .description("execute an example subcommand")
                .handler(Box::new(|c| {
                    println!("run subcommand dispatched with config: {:?}", c);
                    Ok(0)
                })),
        )
        .run(args)
        .unwrap()
        .dispatch();

    match res {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e);
            std::process::exit(1)
        }
    }
}
