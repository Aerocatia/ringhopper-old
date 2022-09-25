/// Output type.
#[derive(Copy, Clone)]
pub enum OutputType {
    /// Standard error
    Stderr,

    /// Standard output
    Stdout
}

/// Information about a TTY.
#[derive(Copy, Clone)]
pub struct TTYMetadata {
    /// Number of columns.
    pub width: usize,

    /// Number of rows.
    pub height: usize,

    /// Color output is supported.
    pub color: bool
}

impl TTYMetadata {
    /// Get the TTY metadata, or null if the output is not a TTY.
    #[allow(unused_variables)]
    pub fn get_tty_metadata(output: OutputType) -> Option<TTYMetadata> {
        if cfg!(target_os = "linux") {
            #[cfg(target_os = "linux")]
            {
                // Use the Linux API to get this
                let mut ws = libc::winsize { ws_col: 0, ws_row: 0, ws_xpixel: 0, ws_ypixel: 0 };
                let result = unsafe { libc::ioctl(match output { OutputType::Stdout => libc::STDOUT_FILENO, OutputType::Stderr => libc::STDERR_FILENO }, libc::TIOCGWINSZ, &mut ws as *mut libc::winsize) };

                return if result == 0 {
                    Some(TTYMetadata { width: ws.ws_col as usize, height: ws.ws_row as usize, color: true })
                }
                else {
                    None
                }
            }
        }
        else if cfg!(target_os = "windows") {
            #[cfg(target_os = "windows")]
            {
                // Use the Windows API to get this
                use windows::Win32::System::Console;

                let mut w = Console::CONSOLE_SCREEN_BUFFER_INFO::default();
                let handle = unsafe { Console::GetStdHandle(match output { OutputType::Stdout => Console::STD_OUTPUT_HANDLE, OutputType::Stderr => Console::STD_ERROR_HANDLE }).unwrap() };
                let is_terminal = unsafe { Console::GetConsoleScreenBufferInfo(handle, &mut w) }.as_bool();

                if !is_terminal {
                    return None
                }

                // Now check if color is supported
                let mut console_mode = Console::CONSOLE_MODE::default();
                let color_support = if unsafe { Console::GetConsoleMode(handle, &mut console_mode).as_bool() } {
                    if (console_mode & Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING).0 != 0 {
                        // We already have color support
                        true
                    }
                    else {
                        // Try enabling color support
                        unsafe { Console::SetConsoleMode(handle, console_mode | Console::ENABLE_VIRTUAL_TERMINAL_PROCESSING) }.as_bool()
                    }
                }
                else {
                    false
                };

                return Some(TTYMetadata {
                    width: w.srWindow.Right as usize - w.srWindow.Left as usize + 1,
                    height: w.srWindow.Bottom as usize - w.srWindow.Top as usize + 1,
                    color: color_support
                })
            }
        }
        else {
            return None
        }
        unreachable!();
    }
}
