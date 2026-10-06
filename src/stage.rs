use crate::utils;
use crate::status::Status;
use crate::Index;
use crate::IndexEntry;
use crate::object::Blob;

use std::fs;
use std::env;
use std::collections::HashMap;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::fs::File;
use rayon::prelude::*;

fn add_dir (dir_path: &str, index: &mut Index) {
    /* using a HashMap to do fast lookups */
    let mut index_map = HashMap::new();
    /* get the list of modified, untracked 
     * and deleted files safely 
     * before we lock the index for writing */
    let status = Status::get_status(&index);
    for entry in &mut index.entries {
        index_map.insert(entry.path.clone(), entry);
    }
    /* modified files */
    if status.modified.len() > 0 {
        for file_path in &status.modified {
            let blob_hash = Blob::add_blob(file_path.clone());
            if let Some(index_entry) = index_map.get_mut(file_path) {
                let meta = fs::symlink_metadata(&file_path).expect("Error reading metadata");
                index_entry.ctime_sec = meta.ctime() as u32;
                index_entry.ctime_nsec = meta.ctime_nsec() as u32;
                index_entry.mtime_sec = meta.mtime() as u32;
                index_entry.mtime_nsec = meta.mtime_nsec() as u32;
                index_entry.file_size = meta.len() as u32;
                index_entry.hash = blob_hash;
                println!("update '{}'", file_path);
            }
        }
    }
    /* untracked (new) files */
    let entries: Vec<IndexEntry> = status.untracked.into_par_iter().map(|file_path| {
        let blob_hash = Blob::add_blob(file_path.clone());
        let meta = fs::symlink_metadata(&file_path).expect("Error reading metadata");
        IndexEntry::new(meta, blob_hash, file_path)
    }).collect();
    for entry in &entries {
        println!("add '{}'", entry.path);
    }
    /* deleted files */
    index.entries.retain(|entry| {
        if status.deleted.contains(&entry.path) {
            println!("remove '{}'", &entry.path);
            return false;
        }
        return true;
    });
    /* finally, write index back to disk */
    index.entries.extend(entries);
    index.header.count = index.entries.len() as u32;
}

fn add_single_file (file_path: String, index: &mut Index) {
    /* check if the file exists */
    if let Ok(true) = fs::exists(&file_path) {
        /* TODO: check if an index entry exists for this file */
        /* ... */
        let blob_hash = Blob::add_blob(file_path.clone());
    } else {
        println!("no such file");
        return;
    }
}
pub fn add_to_index (file_path: String, index: &mut Index) {
    /* check if user gave a directory name or a file name */
    let meta = fs::symlink_metadata(&file_path).expect("Error reading metadata");
    if meta.is_dir() {
        add_dir(&file_path, index);
    } else {
        add_single_file(&file_path, index);
    }
    let index_path = format!("{}/.xip/index", env::current_dir().unwrap().display());
    fs::write(index_path, index.to_bytes())
        .expect("Failed to write the updated index");
}
