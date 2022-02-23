// contains a wrapper around terminal handling

pub fn atty_stdout() -> bool {
    // check if stdout is a tty (which affects whether to show colors)
    console::user_attended()
}
