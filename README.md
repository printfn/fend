# Fend

[![build](https://github.com/printfn/fend-rs/workflows/build/badge.svg)](https://github.com/printfn/fend-rs)
[![codecov](https://codecov.io/gh/printfn/fend-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/printfn/fend-rs)
[![crates.io](https://img.shields.io/crates/v/fend)](https://crates.io/crates/fend)
[![downloads](https://img.shields.io/crates/d/fend)](https://crates.io/crates/fend)
[![docs.rs](https://docs.rs/fend-core/badge.svg)](https://docs.rs/fend-core)

Fend is an arbitrary-precision unit-aware calculator.

Unique features:

* Arbitrary-precision arithmetic using rational numbers
* Full support for complex numbers
* Binary, octal, hexadecimal and all other bases between 2 and 36
* Keeps track of units, with support for SI and US customary units
* Emacs-style CLI shortcuts
* Trigonometric functions

See the [manual](https://github.com/printfn/fend-rs/wiki) for more information.

## Installation

Install `fend` by running `cargo install fend` in your terminal. Alternatively you can download the latest stable binaries [here](https://github.com/printfn/fend-rs/releases/latest).

You can then run `fend` to start a REPL session:

```
$ cargo install fend
...
$ fend
> 1 ft to cm
30.48 cm
>
```

## [Manual](https://github.com/printfn/fend-rs/wiki)

You can find the Fend manual [here](https://github.com/printfn/fend-rs/wiki).

## Examples

```
> 5'10" to cm
177.8 cm
```

```
> cos (pi/4) + i * (sin (pi/4))
approx. 0.7071067811 + 0.7071067811i
```

```
> 0b1001 + 3
0b1100
```

```
> 1 light year to parsec
approx. 0.3066013937 parsec
```

## Changelog

See here: [CHANGELOG.md](CHANGELOG.md)

## License

Fend is MIT-licensed. See [LICENSE.md](LICENSE.md) for more information.
