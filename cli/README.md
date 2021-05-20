[fend](https://printfn.github.io/fend-website) is an arbitrary-precision unit-aware calculator.

This is the cross-platform CLI for fend.

Unique features:

* Arbitrary-precision arithmetic using rational numbers
* Full support for complex numbers
* Binary, octal, hexadecimal and all other bases between 2 and 36
* Keep track of units, with support for SI, US and UK customary and many historical units
* Emacs-style CLI shortcuts
* Trigonometric functions
* Lambda calculus

See the [manual](https://github.com/printfn/fend/wiki) for more information.

## [Web Interface](https://printfn.github.io/fend-website)

fend is available on the web at [printfn.github.io/fend-website](https://printfn.github.io/fend-website).

## Installation

The easiest way to install fend locally is via your package manager.

On systems with Rust:

```bash
rustup update
cargo install fend
```

On Arch Linux from the AUR (using `yay`):

```bash
yay -Sy aur/fend
```

Using the nix package manager:

```bash
nix-env -iA nixpkgs.fend
```

You can also download the latest stable binaries [here](https://github.com/printfn/fend/releases/latest).

Once fend is installed, run `fend` to start a REPL session:

```
$ fend
> 1 ft to cm
30.48 cm
>
```

### Packages

* [AUR](https://aur.archlinux.org/packages/fend/)
* [nixpkgs](https://github.com/NixOS/nixpkgs/blob/master/pkgs/tools/misc/fend/default.nix)

## [Manual](https://github.com/printfn/fend/wiki)

You can find the fend manual [here](https://github.com/printfn/fend/wiki).

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
> 0xffff to decimal
65535
```

```
> 100 °C to °F
210 °F
```

```
> 1 lightyear to parsecs
approx. 0.3066013937 parsecs
```

```
> (x: 2x) pi
approx. 6.2831853071
```

## License

fend is MIT-licensed. See [LICENSE.md](LICENSE.md) for more information.
