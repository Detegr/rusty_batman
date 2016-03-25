use std::env;
use std::path::Path;

pub fn exists_in_path(program: &str) -> bool {
    let path = env::var_os("PATH")
                    .map_or("".to_owned(),
                       |val| val.into_string().unwrap_or("".to_owned()));
    for part in path.split(':') {
        if Path::exists(Path::new(&format!("{}/{}", part, program))) {
            return true
        }
    }
    false
}
