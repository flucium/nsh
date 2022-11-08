use crate::error::*;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
pub fn get_path() -> Result<PathBuf> {
    match env::var("HOME").or(env::var("USER")) {
        Ok(val) => {
            let mut path = PathBuf::from(val);
            path.push(".nsh_profile");
            Ok(path)
        }
        Err(err) => Err(Error::new(ErrorKind::NotFound, err.to_string())),
    }
}

pub fn create() -> Result<()> {
    let path = get_path()?;
    if let Err(err) = File::create(path) {
        return Err(Error::new(ErrorKind::CreateFailed, err.to_string()));
    }
    Ok(())
}

pub fn read() -> Result<String> {
    let path = get_path()?;

    match File::open(path) {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap();
            Ok(buffer)
        }
        Err(err) => Err(Error::new(ErrorKind::OpenFailed, err.to_string())),
    }
}
