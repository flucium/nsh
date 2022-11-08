use std::io::stderr;
use std::io::Write;

fn main() {
    match nsh::shell::Shell::new().initialize() {
        Ok(ok) => ok.repl(),
        Err(err) => stderr()
            .lock()
            .write_all(format!("{err}").as_bytes())
            .unwrap(),
    }
}
