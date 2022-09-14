use crate::manifest::{name,version};
use std::env;

// pub type Prompt = String;

//current user name
const USER_NAME: &str = "\\u";

//shell name
const SHELL_NAME: &str = "\\s";

//version of the shell, you are using
const SHELL_VERSION: &str = "\\v";

//current working directory
const CURRENT_DIRECTORY: &str = "\\w";

//current working directory, full path
const CURRENT_F_DIRECTORY: &str = "\\W";

//host machine name
// const HOST_NAME: &str = "\\h";
pub fn decode(source: String) ->String {
    let mut buffer = source.to_string();

    if source.contains(USER_NAME) {
        buffer = buffer.replace(USER_NAME, &get_user_name().expect(""));
    }

    if source.contains(SHELL_NAME) {
        buffer = buffer.replace(SHELL_NAME, name());
    }

    if source.contains(SHELL_VERSION) {
        buffer = buffer.replace(SHELL_VERSION, version());
    }

    if source.contains(CURRENT_DIRECTORY) {
        let full_path = env::current_dir().unwrap_or_default();

        let name = full_path.file_name().unwrap_or_default();

        buffer = buffer.replace(CURRENT_DIRECTORY, &name.to_string_lossy());
    }

    if source.contains(CURRENT_F_DIRECTORY) {
        let full_path = env::current_dir().unwrap_or_default();

        buffer = buffer.replace(CURRENT_F_DIRECTORY, &full_path.to_string_lossy());
    }

    buffer
}

fn get_user_name() -> Result<String, env::VarError> {
    let string = env::var("USER")?;

    Ok(string)
}