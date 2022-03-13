// contains a wrapper around terminal handling
//use std::{io, mem};

pub fn atty_stdout() -> bool {
    // check if stdout is a tty (which affects whether to show colors)
    atty::is(atty::Stream::Stdout)
}

pub fn atty_stdin() -> bool {
    // check if stdin is a tty (used for whether to show an
    // interactive prompt)
    atty::is(atty::Stream::Stdin)
}

/*#[cfg(unix)]
pub struct ExitRawMode {
    fd: libc::c_int,
    close_fd: bool,
    termios: libc::termios
}

impl Drop for ExitRawMode {
    fn drop(&mut self) {
        match wrap_result(unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.termios) }) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
        if self.close_fd {
            let _ = unsafe { libc::close(self.fd) };
        }
    }
}

#[cfg(unix)]
pub fn enable_raw_mode() -> io::Result<ExitRawMode> {
    let (fd, close_fd) = (libc::STDIN_FILENO, false);
    let mut termios = unsafe {
        let mut termios = mem::zeroed();
        wrap_result(libc::tcgetattr(fd, &mut termios))?;
        termios
    };
    let original_mode_ios = termios;

    unsafe {
        libc::cfmakeraw(&mut termios)
    }
    unsafe {
        wrap_result(libc::tcsetattr(fd, libc::TCSANOW, &termios))?;
    }

    Ok(ExitRawMode { fd, close_fd, termios: original_mode_ios })
}

fn wrap_result(result: i32) -> io::Result<()> {
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}*/
