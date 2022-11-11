use crate::ansi;
use crate::history::History;
use std::io;
use std::io::{stdout, Write};
use std::process::exit;

pub struct Terminal {
    buffer: Vec<u8>,
    buffer_index: usize,
    prompt: String,
    origin_termios: libc::termios,
    history: Option<History>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            buffer_index: 0,
            prompt: String::new(),
            origin_termios: termios(),
            history: None,
        }
    }

    pub fn history(&mut self, history: History) -> &mut Terminal {
        self.history = Some(history);
        self
    }

    pub fn prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }

    pub fn read_line(&mut self) -> io::Result<String> {
        self.set_raw_mode();

        self.init_buffer()?;

        let stdout = stdout();

        let mut stdout = stdout.lock();

        loop {
            stdout.flush()?;

            if let Some(char) = getch() {
                match char {
                    0 => continue,
                    3 => {
                        self.unset_raw_mode();
                        exit(0)
                    }

                    10 => break,

                    27 => {
                        if getch().unwrap_or(27) != 91 {
                            continue;
                        }

                        match getch().unwrap_or(91) {
                            //up
                            65 => {
                                if let Some(history) = self.history.as_mut() {
                                    if let Some(string) = history.next() {
                                        self.buffer.clear();
                                        self.buffer_index = 0;
                                        self.buffer.write_all(string.as_bytes())?;

                                        stdout.write_all(
                                            format!(
                                                "\r{}{}",
                                                ansi::Cursor::ClearLine.get_esc_code(),
                                                self.prompt
                                            )
                                            .as_bytes(),
                                        )?;

                                        stdout.write_all(
                                            format!(
                                                "\r{}{}",
                                                self.prompt,
                                                String::from_utf8_lossy(&self.buffer)
                                            )
                                            .as_bytes(),
                                        )?;
                                    }
                                }
                            }

                            //down
                            66 => {
                                if let Some(history) = self.history.as_mut() {
                                    if let Some(string) = history.prev() {
                                        self.buffer.clear();
                                        self.buffer_index = 0;
                                        self.buffer.write_all(string.as_bytes())?;

                                        stdout.write_all(
                                            format!(
                                                "\r{}{}",
                                                ansi::Cursor::ClearLine.get_esc_code(),
                                                self.prompt
                                            )
                                            .as_bytes(),
                                        )?;

                                        stdout.write_all(
                                            format!(
                                                "\r{}{}",
                                                self.prompt,
                                                String::from_utf8_lossy(&self.buffer)
                                            )
                                            .as_bytes(),
                                        )?;
                                    }
                                }
                            }

                            //right
                            67 => {
                                if self.buffer_index < self.buffer.len() {
                                    self.buffer_index += 1;
                                    stdout.write_all(
                                        format!("{}", ansi::Cursor::Right.get_esc_code())
                                            .as_bytes(),
                                    )?;
                                }
                            }

                            //left
                            68 => {
                                if self.buffer_index > 0 {
                                    stdout.write_all(
                                        format!("{}", ansi::Cursor::Left.get_esc_code()).as_bytes(),
                                    )?;
                                    self.buffer_index -= 1;
                                }
                            }
                            _ => continue,
                        }
                    }

                    127 => {
                        self.backspace()?;
                    }

                    _ => {
                        self.buffer.insert(self.buffer_index, char);

                        self.buffer_index += 1;

                        for i in 0..self.buffer.len() {
                            if i != 0 {
                                stdout.write_all(
                                    format!("{}", ansi::Cursor::Backspace.get_esc_code())
                                        .as_bytes(),
                                )?;
                            }
                        }

                        stdout.write_all(
                            format!("\r{}{}", self.prompt, String::from_utf8_lossy(&self.buffer))
                                .as_bytes(),
                        )?;

                        if self.buffer_index < self.buffer.len() {
                            let move_position = self.prompt.len() + self.buffer_index;

                            stdout.write_all(
                                format!("{}", ansi::Cursor::Move(move_position).get_esc_code())
                                    .as_bytes(),
                            )?;
                        }
                    }
                }
            }
        }

        self.unset_raw_mode();

        stdout.write(b"\n")?;

        let string = String::from_utf8_lossy(&self.buffer);

        if let Some(history) = self.history.as_mut() {
            history.insert(string.to_string());
        }

        Ok(string.to_string())
    }

    fn backspace(&mut self) -> io::Result<()> {
        let stdout = stdout();
        let mut stdout = stdout.lock();

        if self.buffer_index <= 0 {
            return Ok(());
        }

        self.buffer_index -= 1;

        for i in 0..self.buffer.len() {
            if i != 0 {
                stdout
                    .write_all(format!("{}", ansi::Cursor::Backspace.get_esc_code()).as_bytes())?;
            }
        }

        stdout.write_all(
            format!("\r{}{}", self.prompt, String::from_utf8_lossy(&self.buffer)).as_bytes(),
        )?;

        self.buffer.remove(self.buffer_index);

        stdout.write_all(format!("{}", ansi::Cursor::Backspace.get_esc_code()).as_bytes())?;
        stdout.write_all(
            format!(
                "\r{}{}",
                self.prompt,
                String::from_utf8_lossy(&self.buffer).to_string()
            )
            .as_bytes(),
        )?;

        if self.buffer_index < self.buffer.len() {
            let move_position = self.prompt.len() + self.buffer_index - 1;
            stdout.write_all(
                format!("{}", ansi::Cursor::Move(move_position).get_esc_code()).as_bytes(),
            )?;
        }

        Ok(())
    }

    fn init_buffer(&mut self) -> io::Result<()> {
        let stdout = stdout();
        let mut stdout = stdout.lock();

        self.buffer.clear();

        self.buffer_index = 0;

        if self.buffer_index <= self.buffer.len() {
            let move_position = self.prompt.len() + 1;
            stdout.write_all(
                format!(
                    "\r{}{}",
                    self.prompt,
                    ansi::Cursor::Move(move_position).get_esc_code()
                )
                .as_bytes(),
            )?;
        }

        Ok(())
    }

    fn set_raw_mode(&mut self) {
        unsafe { libc::tcgetattr(0, &mut self.origin_termios) };

        let mut raw = self.origin_termios;

        raw.c_lflag = raw.c_lflag & !(libc::ICANON | libc::ECHO | libc::IEXTEN | libc::ISIG);
        // raw.c_lflag = raw.c_lflag & !(libc::ICANON | libc::ECHO );
        raw.c_cc[libc::VTIME] = 0;

        raw.c_cc[libc::VMIN] = 1;

        unsafe {
            libc::tcsetattr(0, 0, &raw);
            libc::fcntl(0, libc::F_SETFL);
        }
    }

    fn unset_raw_mode(&mut self) {
        unsafe {
            libc::tcsetattr(0, 0, &self.origin_termios);
        }
    }
}

fn getch() -> Option<u8> {
    let code = [0; 1];

    let n = unsafe { libc::read(0, code.as_ptr() as *mut libc::c_void, 1) };

    if n <= 0 {
        return None;
    }

    Some(code[0])
}

#[cfg(target_os = "macos")]
fn termios() -> libc::termios {
    libc::termios {
        c_cc: [0u8; 20],
        c_ispeed: 0,
        c_ospeed: 0,
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
    }
}

#[cfg(target_os = "linux")]
fn termios() -> libc::termios {
    libc::termios {
        c_line: 0,
        c_cc: [0; 32],
        c_ispeed: 0,
        c_ospeed: 0,
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
    }
}
