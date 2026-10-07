use std::fs;
use std::env;
use flate2::read::ZlibDecoder;
use std::io::Read;
use memmap2::Mmap;
use std::fs::File;
use std::fs::Metadata;
use hex;
use std::os::unix::fs::MetadataExt;
use rayon::prelude::*;
use io_uring::{opcode, types, IoUring};
use std::os::unix::io::AsRawFd;
use std::collections::HashMap;

use crate::object;
use crate::commit;
use crate::index::Index;
use crate::IndexEntry;

pub fn read_object (hash: &str) -> String {
    let cwd = env::current_dir().unwrap();
    let path = format!("{}/.xip/objects/{}", cwd.display(), &hash[0..]);
    // println!("path: {:?}", path);
    let compressed = fs::read(&path).unwrap();
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();
    decompressed
}

fn read_blob (hash: &str) -> Vec<u8> {
    let cwd = env::current_dir().unwrap();
    let path = format!("{}/.xip/objects/{}/{}", cwd.display(), &hash[0..2], &hash[2..]);
    match File::open(&path) {
        Ok(file) => {
            let compressed = unsafe { Mmap::map(&file).expect("Failed to map blob") };
            let mut decoder = ZlibDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();
            decompressed
        }, 
        Err(_) => {
           panic!("Missing file: {:?}", path);
        }
    }
}

fn remove_first_two_words (blob_bytes: &Vec<u8>) -> &[u8] {
    let null_index = blob_bytes.iter().position(|&b| b == 0).unwrap();
    &blob_bytes[null_index + 1..]
}

fn create_dirs (tree: &str, path: &str, blob_files_to_write: &mut Vec<(String, String)>) {
    for line in tree.lines() {
        let (rest_of_the_line, hash) = line.rsplit_once(' ').unwrap();
        let (object_type, fname) = rest_of_the_line.split_once(' ').unwrap();
        let full_path = format!("{}/{}", path, fname);
        match object_type {
            "tree" => {
                fs::create_dir_all(&full_path).unwrap();
                let tree = read_object(hash);
                create_dirs(&tree, &full_path, blob_files_to_write);
            },
            "blob" => {
                blob_files_to_write.push((full_path, hash.to_string()));
            },
            _ => {}
        }
    }
}

fn write_blobs (blobs_to_write: &Vec<(String, String)>) -> Vec<(String, Metadata)> {
    blobs_to_write.into_par_iter().filter_map(|(full_path, hash)| {
        let blob_bytes = read_blob(&hash);
        fs::write(&full_path, remove_first_two_words(&blob_bytes)).unwrap();
        Some((full_path.clone(), fs::metadata(&full_path).unwrap()))
    }).collect() 
}

fn update_timestamps (updated_files: &Vec<(String, Metadata)>, index_map: &mut HashMap<String, &mut IndexEntry>) {
    for updated_entry in updated_files {
        /* IndexEntry contains relative path */
        let prefix = format!("{}/", env::current_dir().unwrap().display());
        let relative_path = updated_entry.0.strip_prefix(&prefix).unwrap_or(&updated_entry.0);
        if let Some(index_entry) = index_map.get_mut(&relative_path.to_string()) {
            index_entry.mtime_sec = updated_entry.1.mtime() as u32; 
            index_entry.mtime_nsec = updated_entry.1.mtime_nsec() as u32; 
        }
    }
}

/* filter blob_files_to_write here */
fn filter_blobs(blobs_to_write: &mut Vec<(String, String)>, index_map: &mut HashMap<String, &mut IndexEntry>) {
    /* instead of hashing each file, just check their timestamps and file size */
    blobs_to_write.retain(|(full_path, hash)| {
        if let Ok(meta) = fs::metadata(&full_path) {
            /* IndexEntry contains relative path */
            let prefix = format!("{}/", env::current_dir().unwrap().display());
            let relative_path = full_path.strip_prefix(&prefix).unwrap_or(&full_path);
            if let Some(staged_entry) = index_map.get(&relative_path.to_string()) {
                if meta.len() as u32 == staged_entry.file_size && meta.mtime() as u32 == staged_entry.mtime_sec {
                    return false;
                }
            }           
        }
        let prefix = format!("{}/", env::current_dir().unwrap().display());
        let relative_path = full_path.strip_prefix(&prefix).unwrap_or(&full_path);
        println!("reclaim: {}", relative_path);
        return true;
    });
}

fn build_working_dir (commit_hash: &str) {
    /* init objects */
    let commit = read_object(commit_hash);
    let tree = read_object(&commit[5..45]);
    let cwd = env::current_dir().unwrap();
    /* create a list of blobs to write back to disk */
    let mut blob_files_to_write: Vec<(String, String)> = Vec::new();
    create_dirs(&tree, &cwd.display().to_string(), &mut blob_files_to_write);
    /* for lookups */
    let mut index = Index::fetch_index();
    let mut index_map = HashMap::new();
    for entry in &mut index.entries {
        index_map.insert(entry.path.clone(), entry);
    }
    filter_blobs(&mut blob_files_to_write, &mut index_map);
    /* write the filtered blobs to disk */
    let updated_files = write_blobs(&blob_files_to_write); 
    /* update the index */
    update_timestamps(&updated_files, &mut index_map);
    let index_path = format!("{}/.xip/index", env::current_dir().unwrap().display());
    fs::write(index_path, index.to_bytes())
        .expect("Failed to write the updated index");
}
 
pub fn checkout (target: &str) {
    let cwd = env::current_dir().unwrap();
    let ref_path = format!("{}/.xip/refs/heads/{}", cwd.display(), target);
    /* figure out whether target is a commit hash or branch name */
    match fs::exists(&ref_path) {
        /* 'target' is inside '.xip/refs/heads/' */
        Ok(true) => {
            /* read the commit hash inside it and move on */
            let commit_hash = fs::read_to_string(&ref_path).unwrap();
            /* restore working directory based on that commit */
            build_working_dir(&commit_hash);
            /* update HEAD pointing to the new branch inside refs */
            let head_path = format!("{}/.xip/HEAD", cwd.display());
            fs::write(head_path, format!("ref: refs/heads/{}", &target)).unwrap();
        }, 
        /* target is a raw commit hash: make detached HEAD */
        Ok(false) => {
            /* restore working directory based on that commit */
            build_working_dir(target);
            /* update HEAD to that commit hash (Detached HEAD) */
            commit::update_head(&target);
        },
        Err(_) => {}
        /* in both cases: working directory is restored based on a commit hash  */
    }
}
