use glob::glob;

use crate::configuration::CHECKS_DIR;


/// Produce list of dirs/files matching given glob pattern:
pub fn produce_list(glob_pattern: &str) -> Vec<String> {
    let mut list = vec!();
    for entry in glob(&glob_pattern).unwrap() {
        match entry {
            Ok(path) => {
                match path.file_name() {
                    Some(element) => {
                        element
                            .to_str()
                            .and_then(|elem| {
                                list.push(elem.to_string());
                                Some(elem.to_string())
                            });
                    },
                    None => (),
                }
            },
            Err(err) => {
                error!("Error: produce_list(): {}", err);
            },
        }
    }
    debug!("produce_list(): Elements: {:?}", list);
    list
}


/// List all check files found in default checks dir
pub fn list_check_files() -> Vec<String> {
    let glob_pattern = format!("{}/*.json", CHECKS_DIR);
    debug!("list_check_files(): {}", glob_pattern);
    produce_list(&glob_pattern)
}
