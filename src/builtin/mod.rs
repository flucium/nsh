use crate::variable::Variable;
use std::path::Path;
use std::io;
use std::env;

pub fn set(variable: &mut Variable, key: String, val: String) {
    variable.insert(key, val)
}

pub fn unset(variable: &mut Variable, key: String) {
    variable.remove(key)
}

pub fn cd(string: &str) -> io::Result<()> {
    let path = Path::new(string);

    env::set_current_dir(&path)?;

    env::set_var("PWD", path);

    Ok(())
}

// pub fn exit(code: i32) -> ! {
//     process::exit(code)
// }
