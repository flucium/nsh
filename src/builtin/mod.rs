use std::env;
use std::io;
use std::path::Path;
use std::process;

// pub type Command = ();

pub fn exit(code: i32) {
    process::exit(code)
}

pub fn abort() {
    process::abort()
}

pub fn cd(string: String) -> io::Result<()> {
    let path = Path::new(&string);

    env::set_current_dir(&path)?;

    env::set_var("PWD", path);

    Ok(())
}

pub mod crypto {
    use sha1::Digest as Sha1Digest;
    use sha1::Sha1;
    pub fn sha1(string: String) ->Vec<u8>{
        let mut hash = Sha1::new();
        
        hash.update(string);
        
        hash.finalize().to_vec()
    }
}
