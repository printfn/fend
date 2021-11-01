# Welcome to the fend Manual!

fend is an arbitrary-precision unit-aware calculator. If you haven't already, head to [https://printfn.github.io/fend-website](https://printfn.github.io/fend-website) to use the online version, or click [here](#installation) to learn how to install fend on your computer.

The current latest version of fend is `0.1.27`. You can check your version at any time by typing `version`. If you are using the command-line interface, you can also run `fend -v`.

# Table of Contents
1. [Installation](#installation)
    1. [Packages](#packages)
    1. [Pre-built binaries](#pre-built-binaries)
    1. [Install via crates.io](#install-via-cratesio)
1. [Numbers](#numbers)
1. [Arithmetic](#arithmetic)
1. [Units](#units)
    1. [Temperature](#temperature)
1. [Dice](#dice)
1. [Functions and constants](#functions-and-constants)
1. [Number formats](#number-formats)
1. [Strings](#strings)
1. [Configuration](#configuration)
1. [Scripting](#scripting)
1. [Changelog](#changelog)

## Installation

You can use fend without installing by visiting [https://printfn.github.io/fend-website](https://printfn.github.io/fend-website).

If you want to install the fend command-line application, you have the following options:

### Arch Linux

fend is available on the [AUR](https://aur.archlinux.org/packages/fend/):

```bash
yay -Sy aur/fend
```

### nix package manager

fend is available in [nixpkgs](https://github.com/NixOS/nixpkgs/blob/master/pkgs/tools/misc/fend/default.nix):

```bash
nix-env -iA nixpkgs.fend
```

### Pre-built binaries

You can download the latest stable binaries for 64-bit Windows, macOS and Linux [here](https://github.com/printfn/fend/releases/latest).

### Install via crates.io

If you have an up-to-date installation of Rust, you can install `fend` like so:

```bash
rustup update
cargo install fend
```

If you already have an older version of fend installed, this will automatically update to the latest version.

Once you have installed fend, you can launch an interactive REPL by typing `fend` in your terminal:

```
$ fend
> 1 ft to cm
30.48 cm
>
```

## Numbers

You can write numbers as integers or with a decimal point. Feel free to use `,` or `_` as a digit separator:

```
> 1234
1234
> 2.54
2.54
> 1,000,000
1000000
```

To write numbers in binary, octal or hexadecimal, add a `0b`, `0o` or `0x` prefix:

```
> 0x9 + 0x2
0xb
> 0b1000_0001
0b10000001
```

You can write in any other base (between 2 and 36 inclusive) by writing `<base>#<number>`. Here is an example of [senary (base 6)](https://en.wikipedia.org/wiki/Senary):

```
> 6#100 in decimal
36
> 36 to base 6
100
```

There is no difference between `to`, `as` or `in` to convert between bases, formats or units.

You can also use `e` to for exponential notation, like so:

```
> 1e10
10000000000
> 3e8
300000000
> 1.5e-6
0.0000015
> 1E3
1000
```

`i` can be used for complex numbers:

```
> i * i
-1
> (2 + 3i) * i
-3 + 2i
```

You can specify recurring digits by writing them in parentheses, like so:

```
> 0.(3)
approx. 0.3333333333
> 0.(3) to fraction
1/3
> 0.0(15)
approx. 0.0151515151
> 0.0(15) to fraction
1/66
```

## Arithmetic

fend supports the standard operators `+`, `-`, `*`, `/`, `^` and `!`, with the usual order of operations:

```
> 1 + 3 * 4
13
> 3^2
9
> (1 + 3) * 7
28
> 2pi
approx. 6.2831853071
> 5!
120
```

You can use `=` to declare variables:

```
> a = 1
1
> a
1
> a + 8
9
> a = 4 kg; b = 2; a * b^2
16 kg
```

These are all the supported operators:

| Operators | Precedence | Associativity |
| --- | --- | --- |
| Literals, Identifiers, Parentheses | highest | N/A |
| `of` | | right |
| `!` | | left |
| `^`, `**` | | right |
| `*`, `/`, `per`, function application (e.g. `sin 2`), `mod` | | left |
| mixed fractions (e.g. `1 2/3`), implicit sums (e.g. `5 feet 10 inches`) | | N/A |
| `+`, `-`, `to`, `as`, `in` | | left |
| `\ .`, `:`, `=>` | | left |
| `=` | | left |
| `;` | lowest | left |

The most recent calculation result is stored in a special variable `_` (or `ans`):

```
> 5 * 10
50
> _ + 5
55
> _ * 2
110
> ans * 2
220
```

## Units

fend supports many units, such as `kg`, `lb`, `N`, `lightyear`, etc. You can interchangeably use `to`, `as` and `in` to convert between units.

```
> 5'10" to cm
177.8 cm
> 1 mile to km
1.609344 km
> 1 GiB to bytes
1073741824 bytes
> 1m to kg
Error: cannot convert from m to kg: units are incompatible
```

### Temperature

Temperature units are handled differently to other units, because celsius (°C) and fahrenheit (°F) don't start at zero. Namely, absolute zero (0 kelvin) corresponds to -273.15 °C or -459.67 °F. This means that conversions between °C, °F and kelvin (K) need to differentiate between converting *absolute* temperatures and *differences* of temperatures.

If you use the `to` keyword to convert a plain temperature explicitly, fend will perform an *absolute* conversion. Here are some examples:

```
> 0 °C to °F
32 °F
> 100 °C to °F
212 °F
> 0 kelvin to °F
-459.67 °F
```

If, on the other hand, you add or subtract values with different temperature scales, fend performs relative conversions, like so:

```
> 0 °C + 1 kelvin
1 °C
> 0 kelvin + 9°F
5 kelvin
```

Additionally, conversions between more complex units (such as joules per degree celsius, i.e. `J / °C`) will always be relative:

```
> 100 J/K to J/°C
100 J / °C
> 100 J/K to J/°F
approx. 55.5555555555 J / °F
```

## Dice

fend has support for D&D-style dice syntax. For example, `d6` refers to a standard 6-sided die.

```
> roll d6
4
> roll d20 # 20-sided die
17
> roll 2d6 # sum of two 6-sided dice
7
> 2d6 # view the probability distribution
  2:  2.78%  #####
  3:  5.56%  ##########
  4:  8.33%  ###############
  5: 11.11%  ####################
  6: 13.89%  #########################
  7: 16.67%  ##############################
  8: 13.89%  #########################
  9: 11.11%  ####################
 10:  8.33%  ###############
 11:  5.56%  ##########
 12:  2.78%  #####
> roll(d20 + d6 + 4) # arithmetic operations
14
```

## Functions and constants

fend has a number of predefined functions:

* Roots: `sqrt`, `cbrt` for square roots and cube roots
* Standard trigonometric functions: `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
* Absolute value: `abs`
* Logarithms: `ln`, `log` (or `log10`), `log2`
* Exponential function (i.e. `e^x`): `exp`

Here are some examples of these functions:

```
> sin(1°)
approx. 0.0174524064
> asin 1
approx. 1.5707963267
> exp 2
approx. 7.3890560989
> abs (1 + i)
approx. 1.4142135619
```

Many constants are available, including:
* `pi`: approx. 3.1415926535
* `e`: approx. 2.7182818284
* `c`: 299792458 m/s (speed of light)
* `h`: 6.62607015e-34 J s (Planck constant)
* `boltzmann`: 1.380649e-23 J / K (Boltzmann constant)
* `avogadro`: 6.02214076e23 / mol (Avogadro constant)
* `electroncharge`, `electronmass`, `protonmass`, etc.

You can define your own lambda functions using either `\ .`, `:` or `=>`:

```
> \x.x
\x.x
> (\x.x) 5
5
> (\x.2x) 5
10
> (x: x to lb to 2 dp) (60 kg)
132.27 lbs
```

Even the [Y Combinator](https://en.wikipedia.org/wiki/Fixed-point_combinator#Fixed-point_combinators_in_lambda_calculus) can be defined as `\f.(\x.f (x x)) \x.f(x x)`.

## Number formats

fend supports a few different output formats. It tries to choose an appropriate format automatically based on the given number, but you can change it using the `to` operator. These are the currently supported formats:

* `auto`: This is the default format, which prints most numbers as decimals. For example, `1/4` is printed as `0.25`, while `1/3` becomes `approx. 0.3333333333`. Approximate values like π or 1/3 are printed to 10 decimal places in this format.
* `exact`: In this format numbers are printed as exact values whenever possible. `1/3` is shown as a fraction, and multiples of π are also shown directly without being approximated as decimals.
* `float`: In this format, the value is always printed as a "decimal" (albeit not necessarily in base 10), with arbitrary precision. [Recurring digits](https://en.wikipedia.org/wiki/Repeating_decimal) are represented using parentheses. For example, `1/3` is shown as `0.(3)` to indicate the repeating `3`s.
* `fraction` (or `frac`): In this format, any non-integer value is printed as its simplest fraction (i.e. the fraction with the lowest possible denominator). For example, `0.25` becomes `1/4`.
* `mixed_fraction` (or `mixed_frac`): Numbers larger than 1 are shown as mixed fractions, so `4/3` is written as `1 1/3`.
* `<n> sf`: Numbers are shown with the given number of significant figures. For example `pi to 3 sf` becomes `approx. 3.14`.
* `<n> dp`: This format shows the number as a decimal, with up to the given number of digits after the decimal point. Recurring digits will also be shown normally. For example, `1/3 to 5 dp` becomes `0.33333`.

## Strings

fend supports string literals, which can be enclosed in either single or double quotes. Strings are always encoded in UTF-8. Either type supports all the same escape sequences, which are as follows:

```
\\    backslash (\)
\"    double quote (")
\'    single quote (')
\a    bell (0x07)
\b    backspace (0x08)
\e    escape (0x1b)
\f    form feed (0x0c)
\n    line feed (0x0a)
\r    carriage return (0x0d)
\t    (horizontal) tab (0x09)
\v    vertical tab (0x0b)
\x    ASCII escape sequence, e.g. \x7e for '~'
\u    Unicode escape sequence, e.g. \u{3c0} for 'π'
\^    ASCII control character escape sequence, e.g. \^H for backspace (0x08)
\z    This causes fend to ignore any following whitespace characters
```

Here are some examples of using strings:

```
> "This is pi: \u{3c0}"
This is pi: π
> 'pi = ' + (pi to string)
pi = approx. 3.1415926535
> 'A' to codepoint
0x41
```

## Configuration

The CLI version of fend supports a configuration file.

The location of this file differs based on your operating system:
* Linux: `$XDG_CONFIG_HOME/fend` or `$HOME/.config/fend`
* macOS: `$HOME/Library/Application Support/fend`
* Windows: `{FOLDERID_RoamingAppData}\fend\config`

You can always confirm the path that fend uses by typing `help`.

These are the options currently available, along with their default values:

```toml
prompt = '> '
enable-colors = false

# change 'C' and 'F' to refer to coulomb and farad
# instead of degrees celsius and degrees fahrenheit
coulomb-and-farad = false

[colors]
number = {}
string = { foreground = 'yellow', bold = true }
identifier = { foreground = 'white' }
keyword = { foreground = 'blue', bold = true }
built-in-function = { foreground = 'blue', bold = true }
date = {}
other = {}
```

## Scripting

You can use `fend` programmatically using pipes or command-line arguments:

```bash
$ echo "sin (pi/4)" | fend
approx. 0.7071067811
$ fend "sqrt 2"
approx. 1.4142135619
```

The return code is 0 on success, or 1 if an error occurs during evaluation.

## Changelog

### v0.1.27 (2021-11-02)

* Improve command-line argument parsing, including support for multiple arguments
* The most recent calculation result is now stored in a special variable `_` (or `ans`):
    ```
    > 5 * 10
    50
    > _ + 5
    55
    > _ * 2
    110
    > ans * 2
    220
    ```

### v0.1.26

* Fix minor bug in the `fend-wasm-web` NPM package

### v0.1.25 (2021-09-27)

* Add `stone` unit
* Add initial support for date arithmetic, e.g. `('2020-05-04' to date) + 500 days`
* There is now a `fend-wasm-web` NPM package available, built with the `--target web` option

### v0.1.24 (2021-08-09)

* Add basic tab completion support to the CLI interface
* Improve output for `()` types in the CLI and in `fend-wasm`

### v0.1.23 (2021-08-06)

* Fully support empty statements and trailing semicolons

### v0.1.22 (2021-07-29)

* Add amp hour abbreviation (e.g. `mAh`)
* Improve error message when attempting to convert between incompatible units

### v0.1.21 (2021-07-11)

* Add support for D&D-style dice syntax. For example, `d6` refers to a standard 6-sided die.
    ```
    > roll d6
    4
    > roll d20 # 20-sided die
    17
    > roll 2d6 # sum of two 6-sided dice
    7
    > 2d6 # view the probability distribution
      2:  2.78%  #####
      3:  5.56%  ##########
      4:  8.33%  ###############
      5: 11.11%  ####################
      6: 13.89%  #########################
      7: 16.67%  ##############################
      8: 13.89%  #########################
      9: 11.11%  ####################
     10:  8.33%  ###############
     11:  5.56%  ##########
     12:  2.78%  #####
    > roll(d20 + d6 + 4) # arithmetic operations
    14
    ```
* Fix `lux` unit definition
* Remove the `->` conversion syntax: use `to` instead

### v0.1.20 (2021-06-29)

* Support modulo with arbitrarily large integers
* Add a config option to use `C` and `F` as coulomb and farad instead of
  degrees celsius and degrees fahrenheit
* Add more units: `horsepower`, `lumen`, `lux`, `decare` etc.

### v0.1.19 (2021-06-29)

* Add more units: `atmosphere`, `mmHg`, `inHg`, `dB`, `mil` and more
* Support variables over multiple statements in the wasm API

### v0.1.18 (2021-06-28)

* Add variables and multiple statements (e.g. `a = 4; b = 10; a * b` is `40`)
* Add `mixed_frac` as an alias for `mixed_fraction`
* Support the `£` symbol for GBP
* Allow `$` and `£` before the associated number, e.g. `$100/4` is `$25`

### v0.1.17 (2021-06-08)

* Add `k`, `M`, `G` and `T` suffixes for numbers (e.g. `5k` is `5000`)
* Add a modulo operator `mod` (e.g. `5 mod 2` is `1`)
* Improve error messages for invalid unit conversions
* Add the golden ratio constant phi ϕ (`phi` is `approx. 1.6180339886`)
* Fix incorrect currency exchange rates
* Add `true`, `false`, and a `not()` function
* Add `sqm` and `sqft` units for square meters and square feet respectively

### v0.1.16 (2021-05-21)

* Add support for Unicode operators, such as ✕ or ÷
* Add color customization to the command-line interface by editing the `config.toml` file. Refer to the default `config.toml` file [here](https://github.com/printfn/fend/blob/main/cli/src/default_config.toml).

### v0.1.15 (2021-05-20)

* Case-insensitive units: you can now write `5 Meters`
* Experimental date support:
  * You can create a date object like so:
    ```
    > "2021-05-20" to date
    Thursday, 20 May 2021
    ```
  * No other date-related functionality has been implemented yet, including no times/timezones, durations, date arithmetic or different date formats.
* fend now parses single and double quotes as feet and inches (instead of as string literals) in more situations, so you can once again write:
    ```
    > 1.2192 m to '
    4'
    ```
* The CLI program can now read options from a config file. Type `help` to find out where it is stored. The `config.toml` file can contain the following options:
    ```toml
    prompt = '> '
    color = false
    ```
* Terminal color support: this is disabled by default, so you'll need to create a config file containing `color = true` first.
* Added a `conjugate` function that computes the complex conjugate of a number: e.g. `conjugate(i)` is `-i`.
* Improve consistency around error messages

### v0.1.14 (2021-02-14)

* Add support for strings and string literals. They can be in single or double quotes, and support a variety of escape sequences.
* Support conversions of numbers to strings
* Use `b` as shorthand for bits (including e.g. `Gb/s`)
* Remove the `0d` number prefix

### v0.1.13 (2021-01-20)

* Add °C and °F (including temperature conversions)
* Automatically simplify units in some calculations (e.g. `100 km/hr * 36 seconds` becomes `1 km`)
* Add initial support for objects (try e.g. `mass of earth`)
* Add `square` and `cubic` functions
* Add hectares and acres

### v0.1.12 (2020-11-27)

This build was only released on NPM.

* Fix NPM package

### v0.1.11 (2020-11-27)

* Improve debug representation (using e.g. `!debug 1`)

### v0.1.10 (2020-11-23)

* Allow leading zeroes for decimal numbers
* Allow upper-case "E" for exponential numbers (e.g. `1E3` is `1000`)
* Add `in` as an alias for `to` (so you can now write `3.14 in binary`)
* Add `log()` function as shorthand for `log10()`
* Fix `kWh` unit definition

### v0.1.9 (2020-11-06)

* Include `LICENSE.md` files as part of the package on [crates.io](https://crates.io)

### v0.1.8 (2020-11-06)

* Add `version` command to get the current fend version
* Support `-sin pi` and `3 sin pi` without parentheses
* Support function inverses: e.g. `sin^-1` becomes `asin`
* Add `to sf` to convert numbers to a fixed number of significant figures
* Make many calculations involving `sin` and `cos` exact
* Add `tau` (`τ`), equal to `2π`
* Add Yi- and Zi- binary prefixes
* Use decimal approximations instead of fractions by default
* `x to exact` will now convert `x` to an exact representation whenever possible, including using multiples of π
* Add `cis` as a shorthand for `cos θ + i * (sin θ)`

### v0.1.7 (2020-10-14)

* Ensure that approximate numbers are always marked as such
* Fix a bug involving sin/cos/tan of negative numbers
* Make some calculations involving pi exact
* Fix parsing of recurring decimals in bases other than 10

### v0.1.6 (2020-10-05)

* Support outputting mixed fractions (implicitly or via `to mixed_fraction`)
* Support unmatched parentheses (e.g. `2+3)*(1+2` is `15`)
* Support parsing of numbers with recurring digits (e.g. `0.(3)` is equal to `1/3`)
* Allow numbers that start with a decimal point, such as `.1`

### v0.1.5 (2020-09-29)

* Add support for lambda functions (e.g. `\x.x`)
* Change precedence of `to` and `as`
* Add live CLI output

### v0.1.4 (2020-09-15)

* Add the GNU units database, containing several thousand unit definitions
* Add support for square roots of numbers with units
* Add unary division as a shorthand for `1/x` (e.g. `/ second`, `64^/2`)
* Support `+` in exponential notation (e.g. `1.5e+2`)
* Allow `,` as a digit separator (e.g. `1,048,576`)

### v0.1.3 (2020-09-05)

* Use correct singular/plural unit names
* Base conversions using `to`
* Make exponents in other bases more consistent

### v0.1.2 (2020-09-02)

* Allow leading zeroes for non-decimal numbers
* Support interrupting an ongoing calculation with Ctrl-C

### v0.1.1 (2020-09-01)

* Save history continuously instead of only on program exit
* Fix parsing of `log10()` and `log2()`
* Add factorial operator (`!`)

### v0.1.0 (2020-08-31)

Initial release:

* Arbitrary-precision arithmetic using rational numbers
* Support for complex numbers
* Binary, octal, hexadecimal and other number bases between 2 and 36.
* Keeps track of units, with support for SI and US customary units.
* Support for Emacs-style CLI shortcuts
* Trigonometric functions
* Useful physical and mathematical constants
