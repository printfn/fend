# Fend

[![Build Status](https://github.com/printfn/fend-rs/workflows/Rust/badge.svg)](https://github.com/printfn/fend-rs)
[![crates.io](https://img.shields.io/crates/v/fend)](https://crates.io/crates/fend)
[![downloads](https://img.shields.io/crates/d/fend)](https://crates.io/crates/fend)

Fend is an arbitrary-precision unit-aware calculator.

Unique features:

* Arbitrary-precision arithmetic using rational numbers
* Support for complex numbers
* Binary, octal, hexadecimal and all other bases between 2 and 36.
* Keeps track of units, with support for SI and US customary units.

## Installation

`cargo install fend`

You can then run `fend` to open a REPL session.

## [Manual](https://github.com/printfn/fend-rs/wiki)

You can find the Fend manual [here](https://github.com/printfn/fend-rs/wiki).

## Examples

```
> 5'10" to cm
177.8 cm
```

```
> cos (pi/4) + (sin (pi/4)) i
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

## License

Fend is MIT-licensed. See [LICENSE.md](LICENSE.md) for more information.
