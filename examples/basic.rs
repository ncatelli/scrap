extern crate scrap;
use scrap::prelude::v1::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().into_iter().collect();

    let res = Cmd::new()
        .name("basic")
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
            println!("dispatched with config: {:?}", c);
            Ok(0)
        }))
        .flag(
            Flag::new()
                .name("test")
                .short_code("t")
                .action(Action::StoreTrue)
                .value_type(ValueType::Bool)
                .default_value(Value::Bool(false)),
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
