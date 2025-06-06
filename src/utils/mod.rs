use std::{fs, iter::once, path::PathBuf, process::Command};

pub fn split_by_delimiter<'a>(text: &'a str, delimiter: &'a str) -> Vec<&'a str> {
    text.split_terminator(delimiter)
        .flat_map(|part| {
            once(part).chain(once(delimiter)).take({
                let last_element = text.split(delimiter).last().unwrap();
                if part == last_element { 1 } else { 2 }
            })
        })
        .collect::<Vec<_>>()
}
pub fn load_dir_content(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    fs::read_dir(path)?
        .map(|e| {
            let entry = e?;

            if entry.path().is_file() {
                Ok(entry.path())
            } else {
                Ok(PathBuf::new())
            }
        })
        .filter(|x| x.as_ref().is_ok_and(|e| *e != PathBuf::new()))
        .collect::<anyhow::Result<Vec<_>>>()
}
pub async fn read_journal(path: PathBuf) -> String {
    let file_content = Command::new("journalctl")
        .arg("--file")
        .arg(path.as_os_str())
        .output();
    match file_content {
        Ok(content) => String::from_utf8(content.stdout).unwrap_or("Could not read stdout".into()),
        Err(e) => format!("Error occurred during loading {}", e),
    }
}

pub fn path_vec_to_string_vec(paths: &Vec<PathBuf>) -> Vec<String> {
    paths
        .iter()
        .map(|e| e.to_string_lossy().to_string())
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use crate::utils::split_by_delimiter;

    #[test]
    fn get_separated_string() {
        let result = split_by_delimiter("HelloRustWorld", "Rust");
        assert_eq!(result, vec!["Hello", "Rust", "World"]);
    }

    #[test]
    fn get_separated_string_by_matching_long() {
        let test_string = "HelloRustWorldKek";

        let result = split_by_delimiter(test_string, "Rust");
        assert_eq!(result, vec!["Hello", "Rust", "WorldKek"]);
    }
    #[test]
    fn get_separated_string_by_matching_long_with_whitespace() {
        let test_string = "Hello Rust World Kek";

        let result = split_by_delimiter(test_string, "Rust");
        assert_eq!(result, vec!["Hello ", "Rust", " World Kek"]);
    }
    #[test]
    fn get_separated_string_by_matching_multiple() {
        let test_string = "HelloRustWorldKekRust";

        let result = split_by_delimiter(test_string, "Rust");
        assert_eq!(result, vec!["Hello", "Rust", "WorldKek", "Rust"]);
    }
}
