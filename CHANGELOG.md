## Changelog

### v1.1.3 (2022-11-28)

* Add operators for permutations and combinations (by [@frectonz](https://github.com/frectonz))
    * `n permute k` or `n nPr k`: compute the number of `k`-permutations of `n`
    * `n choose k` or `n nCr k`: number of `k`-combinations of `n`
* Add `@noapprox` attribute to hide the `approx.` annotation in the output:
    ```
    > pi
    approx. 3.1415926535
    > @noapprox pi
    3.1415926535
    ```
* Add `@plain_number` attribute, to remove `approx.` and any units. This is especially useful in automated scripts.
    ```
    > 5 m / (3 s)
    approx. 1.6666666666 m / s
    > @plain_number 5 m / (3 s)
    1.6666666666
    ```
* Add a new date literal syntax, e.g. `@2000-01-01`
* Improve visual feedback when using the Telegram bot (by [@miki-tebe](https://github.com/miki-tebe))
* Add new SI prefixes quecca, ronna, ronto and quecto (by [@frectonz](https://github.com/frectonz))
* Add support for 256 (8-bit) colors in the CLI configuration
* Change `!debug` to `@debug` for consistency and improved shell script interoperability

### v1.1.2 (2022-10-31)

* Add `and` and `or` keywords as alternatives to the `&` and `|` bitwise operators
* Add Homebrew package (by [@rhysmdnz](https://github.com/rhysmdnz))
* Add Chocolatey package (by [@dgalbraith](https://github.com/dgalbraith))
* Fix incorrect description of command-line arguments in
    man page (by [@mcint](https://github.com/mcint))
* Support `_` in fend-web (stores the previous answer)
* Improve fend-web performance by serializing/storing variables properly
* Support case-insensitive currencies
* Support currency exchange rates in fend-wasm (by [@NickGeek](https://github.com/NickGeek))
* Support using any base currency for exchange rate calculations, not just USD

### v1.1.1 (2022-09-23)

* Add bitwise operators:
    * `&`: bitwise AND
    * `|`: bitwise OR
    * `xor`: bitwise XOR (`^` is already used for exponentiation)
    * `<<`: left shift
    * `>>`: right shift

### v1.1.0 (2022-09-22)

* Automatically download up-to-date currency exchange rates,
    e.g. when calculating `10 EUR to USD`
* fend can now read directly from files, e.g. `fend calculation.txt` will
    read and evaluate the contents of `calculation.txt`. Multiple files can be
    specified, as well as combinations of files and expressions.
* Shebangs (e.g. `#!/usr/bin/env fend`) no longer result in parse errors
* You can now use `--` to force fend to interpret arguments literally,
    e.g. `fend -- -V` is interpreted as `-1 volts` instead of showing the version number
* Fix bug where trailing whitespace or comments would result in a parse error
* Add a man page
* Add an MSI installer for Windows
* Remove MSIX installer, which was difficult to use due to it being unsigned
* Change fend website to [https://printfn.github.io/fend](https://printfn.github.io/fend)
* Move fend documentation to
    [https://printfn.github.io/fend/documentation](https://printfn.github.io/fend/documentation)
* Add a fend package to the Windows Package Manager (`winget`)

### v1.0.5 (2022-08-14)

* Add a `fend-wasm-nodejs` NPM package
* Add a [Telegram bot](https://t.me/fend_calc_bot)
* Improve behaviour of percentages (credit to @Techcable), e.g. `5% * 80kg` is now `4 kg`
* Add a Markdown-like parsing mode to the WASM API

### v1.0.4 (2022-07-22)

* Support `kn` for `knots`
* Add an installer for Windows

### v1.0.3 (2022-06-07)

* Support `rad` for `radians` (e.g. `10 RPM to rad/s`)
* Support implicit inches for `feet` (e.g. `5 foot 5 to cm`)
* Ensure fend does not exit when pressing Ctrl-C to clear the current line

### v1.0.2 (2022-06-01)

* Improve CLI colors
* Use Coulomb and Farad for prefixed units like `mC` or `µF`
* Add `sqmm` unit for square millimeters
* Add `point` unit for typographical points (i.e. 1/72 inch)
* Improve CLI behavior when pressing Ctrl-C to clear the current line

### v1.0.1 (2022-03-19)

* Support omitting inches when writing e.g. `5'1`: fend will now automatically
    interpret that as `5'1"`
* Improve reliability when piping data into fend on the command line, e.g.
    when running `echo "1+1" | fend`
* Support compiling fend with Rust 1.56 (rather than requiring Rust 1.59)

### v1.0.0 (2022-03-12)

* Allow `kmh` and `km/h` for kilometers per hour
* Change Planck's constant from `h` to `planck`
* Add support for lambda notation `λx.x` (in addition to the previous
    lambda notations `\x.x`, `x:x` and `x => x`)
* Add a ZSH helper script that makes `fend` more convenient on the
    command line
* Despite the major version bump, this release contains no breaking
    API changes

### v0.1.29 (2022-02-23)

* The locations for the config and history files have changed on some operating
    systems. On Linux and macOS, fend will now look in `~/.config/fend/config.toml`
    for its configuration file, and store history in `~/.local/state/fend/history`.
    You can run `fend help` to see which paths fend uses, and override them
    via the `FEND_CONFIG_DIR` and `FEND_STATE_DIR` environment variables
    if necessary.
* Colors in the CLI are now enabled by default. They can be disabled via the
    `enable-colors` config option, or via the `NO_COLOR` environment variable.
    `CLICOLOR` and `CLICOLOR_FORCE` environment variables are also respected.
    See https://bixense.com/clicolors/ and https://no-color.org for more info.
* Add a `max-history-size` config option to control how many history entries are
    saved by default.
* Improve error-checking when reading the config file. Minor errors will now only
    produce warnings, and no longer cause parsing to fail entirely.
* There are now Linux ARM builds available, for both `armv7-gnueabihf` and
    `aarch64` architectures.

### v0.1.28 (2022-02-12)

* Add base names `ternary` and `senary`
* Reduce CLI binary sizes

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

### v0.1.26 (2021-09-27)

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
