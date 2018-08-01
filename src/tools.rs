use std::path::Path;

//return true if path exists
pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}
