use std::env;

//DaMeDaYo->pub fn parse(source: String) -> String {
pub fn parse(source: &str) -> String {
    source
        .replace("\\W", &get_current_dir_path(true))
        .replace("\\w", &get_current_dir_path(false))
}

fn get_current_dir_path(is_full_path: bool) -> String {
    match env::current_dir() {
        Ok(path) => {
            if is_full_path == false {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                path.to_string_lossy().to_string()
            }
        }
        Err(_) => "/".to_owned(),
    }
}
