# [fend](https://fend.printfn.nz)

[fend](https://fend.printfn.nz) is an arbitrary-precision unit-aware calculator.

Unique features:

* Arbitrary-precision arithmetic using rational numbers
* Full support for complex numbers
* Binary, octal, hexadecimal and all other bases between 2 and 36
* Keep track of units, with support for SI, US and UK customary and many historical units
* Emacs-style CLI shortcuts
* Trigonometric functions
* Lambda calculus

See the [manual](https://github.com/printfn/fend/wiki) for more information.

## License

fend is MIT-licensed. See [LICENSE.md](LICENSE.md) for more information.

fend optionally includes the GPLv3-licenced `definitions.units` and
`currency.units` data files from [GNU Units](https://www.gnu.org/software/units/).
This can be changed via the `gpl` feature defined in `core/Cargo.toml`.
