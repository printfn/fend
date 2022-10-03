# Welcome to the fend Manual!

fend is an arbitrary-precision unit-aware calculator. If you haven't already, head to [https://printfn.github.io/fend](https://printfn.github.io/fend) to use the online version, or click [here](#installation) to learn how to install fend on your computer.

You can check your version of fend at any time by typing `version`. If you are using the command-line interface, you can also run `fend --version`.

# Table of Contents
1. [Installation](#installation)
    1. [Windows](#windows)
    1. [macOS](#macos)
    1. [Arch Linux](#arch-linux)
    1. [NixOS / Nix package manager](#nixos-nix-package-manager)
    1. [Pre-built binaries](#pre-built-binaries)
    1. [Install via crates.io](#install-via-crates.io)
1. [Numbers](#numbers)
1. [Arithmetic](#arithmetic)
1. [Units](#units)
    1. [Temperature](#temperature)
1. [Dice](#dice)
1. [Functions and constants](#functions-and-constants)
1. [Number formats](#number-formats)
1. [Strings](#strings)
1. [Debug Representation](#debug-representation)
1. [Configuration](#configuration)
1. [Scripting](#scripting)
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

Or using [Chocolatey](https://chocolatey.org/):

```ps1
choco install fend
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

```{.include}
chapters/expressions.md
```

## Configuration

```{.include}
chapters/configuration.md
```

```{.toml include="../cli/src/default_config.toml"}
```

## Scripting

```{.include}
chapters/scripting.md
```

```{.include}
../CHANGELOG.md
```
