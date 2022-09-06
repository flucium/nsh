use crate::ansi;
use crate::builtin;
use crate::parser::Command;
use crate::parser::Redirect;
use crate::parser::{lexer::Lexer, Error, Node, Parser};
use crate::prompt;
use crate::variable::Variable;
use std::env;
use std::f32::consts::PI;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Stderr;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::process::Child;
use std::process::ChildStdout;
use std::process::CommandArgs;
use std::result;

pub struct Shell {
    prompt: String,
    variable: Variable,
    termios: libc::termios,
    stdin: Option<process::Stdio>,
    stdout: Option<process::Stdio>,
    stderr: Option<process::Stdio>,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            prompt: String::new(),
            variable: Variable::new(),
            termios: termios(),
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn initialize(&mut self) -> &mut Self {
        self.load_profile();

        self
    }

    fn load_profile(&mut self) {
        let local_profile = Profile::new(ProfileKind::Local);

        let profile = if local_profile.exists() {
            match local_profile.read() {
                Ok(ok) => ok,
                Err(err) => panic!("{err}"),
            }
        } else {
            match local_profile.create() {
                Ok(ok) => ok,
                Err(err) => panic!("{err}"),
            }
        };

        for line in profile.split("\n") {
            if let Some(node) = parse(line) {
                println!("{:?}", node);
            }
        }
    }

    fn update_prompt(&mut self) {
        self.prompt = match self.variable.get("NSH_PROMPT") {
            Some(string) => prompt::decode(&string),
            None => "#".to_string(),
        };
    }

    pub fn repl(&mut self) {
        loop {
            self.rep();
        }
    }

    fn rep(&mut self) {
        self.update_prompt();

        if let Some(string) = self.read_line() {
            if let Some(node) = parse(&string) {
                self.eval(node);
            }
        }
    }

    fn read_line(&mut self) -> Option<String> {
        let mut buffer = Vec::new();

        let mut buffer_index = 0;

        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        set_raw_mode(&mut self.termios);

        if buffer_index <= buffer.len() {
            let move_position = self.prompt.len() + 1;
            stdout
                .write_all(
                    format!(
                        "\r{}{}",
                        self.prompt,
                        ansi::Cursor::Move(move_position).get_esc_code()
                    )
                    .as_bytes(),
                )
                .unwrap();
        }

        loop {
            if let Some(code) = getch() {
                match code {
                    [0] => continue,
                    [3] => {
                        unset_raw_mode(&mut self.termios);

                        process::exit(0)
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
                                if buffer_index < buffer.len() {
                                    buffer_index += 1;
                                    stdout
                                        .write_all(
                                            format!("{}", ansi::Cursor::Right.get_esc_code())
                                                .as_bytes(),
                                        )
                                        .unwrap();
                                }
                            }
                            //left
                            [68] => {
                                if buffer_index > 0 {
                                    stdout
                                        .write_all(
                                            format!("{}", ansi::Cursor::Left.get_esc_code())
                                                .as_bytes(),
                                        )
                                        .unwrap();
                                    buffer_index -= 1;
                                }
                            }
                            _ => continue,
                        }
                    }
                    [127] => {
                        if buffer_index <= 0 {
                            continue;
                        }

                        buffer_index -= 1;

                        for i in 0..buffer.len() {
                            if i != 0 {
                                stdout
                                    .write_all(
                                        format!("{}", ansi::Cursor::Backspace.get_esc_code())
                                            .as_bytes(),
                                    )
                                    .unwrap()
                            }
                        }

                        stdout
                            .write_all(
                                format!("\r{}{}", self.prompt, String::from_utf8_lossy(&buffer))
                                    .as_bytes(),
                            )
                            .unwrap();

                        buffer.remove(buffer_index);

                        stdout
                            .write_all(
                                format!("{}", ansi::Cursor::Backspace.get_esc_code()).as_bytes(),
                            )
                            .unwrap();
                        stdout
                            .write_all(
                                format!(
                                    "\r{}{}",
                                    self.prompt,
                                    String::from_utf8_lossy(&buffer).to_string()
                                )
                                .as_bytes(),
                            )
                            .unwrap();

                        if buffer_index < buffer.len() {
                            let move_position = self.prompt.len() + buffer_index - 1;
                            stdout
                                .write_all(
                                    format!("{}", ansi::Cursor::Move(move_position).get_esc_code())
                                        .as_bytes(),
                                )
                                .unwrap();
                        }
                    }
                    _ => {
                        let code = match code.get(0) {
                            Some(code) => *code,
                            None => continue,
                        };

                        buffer.insert(buffer_index, code);
                        buffer_index += 1;
                        for i in 0..buffer.len() {
                            if i != 0 {
                                stdout
                                    .write_all(
                                        format!("{}", ansi::Cursor::Backspace.get_esc_code())
                                            .as_bytes(),
                                    )
                                    .unwrap();
                            }
                        }

                        stdout
                            .write_all(
                                format!("\r{}{}", self.prompt, String::from_utf8_lossy(&buffer))
                                    .as_bytes(),
                            )
                            .unwrap();

                        if buffer_index < buffer.len() {
                            let move_position = self.prompt.len() + buffer_index;

                            stdout
                                .write_all(
                                    format!("{}", ansi::Cursor::Move(move_position).get_esc_code())
                                        .as_bytes(),
                                )
                                .unwrap();
                        }
                    }
                }
            }

