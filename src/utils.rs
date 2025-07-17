// a function auto append Cargo.toml to the path if it is a directory, else return the path as is

use std::borrow::Cow;
use std::path::Path;

pub fn ensure_cargo_toml_path(path_str: &str) -> Cow<str> {
    let path = Path::new(path_str);
    if path.is_dir() {
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            Cow::Owned(cargo_toml.to_string_lossy().to_string())
        } else {
            Cow::Borrowed(path_str)
        }
    } else {
        Cow::Borrowed(path_str)
    }
}
