use std::process;
use std::env;
use std::io;
use std::path::Path;

pub fn exit(code: i32) {
    process::exit(code)
}

pub fn abort(){
    process::abort()
}

pub fn cd(string: String) -> io::Result<()> {
    let path = Path::new(&string);

    env::set_current_dir(&path)?;

    env::set_var("PWD", path);

    Ok(())
}