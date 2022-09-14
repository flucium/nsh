use crate::variable::Variable;
use std::env;
use std::io;
use std::path::Path;
use std::process;

pub fn exit(code: i32) -> ! {
    process::exit(code)
}

pub fn cd(string: &str) -> io::Result<()> {
    let path = Path::new(string);

    env::set_current_dir(&path)?;

    env::set_var("PWD", path);

    Ok(())
}

pub fn senv(v: &mut Variable) ->io::Result<String>{
    let mut buffer = String::new();

    for key in v.keys() {
        let key = &key;

        buffer.push_str(&format!("{} {}", key, v.get(key).unwrap_or_default()));

        buffer.push('\n');
    }

    Ok(buffer)
}
