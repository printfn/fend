# Changelog

## Next release

* Support outputting mixed fractions (implicitly or via `to mixed_fraction`)
* Allow numbers that start with a decimal point, such as `.1`
* Support unmatched parentheses (e.g. `2+3)*(1+2` is `15`)
* Support parsing of numbers with recurring digits (e.g. `0.(3)` is equal to `1/3`)

## v0.1.5 (2020-09-29)

* Add support for lambda functions (e.g. `\x.x`)
* Change precedence of `to`, `as` and `->`
* Add live CLI output

## v0.1.4 (2020-09-15)

* Add the GNU units database, containing several thousand unit definitions
* Add support for square roots of numbers with units
* Add unary division as a shorthand for `1/x` (e.g. `/ second`, `64^/2`)
* Support `+` in exponential notation (e.g. `1.5e+2`)
* Allow `,` as a digit separator (e.g. `1,048,576`)

## v0.1.3 (2020-09-05)

* Use correct singular/plural unit names
* Base conversions using `to`
* Make exponents in other bases more consistent

## v0.1.2 (2020-09-02)

* Allow leading zeroes for non-decimal numbers
* Support interrupting an ongoing calculation with Ctrl-C

## v0.1.1 (2020-09-01)

* Save history continuously instead of only on program exit
* Support for Rust 1.43.1
* Fix parsing of `log10()` and `log2()`
* Add factorial operator (`!`)

## v0.1.0 (2020-08-31)

Initial release:

* Arbitrary-precision arithmetic using rational numbers
* Support for complex numbers
* Binary, octal, hexadecimal and other number bases between 2 and 36.
* Keeps track of units, with support for SI and US customary units.
* Support for Emacs-style CLI shortcuts
* Trigonometric functions
* Useful physical and mathematical constants
