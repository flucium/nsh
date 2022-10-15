use crate::variable::Variable;
use std::env;
use std::io;
use std::path::Path;

pub fn set(variable: &mut Variable, key: String, val: String) {
    variable.insert(key, val)
}

pub fn unset(variable: &mut Variable, key: String) {
    variable.remove(key)
}

pub fn cd(string: String) -> io::Result<()> {
    let path = Path::new(&string);

    env::set_current_dir(&path)?;

    env::set_var("PWD", path);

    Ok(())
}
