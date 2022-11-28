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

Bitwise operators are also supported. `&` is used for bitwise AND, `|`
for bitwise OR, and `xor` for bitwise XOR, since `^` is already used
for exponentiation. Left and right bitshifts can be done with `<<` and
`>>`.

The operator precedence for these is the same as in C, with bitshifts
having the highest precedence, followed by `&`, then `xor`, and finally
`|` which has the lowest precedence.

```
> 1 & 1
1
> 0xff & 0x100
0x0
> 0xff & 0xcb
0xcb
> 0b0011 | 0b0101
0b111
> 0b0011 xor 0b0101
0b110
> 1 << 2
4
> 7 >> 1
3
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
| `<<`, `>>` | | left |
| `&`, `and` | | left |
| `xor` | | left |
| `|`, `or` | | left |
| `choose`, `nCr` | | left |
| `permute`, `nPr` | | left |
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

## Dates

fend also has built-in syntax for dates:

```
> @1970-01-01
Thursday, 1 January 1970
> @2000-01-01 + 10000 days
Wednesday, 19 May 2027
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

## Debug Representation

You can see the debug representation of a value in fend by writing
`@debug` at the start of your calculation. For example, you can type:

```
> @debug 1+1
2 (unitless) (base 10, auto, simplifiable)
```
