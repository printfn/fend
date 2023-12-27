---
title: fend
author: printfn
section: 1
---

# NAME

fend - arbitrary-precision unit-aware calculator

# SYNOPSIS

_fend_ **[OPTION | FILE | EXPRESSION]...** **[\--]** **[EXPRESSION]...**

# OPTIONS

**-h**, **\--help**
: Show help

**-v**, **-V**, **\--version**
: Show the current version number

**\--default-config**
: Print the default configuration file

**-e**, **\--eval** **\<expr>**
: Evaluate the given expression

**-f**, **\--file** **\<filename>**
: Read and evaluate the given file

# DESCRIPTION

```{.include}
chapters/expressions.md
```

# CONFIGURATION

```{.include}
chapters/configuration.md
```

```{.toml include="../cli/src/default_config.toml"}
```

# SCRIPTING

```{.include}
chapters/scripting.md
```

# EXIT VALUES

**0**
: Success

**1**
: Error

# BUGS

Bugs and feature suggestions can be reported at
[https://github.com/printfn/fend/issues](https://github.com/printfn/fend/issues).

# COPYRIGHT

fend is licensed under the GPL 3.0 (or later). You can find the source code at
[https://github.com/printfn/fend](https://github.com/printfn/fend).
