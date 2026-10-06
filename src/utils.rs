use std::collections::HashMap;
use std::fs;
use std::env;

use crate::Index;
use crate::Status;

/* check path: if it's a file name just skip,
    * else traverse the directory recursively and 
    * add files not present in the index */
pub fn process_path<T>(file_path: String, untracked: &mut Vec<String>, index_map: &HashMap<String, T>) {
    let meta = fs::symlink_metadata(&file_path).expect("Error reading metadata");
    if meta.is_dir() {
        let dir_entries = fs::read_dir(file_path.clone())
        .expect("Error opening dir");
        for dir_entry in dir_entries {
            let entry = dir_entry.expect("Error reading directory entry");
            let child_path = entry.path().display().to_string();
            if child_path.contains(".xip") || child_path.contains(".git") {
                continue;
            }
            process_path(child_path, untracked, index_map);
        }
    } else {
        let prefix = format!("{}/", env::current_dir().unwrap().display());
        let relative_path = file_path.strip_prefix(&prefix).unwrap_or(&file_path);
        let trimmed = relative_path.trim_start_matches("./").to_string();

        if !index_map.contains_key(&trimmed) {
            untracked.push(trimmed);
        }
    }
}

pub fn get_modified_files(index: &Index) -> Vec<String> {
    index.entries.iter().filter_map(|entry| {
        if let Ok(meta) = fs::symlink_metadata(&entry.path) {
            if Status::is_modified(&entry, &meta) {
                return Some(entry.path.clone());
            } 
        }
        None
    }).collect()
}


