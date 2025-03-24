# [fend](https://printfn.github.io/fend)

[![build](https://github.com/printfn/fend/workflows/build/badge.svg)](https://github.com/printfn/fend)
[![codecov](https://codecov.io/gh/printfn/fend/branch/main/graph/badge.svg)](https://codecov.io/gh/printfn/fend)
[![crates.io](https://img.shields.io/crates/v/fend)](https://crates.io/crates/fend)
[![downloads](https://img.shields.io/crates/d/fend-core)](https://crates.io/crates/fend)

<a href="https://printfn.github.io/fend"><img alt="fend icon" src="https://raw.githubusercontent.com/printfn/fend/main/icon/icon.svg" width="100" /></a>

[fend](https://printfn.github.io/fend) is an arbitrary-precision unit-aware calculator.

Unique features:

* Arbitrary-precision arithmetic using rational numbers
* Full support for complex numbers
* D&D-style dice rolls
* Variables
* Binary, octal, hexadecimal and all other bases between 2 and 36
* Keep track of units, with support for SI, US and UK customary and many historical units
* Emacs-style CLI shortcuts
* Trigonometric functions
* Lambda calculus

See the [manual](https://printfn.github.io/fend/documentation/) for more information.

## [Web Interface](https://printfn.github.io/fend)

fend is available on the web at [printfn.github.io/fend](https://printfn.github.io/fend).

## Installation

The easiest way to install fend locally is via your package manager:

| Package Manager         | Command                    |
| ----------------------- | -------------------------- |
| Homebrew                | `brew install fend`        |
| MacPorts                | `sudo port install fend`   |
| AUR (Arch Linux)        | `yay -S aur/fend-bin`      |
| AOSC OS                 | `oma install fend`         |
| Xbps (Void Linux)       | `xbps-install fend`        |
| Nix                     | Add `nixpkgs.fend` to your config, or install ephemerally using `nix-shell -p fend`. |
| Cargo                   | `cargo install fend`       |
| Windows Package Manager | `winget install fend`      |
| Chocolatey              | `choco install fend`       |
| Scoop                   | `scoop install fend`       |
| Pkgx                    | `pkgx fend`                |

Alternatively, you can manually download the latest stable binaries
[here](https://github.com/printfn/fend/releases/latest).

Once fend is installed, run `fend` to start a REPL session:

```
$ fend
> 1 ft to cm
30.48 cm
>
```

### Packages

* [Homebrew](https://github.com/Homebrew/homebrew-core/blob/HEAD/Formula/f/fend.rb)
* [MacPorts](https://ports.macports.org/port/fend/)
* [AUR (built from source)](https://aur.archlinux.org/packages/fend/)
* [AUR (pre-built binary)](https://aur.archlinux.org/packages/fend-bin/)
* [xbps](https://github.com/void-linux/void-packages/tree/master/srcpkgs/fend)
* [AOSC OS](https://github.com/AOSC-Dev/aosc-os-abbs/tree/stable/app-utils/fend)
* [nixpkgs](https://github.com/NixOS/nixpkgs/blob/master/pkgs/tools/misc/fend/default.nix)
* [Windows Package Manager](https://github.com/microsoft/winget-pkgs/tree/master/manifests/p/printfn/fend)
* [Chocolatey](https://community.chocolatey.org/packages/fend)
* [NPM fend-wasm package](https://www.npmjs.com/package/fend-wasm)
* [NPM fend-wasm-web package](https://www.npmjs.com/package/fend-wasm-web)
* [NPM fend-wasm-nodejs package](https://www.npmjs.com/package/fend-wasm-nodejs)
* [Telegram Bot](https://t.me/fend_calc_bot)
* [Pkgx](https://pkgx.dev/pkgs/printfn.github.io/fend/)

## [Manual](https://printfn.github.io/fend/documentation/)

You can find the fend manual [here](https://printfn.github.io/fend/documentation/).

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
> 100 C to F
210 °F
```

```
> temperature = 30 °C
30 °C
> temperature to °F
86 °F
```

```
> roll d20
8
> roll 4d6
17
```

## Projects using fend

These are some projects making use of fend:

* [MicroPad](https://getmicropad.com)
* [Fendesk](https://github.com/SekoiaTree/fendesk)
* [metasearch2](https://github.com/mat-1/metasearch2)
* [FendApp](https://github.com/JadedBlueEyes/fendapp)

Feel free to make a pull request to add your own!

## License

fend is available under the MIT license. See [LICENSE.md](LICENSE.md)
for more information.
