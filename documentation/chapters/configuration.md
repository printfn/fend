The CLI version of fend supports a configuration file.

The location of this file differs based on your operating system:

* Linux: `$XDG_CONFIG_HOME/fend/config.toml` (usually `$HOME/.config/fend/config.toml`)
* macOS: `$HOME/.config/fend/config.toml`
* Windows: `\Users\{UserName}\.config\fend\config.toml`

You can always confirm the path that fend uses by typing `help`. You can also
see the default configuration file that fend uses by running `fend --default-config`.

You can override the config path location using the
environment variable `FEND_CONFIG_DIR`.

fend stores its history file in `$HOME/.local/state/fend/history` by default,
although this can be overridden with the `FEND_STATE_DIR` environment variable.

Cache data is stored in `$HOME/.cache/fend` by default. This can be overridden
with the `FEND_CACHE_DIR` environment variable.

These are the configuration options currently available, along with their default values:
