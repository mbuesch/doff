# Döff

![Logo](./assets/logo-64x64.png)

Desktop side-by-side diff viewer with merging and editing tools.

## Usage

Just run Döff and open two files to compare.

You can also specify the files to open as command-line arguments:

```sh
doff [LEFT_FILE] [RIGHT_FILE]
```

## Features

- Side-by-side diff view
- Inline editing of either file
- Merge changes from one file to the other

## Build

Requires [Rust](https://rust-lang.org/) and the [Dioxus](https://crates.io/crates/dioxus-cli):

```sh
cargo install dioxus-cli
```

Then build:

```sh
./build.sh
```

This produces the `doff` binary in the project root.

## License

MIT OR Apache-2.0
