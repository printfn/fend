# Contribution Guide

Thank you for contributing to fend!

Feel free to make any change you'd consider useful. If you need inspiration, you can
always look at the [GitHub Issue List](https://github.com/printfn/fend/issues).

Remember that fend is not primarily designed to be a programming language, but
intended to be an easy-to-use math/calculator tool. Ease-of-use is more important
than any internal consistency in the codebase.

This repository has a large number of unit and integration tests, for example
in [`core/tests/integration_tests.rs`](https://github.com/printfn/fend/blob/main/core/tests/integration_tests.rs).
While the tests are really useful for finding bugs and accidental regressions,
try not to take them as gospel! If you need to change any test, don't hesitate
to do so. User-friendliness is more important than strict backwards compatibility.
**Be bold!!**

If you want to add a unit definition to fend, take a look at
[`core/src/units/builtin.rs`](https://github.com/printfn/fend/blob/main/core/src/units/builtin.rs).

The repository is organised as a cargo workspace. `core` contains the fend back-end,
performs all the actual calculations, and exposes a small Rust API. It also contains
many unit and integration tests. `cli` depends on `core` and provides an interactive
command-line UI for fend. `wasm` contains Web Assembly bindings to fend, and provides
a JavaScript API. `web` contains code for the website
[printfn.github.io/fend](https://printfn.github.io/fend), which always
updates based on the `main` branch of this repository.

Make sure to run `cargo fmt` and `cargo clippy` before committing. If a particular
Clippy warning is hard to get rid of, you can always use an `#[allow(...)]` attribute.
To run unit and integration tests, run `cargo test`. These commands will automatically
apply to all Rust crates in the workspace.
