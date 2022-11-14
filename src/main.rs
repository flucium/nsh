// use clap::Arg;
// use clap::ArgAction;
// use clap::Command;
use std::io::stderr;
use std::io::Write;

// fn clap_app() -> Command {
//     Command::new(nsh::manifest::name())
//         .version(nsh::manifest::version())
//         .author(nsh::manifest::author())
//         .arg(
//             Arg::new("profile")
//                 .long("profile")
//                 .short('p')
//                 .action(ArgAction::Set)
//                 .required(false),
//         )
// }

fn main() {
    match nsh::shell::Shell::new().initialize() {
        Ok(ok) => ok.repl(),
        Err(err) => stderr()
            .lock()
            .write_all(format!("{err}").as_bytes())
            .unwrap(),
    }
}
