# scrap
[![Code Quality (lint, test)](https://github.com/ncatelli/scrap/actions/workflows/code_quality.yml/badge.svg)](https://github.com/ncatelli/scrap/actions/workflows/code_quality.yml)

A minimal command-line utility framework built with zero external dependencies. This tool attempts to retain type information throughout the entire lifecycle of a command parse and usage with the intent of making as much of usage of this libary compile-time verifiable.

## Table of Contents
<!-- TOC -->

- [scrap](#scrap)
	- [Table of Contents](#table-of-contents)
	- [Warnings](#warnings)
	- [Dependencies](#dependencies)
	- [Testing](#testing)
		- [Locally](#locally)
	- [Examples](#examples)

<!-- /TOC -->

## Warnings
This tool was primarily built to support other projects that shared the same, no dependency goals and restrictions that I am currently working on. Use under the understanding that support for this will be best-effort.

## Dependencies
- rust 1.31+

## Testing
### Locally
Local tests are heavily implemented withing doctests and can be run using cargo's build in test subcommand.

```bash
$> cargo test
```

## Examples
The cli supports both a flat command, and a hierarchical set of commands, both covered in the following examples:

- [Flat Command](./examples/basic.rs)
- [Hierarchical](./examples/basic.rs)
