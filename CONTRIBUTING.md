# Contribution Guide

Thank you for contributing to fend!

You can take a look at the [GitHub Issue List](https://github.com/printfn/fend/issues)
to find anything you'd like to improve. Alternatively, feel free to make any change
you'd consider useful.

The repository is organised as a cargo workspace. `core` contains the fend back-end,
performs all the actual calculations, and exposes a small Rust API. It also contains
many unit and integration tests. `cli` depends on `core` and provides an interactive
command-line UI for fend. `wasm` contains Web Assembly bindings to fend, and provides
a JavaScript API. `web` contains code for the website
[printfn.github.io/fend-website](https://printfn.github.io/fend-website), which always
updates based on the `main` branch of this repository.

Make sure to run `cargo fmt` and `cargo clippy` before committing. To run unit and
integration tests, run `cargo test`. These commands will automatically apply to
all Rust crates in the workspace.
