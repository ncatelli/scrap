# scrap
[![Code Quality (lint, test)](https://github.com/ncatelli/scrap/actions/workflows/code_quality.yml/badge.svg)](https://github.com/ncatelli/scrap/actions/workflows/code_quality.yml)

A minimal command-line utility framework built with zero external dependencies. This tool attempts to retain type information throughout the entire lifecycle of a command parse with the intent of lifting validation of a command's handler to compile-time verifiable.

## Table of Contents
<!-- TOC -->

- [scrap](#scrap)
	- [Table of Contents](#table-of-contents)
	- [Features](#features)
	- [Dependencies](#dependencies)
	- [Getting Started](#getting-started)
		- [Installing](#installing)
		- [Basic Example](#basic-example)
		- [Additional examples](#additional-examples)
		- [Extending from 10,000 Feet](#extending-from-10000-feet)
			- [Evaluatable](#evaluatable)
			- [Helpable](#helpable)
			- [ShortHelpable](#shorthelpable)
			- [Dispatchable](#dispatchable)
			- [Putting it all together](#putting-it-all-together)
	- [Testing](#testing)
		- [Locally](#locally)
	- [Warnings](#warnings)

<!-- /TOC -->

## Features
- Type-safe evaluators and handlers
- Easy-to-use
- Easy to extend via a trait-based api.
  - `ShortHelpable`: Short help documentation generators
  - `Helpable`: help documentation generators
  - `Evauatable`: Flag and argument parsers
  - `Dispatchers`: Handler executors

## Dependencies
- rust 1.50+

## Getting Started
### Installing
Currently this is not available on crates.io and must be installed via git.

```toml
scrap = { git = "https://github.com/ncatelli/scrap", branch = "main" }
```

### Basic Example
```rust
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
    let cmd = scrap::Cmd::new("minimal")
        .description("A minimal example cli.")
        .author("John Doe <jdoe@example.com>")
        .version("1.2.3")
        .with_flag(help)
        .with_flag(direction)
        // Finally a handler is defined with its signature being a product of
        //the cli's defined flags.
        .with_handler(|(_, direction)| println!("You chose {}.", direction));

    // The help method generates a help command based on the output rendered
    // from all defined flags.
    let help_string = cmd.help();

    // Evaluate attempts to parse the input, evaluating all commands and flags
    // into concrete types which can be passed to `dispatch`, the defined
    // handler.
    let res = cmd
        .evaluate(&args[..])
        .map_err(|e| e.to_string())
        .and_then(|(help, direction)| {
            if help {
                Err("output help".to_string())
            } else {
                cmd.dispatch((help, direction));
                Ok(())
            }
        });

    match res {
        Ok(_) => (),
        Err(_) => println!("{}", help_string),
    }
}
```

### Additional examples
The cli supports both a flat command, and a hierarchical set of commands, both covered in the following examples:

- [Flat Command](./examples/basic.rs)
- [Hierarchical](./examples/subcommands.rs)

### Extending from 10,000 Feet
The API for extending commands is built around three primary traits.

#### Evaluatable
Evaluatable provides the functionality to evaluate a given input for a match. Below defines an example of an evaluator that reads any `&str` at the head of the input, converting it into a `String`.

```rust
impl<'a> Evaluatable<'a, &'a [&'a str], String> for StringValue {
    fn evaluate(&self, input: &'a [&'a str]) -> EvaluateResult<'a, String> {
        input
            .get(0)
            .map(|v| v.to_string())
            .ok_or(CliError::ValueEvaluation)
    }
}
```

`Evaluatable` provides a simple input-output functionality allowing `Evaluatable` types to be composed. An example of this functionality is the `Join`, which merges the outputs of two `Evaluatables` (which we will call `E1` and `E2`) into a tuple, `(O1, O2)`.

```rust
#[derive(Debug)]
pub struct Join<E1, E2> {
    evaluator1: E1,
    evaluator2: E2,
}

impl<E1, E2> IsFlag for Join<E1, E2> {}

impl<E1, E2> Join<E1, E2> {
    /// Instantiates a new instance of Join with two given evaluators.
    pub fn new(evaluator1: E1, evaluator2: E2) -> Self {
        Self {
            evaluator1,
            evaluator2,
        }
    }
}

impl<'a, E1, E2, A, B, C> Evaluatable<'a, A, (B, C)> for Join<E1, E2>
where
    A: Copy + std::borrow::Borrow<A> + 'a,
    E1: Evaluatable<'a, A, B>,
    E2: Evaluatable<'a, A, C>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, (B, C)> {
        self.evaluator1
            .evaluate(input)
            .map_err(|e| e)
            .and_then(|e1_res| match self.evaluator2.evaluate(input) {
                Ok(e2_res) => Ok((e1_res, e2_res)),
                Err(e) => Err(e),
            })
    }
}
```

#### Helpable
`Helpable` provides the behavior for generating help strings from any given object. This allows for the building of a cli's help string solely from its consituent parts. A basic example of a common helpable definition.

```rust


impl<H> Helpable for Cmd<(), H> {
    type Output = String;

    fn help(&self) -> Self::Output {
        format!(
            "Usage: {} [OPTIONS]\n{}\nFlags:\n",
            self.name, self.description,
        )
    }
}
```

#### ShortHelpable
`ShortHelpable`, much like `Helpable` provides the behavior for generating short-help strings. This can be thought of as the consituent parts of a larger help string.

#### Dispatchable
Dispatchable provides a method, `dispatch` who's signature is equivalent to the output of all Flag `Evaluatable`s.

To illustrate this behavior, I will reference the above `Join`.

```rust
impl<'a, E1, E2, A, B, C> Evaluatable<'a, A, (B, C)> for Join<E1, E2>
where
    A: Copy + std::borrow::Borrow<A> + 'a,
    E1: Evaluatable<'a, A, B>,
    E2: Evaluatable<'a, A, C>,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, (B, C)> {
        self.evaluator1
            .evaluate(input)
            .map_err(|e| e)
            .and_then(|e1_res| match self.evaluator2.evaluate(input) {
                Ok(e2_res) => Ok((e1_res, e2_res)),
                Err(e) => Err(e),
            })
    }
}
```

If given the above `Evaluatable` A cli's implemented dispatchable would take an `Fn((B, C))` and yield whatever return type is defined for the closure.

#### Putting it all together
To illustrate how easy it is to write custom `Evaluator` implementations, I will show an example of a `WithOpen` evaluator below, which takes an evaluator that yields a type marked `Openable` and attempts to open the resulting value as a file. 

```rust
pub trait Openable {}

#[derive(Debug)]
pub struct WithOpen<E> {
    evaluator: E,
}

impl<E> IsFlag for WithOpen<E> {}

impl<E> WithOpen<E> {
    pub fn new(evaluator: E) -> Self {
        Self { evaluator }
    }
}

impl<'a, E, A> Evaluatable<'a, A, std::fs::File> for WithOpen<E>
where
    A: 'a,
    E: Evaluatable<'a, A, String> + Openable,
{
    fn evaluate(&self, input: A) -> EvaluateResult<'a, std::fs::File> {
        self.evaluator.evaluate(input).and_then(|fp| {
            std::fs::File::open(&fp).map_err(|e| {
                CliError::FlagEvaluation(format!("unable to open file evaluator: {}", e))
            })
        })
    }
}

impl<E> ShortHelpable for WithOpen<E>
where
    E: ShortHelpable<Output = FlagHelpCollector> + Defaultable,
{
    type Output = FlagHelpCollector;

    fn short_help(&self) -> Self::Output {
        match self.evaluator.short_help() {
            // Provides a small helper for combining strings.
            FlagHelpCollector::Single(fhc) => {
                FlagHelpCollector::Single(fhc.with_modifier("will_open".to_string()))
            }
            fhcj => fhcj,
        }
    }
}
```

## Testing
### Locally
Local tests are heavily implemented within doctests and can be run using cargo's build in test subcommand.

```bash
$> cargo test
```

## Warnings
This tool was primarily built to support other projects that shared the same, no dependency goals and restrictions that I am currently working on. Use under the understanding that support for this will be best-effort.