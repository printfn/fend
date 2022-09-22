# Welcome to the fend Manual!

fend is an arbitrary-precision unit-aware calculator. If you haven't already, head to [https://printfn.github.io/fend](https://printfn.github.io/fend) to use the online version, or click [here](#installation) to learn how to install fend on your computer.

You can check your version of fend at any time by typing `version`. If you are using the command-line interface, you can also run `fend -v`.

# Table of Contents
1. [Installation](#installation)
    1. [Windows](#windows)
    1. [macOS](#macos)
    1. [Arch Linux](#arch-linux)
    1. [NixOS / Nix package manager](#nixos-nix-package-manager)
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
1. [Debug Representation](#debug-representation)
1. [Changelog](#changelog)

## Installation

You can use fend without installing by visiting [https://printfn.github.io/fend](https://printfn.github.io/fend).

If you want to install the fend command-line application, you have the following options:

### Windows

On Windows, you can install fend with a standard Windows installer package,
which you can download [here](https://github.com/printfn/fend/releases/latest/download/fend-windows-x64.msi).

Alternatively you can install fend via
[`winget`](https://docs.microsoft.com/en-us/windows/package-manager/winget/):

```ps1
winget install fend
```

### macOS

fend is available on [Homebrew](https://github.com/printfn/homebrew-fend/blob/main/Formula/fend.rb):

```bash
brew install printfn/fend/fend
```

### Arch Linux

fend is available on the [AUR](https://aur.archlinux.org/packages/fend-bin/):

```bash
yay -Syu aur/fend-bin
```

You can also [build it from source](https://aur.archlinux.org/packages/fend/) with:

```bash
yay -Syu aur/fend
```

### NixOS / Nix package manager

fend is available in [nixpkgs](https://github.com/NixOS/nixpkgs/blob/master/pkgs/tools/misc/fend/default.nix):

```bash
nix-env -iA nixpkgs.fend
```

Or using `nix-shell`:

```bash
nix-shell -p fend
```

### Pre-built binaries

You can download the latest stable binaries for Windows, macOS and Linux [here](https://github.com/printfn/fend/releases/latest).

Binaries are available for:

* Linux (aarch64)
* Linux (x86-64)
* Linux (armv7-gnueabihf)
* macOS (64-bit Intel)
* macOS (Apple Silicon)
* Windows (64-bit)

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
* `planck`: 6.62607015e-34 J s (Planck constant)
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

The notation `λx.x` is also supported.

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
\^    ASCII control character escape sequence,
          e.g. \^H for backspace (0x08)
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

* Linux: `$XDG_CONFIG_HOME/fend/config.toml` (usually `$HOME/.config/fend/config.toml`)
* macOS: `$HOME/.config/fend/config.toml`
* Windows: `\Users\{UserName}\.config\fend\config.toml`

You can always confirm the path that fend uses by typing `help`. You can also
see the default configuration file that fend uses by running `fend --default-config`.

You can override the config path location using the
environment variable `FEND_CONFIG_DIR`.

These are the options currently available, along with their default values:

```{.toml include="../cli/src/default_config.toml"}
```

fend stores its history file in `$HOME/.local/state/fend/history` by default,
although this can be overridden with the `FEND_STATE_DIR` environment variable.

Cache data is stored in `$HOME/.cache/fend` by default. This can be overridden
with the `FEND_CACHE_DIR` environment variable.


## Scripting

You can use `fend` programmatically using pipes or command-line arguments:

```bash
$ echo "sin (pi/4)" | fend
approx. 0.7071067811
$ fend "sqrt 2"
approx. 1.4142135619
```

The return code is 0 on success, or 1 if an error occurs during evaluation.

You can also specify filenames directly on the command-line, like this:

```bash
$ cat calculation.txt
16^2
$ fend calculation.txt
256
```

By default, fend will automatically try to read in files, or fall back to
evaluating expressions. This behavior can be overridden with these command-line
options:

* `-f` (or `--file`): read and evaluate the specified file
* `-e` (or `--eval`) evaluate the specified expression

For example:

```
$ cat calculation.txt
16^2
$ fend calculation.txt
256
$ fend -f calculation.txt
256
$ fend -e calculation.txt
Error: unknown identifier 'calculation.txt'
```

Or:

```
$ fend 1+1
2
$ fend -f 1+1
Error: No such file or directory (os error 2)
$ fend -e 1+1
2
```

`-f` and `-e` can be specified multiple times, in which case fend will evaluate
each specified expression one after the other. Any variables defined in earlier
expressions can be used by later expressions:

```bash
$ fend -e "a = 5" -e "2a"
10
```

## Debug Representation

You can see the debug representation of a value in fend by writing
`!debug` at the start of your calculation. For example, you can type:

```
> !debug 1+1
2 (unitless) (base 10, auto, simplifiable)
```

```{.include}
../CHANGELOG.md
```
