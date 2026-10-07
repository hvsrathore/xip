use crate::Index;
use crate::IndexEntry;
use crate::utils::process_path;
use crate::object;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::fs::File;
use memmap2::Mmap;

pub struct Status {
    pub untracked: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>
}

impl Status {
    pub fn get_dir_status (dir_name: &str, index: &Index) -> Status {
        let mut modified = Vec::new();
        let mut deleted = Vec::new();
        /* using a HashMap to do fast lookups */
        let mut index_map = HashMap::new();
        for entry in &index.entries {
            match fs::symlink_metadata(&entry.path) {
                Ok(meta) => {
                    if Self::is_modified(&entry, &meta) {
                        modified.push(entry.path.clone());
                    }
                }, 
                Err(_) => {
                    deleted.push(entry.path.clone());
                }
            }
            index_map.insert(entry.path.clone(), entry);
        }
        /* now for new files */
        let cwd = env::current_dir().unwrap();
        let mut untracked: Vec<String> = Vec::new();
        process_path((&cwd.display()).to_string(), &mut untracked, &index_map);
        Status { untracked, modified, deleted }
    }

    pub fn print_status (status: &Status) {
        /* print the lists */
        let mut did_some = false;
        if status.modified.len() > 0 {
            println!("modified:");
            for fname in status.modified.iter() {
                println!("\t{}", fname);
            }
            println!("\n");
            did_some = true;
        }

        if status.untracked.len() > 0 {
            println!("untracked:");
            for fname in status.untracked.iter() {
                println!("\t{}", fname);
            }
            did_some = true;
            println!("\n");
        }

        if status.deleted.len() > 0 {
            println!("deleted:");
            for fname in status.deleted.iter() {
                println!("\t{}", fname);
            }
            did_some = true;
        }

        if !did_some {
            println!("nothing changed");
        }
    }

    pub fn is_modified (index_entry: &IndexEntry, meta: &Metadata) -> bool {
        if meta.len() as u32 != index_entry.file_size || meta.mtime() as u32 != index_entry.mtime_sec {
            /* compare hashes here */
            match File::open(&index_entry.path) {
                Ok(file) => {
                    let file_content = unsafe { Mmap::map(&file).expect("Failed to map file") };
                    let blob = object::Blob::format_blob(&file_content);
                    let calculated_hash = object::Blob::hash_blob(&blob);
                    if calculated_hash != index_entry.hash {
                        return true;
                    }
                }, 
                Err(_) => {}
            }
        }
        false
    }
}
