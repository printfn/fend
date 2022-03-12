// contains a wrapper around terminal handling

pub fn atty_stdout() -> bool {
    // check if stdout is a tty (which affects whether to show colors)
    atty::is(atty::Stream::Stdout)
}

pub fn atty_stdin() -> bool {
    // check if stdin is a tty (used for whether to show an
    // interactive prompt)
    atty::is(atty::Stream::Stdin)
}