            stdout.flush().unwrap_or_default();
        }

        unset_raw_mode(&mut self.termios);

        stdout.write(b"\n").unwrap();

        if buffer.len() != 0 {
            Some(String::from_utf8_lossy(&buffer).to_string())
        } else {
            None
        }
    }

    fn eval(&mut self, node: Node) {
        match node {
            Node::Pipe(mut pipe) => {
                if let Some(left) = pipe.left() {
                    if matches!(*left, Node::Pipe(_)) {
                        self.eval(*left);
                    } else {
                        match *left {
                            Node::Command(command) => {}
                            _ => {}
                        }
                    }
                }

                if let Some(right) = pipe.right() {
                    if matches!(*right, Node::Pipe(_)) {
                        self.eval(*right);
                    } else {
                        match *right {
                            Node::Command(command) => {}
                            _ => {}
                        }
                    }
                }
            }

            Node::Command(command) => {
                println!("{:?}", self.expand_command_node(command));
            }

            Node::VInsert(mut vinsert) => {
                let key = match vinsert.key() {
                    Some(key) => match *key {
                        Node::String(string) => string,
                        _ => return,
                    },
                    None => return,
                };

                let val = match vinsert.val() {
                    Some(val) => match *val {
                        Node::String(string) => string,
                        _ => return,
                    },
                    None => return,
                };

                self.variable.insert(&key, &val);
            }

            _ => {}
        }
    }

    fn expand_command_node(
        &mut self,
        mut command: Command,
    ) -> Option<(
        String,
        Vec<String>,
        (Option<String>, Option<String>, Option<String>),
        bool,
    )> {
        let (mut program, mut args, mut redirect, mut background): (
            String,
            Vec<String>,
            (Option<String>, Option<String>, Option<String>),
            bool,
        ) = (String::new(), Vec::new(), (None, None, None), false);

        if let Some(prefix) = command.prefix() {
            match *prefix {
                Node::String(string) => {
                    program = string;
                }
                Node::VReference(key) => {
                    program = self.variable.get(&key).unwrap_or_default();
                }
                _ => return None,
            }
        } else {
            return None;
        }

        if let Some(mut suffix) = command.suffix() {
            while let Some(node) = suffix.pop() {
                match node {
                    Node::String(string) => args.push(string),
                    Node::VReference(key) => {
                        if let Some(val) = self.variable.get(&key) {
                            args.push(val);
                        }
                    }
                    Node::Redirect(rd) => match rd.file().as_ref() {
                        Node::String(string) => match rd.fd() {
                            0 => redirect.0 = Some(string.to_string()),
                            1 => redirect.1 = Some(string.to_string()),
                            2 => redirect.2 = Some(string.to_string()),
                            _ => continue,
                        },
                        _ => continue,
                    },
                    Node::Background(_) => background = true,
                    _ => {}
                }
            }
        }

        args.reverse();

        Some((program, args, redirect, background))
    }
}

fn parse(source: &str) -> Option<Node> {
    let source = source.replace("~", &env::var("HOME").unwrap_or_default());

    match Parser::new(Lexer::new(source.chars().collect())).parse() {
        Ok(ok) => ok,
        Err(err) => panic!("{:?}", err),
    }
}

struct Profile(ProfileKind);

impl Profile {
    fn new(profile_type: ProfileKind) -> Self {
        Self { 0: profile_type }
    }

    fn exists(&self) -> bool {
        if let Ok(path) = self.lookup() {
            return path.exists();
        }

        false
    }

    fn create(&self) -> io::Result<String> {
        const DEFAULT_VALUE: &str = "NSH_PROMPT = \\w#";

        let path = self.lookup()?;

        fs::File::create(path)?.write_all(DEFAULT_VALUE.as_bytes())?;
        Ok(DEFAULT_VALUE.to_string())
    }

    fn read(&self) -> io::Result<String> {
        let path = self.lookup()?;

        let mut buffer = Vec::new();

        fs::File::open(path)?.read_to_end(&mut buffer)?;

        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    fn lookup(&self) -> io::Result<PathBuf> {
        match self.0 {
            ProfileKind::Local => match env::var("HOME") {
                Ok(val) => {
                    let mut path = PathBuf::from(val);
                    path.push(".nsh_profile");
                    Ok(path)
                }
                Err(err) => panic!("{err}"),
            },
        }
    }
}

enum ProfileKind {
    Local,
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
        libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK);
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
