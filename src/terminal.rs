use crate::ansi;
use libc;
use std::cell::RefCell;
use std::io;
use std::io::{stdout, Stdout, Write};
use std::process::exit;

pub struct Terminal {
    buffer: Vec<u8>,
    buffer_index: usize,
    prompt: String,
    stdout: RefCell<Stdout>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            buffer_index: 0,

            prompt: String::new(),

            stdout: RefCell::new(stdout()),
        }
    }

    pub fn prompt(&mut self, prompt: String) -> &mut Terminal {
        self.prompt = prompt.to_string();
        self
    }

    fn backspace(&mut self) -> io::Result<()> {
        let mut stdout = self.stdout.borrow_mut().lock();

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
        self.buffer.clear();

        self.buffer_index = 0;

        if self.buffer_index <= self.buffer.len() {
            let move_position = self.prompt.len() + 1;
            self.stdout.borrow_mut().lock().write_all(
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

    pub fn read_line(&mut self) -> io::Result<String> {
        let mut termios = termios();

        set_raw_mode(&mut termios);

        let mut stdout = self.stdout.borrow_mut().lock();

        self.init_buffer()?;
        
        
        
        
        
        loop {
            stdout.flush().unwrap_or_default();
            if let Some(code) = getch() {
                match code {
                    [0] => continue,
                    [3] => {
                        unset_raw_mode(&mut termios);

                        exit(0)
                    }

                    [10] => break,

                    [27] => {
                        if getch().unwrap_or([27]) != [91] {
                            continue;
                        }

                        match getch().unwrap_or([91]) {
                            //up
                            [65] => {}

                            //down
                            [66] => {}

                            //right
                            [67] => {
                                if self.buffer_index < self.buffer.len() {
                                    self.buffer_index += 1;
                                    stdout.write_all(
                                        format!("{}", ansi::Cursor::Right.get_esc_code())
                                            .as_bytes(),
                                    )?;
                                }
                            }
                            //left
                            [68] => {
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
                    [127] => {
                        self.backspace()?;
                    }
                    _ => {
                        let code = match code.get(0) {
                            Some(code) => *code,
                            None => continue,
                        };

                        self.buffer.insert(self.buffer_index, code);
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

        unset_raw_mode(&mut termios);

        stdout.write(b"\n")?;


        Ok(String::from_utf8_lossy(&self.buffer).to_string())
    }
}

fn getch() -> Option<[u8; 1]> {
    let code = [0; 1];

    let n = unsafe { libc::read(0, code.as_ptr() as *mut libc::c_void, 1) };

    if n <= 0 {
        return None;
    }

    Some(code)
}

fn unset_raw_mode(termios: &mut libc::termios) {
    termios.c_lflag = libc::ECHO | libc::ICANON;

    unsafe {
        libc::tcsetattr(0, 0, termios);
    }
}

fn set_raw_mode(termios: &mut libc::termios) {
    unsafe {
        libc::tcgetattr(0, termios);
    };

    termios.c_lflag = termios.c_lflag & !(libc::ECHO | libc::ICANON);
    termios.c_cc[libc::VTIME] = 0;
    termios.c_cc[libc::VMIN] = 1;

    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, termios);
        // libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK);
        libc::fcntl(0, libc::F_SETFL);
    };
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