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

Trailing newlines can be omitted by prefixing the calculation with
`@no_trailing_newline`, like so:

```bash
$ fend @no_trailing_newline 5+5
10
```
