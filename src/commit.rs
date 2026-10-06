use crate::Index;
use crate::tree;
use std::time::{SystemTime, UNIX_EPOCH};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
use sha1::{Sha1, Digest};
use std::fs;
use std::env;

/* hash commit contents */
fn hash_commit (contents: &String) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&contents);
    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);
    hash_str
}

/* compress commit contents */
fn compress_commit (contents: &String) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(contents.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    compressed
}

/* create commit object path and write the compressed commit */
fn write_commit (commit_hash: &String, compressed: Vec<u8>) {
    let cwd = env::current_dir().unwrap();
    let commit_object_path = format!("{}/.xip/objects/{}", cwd.display(), &commit_hash[0..]);
    fs::write(commit_object_path, compressed).unwrap();
}

/* point to the latest commit: detached state */
pub fn update_head (commit_hash: &str) {
    let cwd = env::current_dir().unwrap();
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    fs::write(head_path, commit_hash).unwrap();
}

/* point to the latest commit: branched state */
fn update_branch_ref (commit_hash_str: &String) {
    let cwd = env::current_dir().unwrap();
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    let head_content = fs::read_to_string(&head_path).unwrap();
    let (_, branch_ref_path) = head_content.split_once(':').unwrap();
    let ref_path = format!("{}/.xip/{}", cwd.display(), branch_ref_path.trim());
    fs::write(ref_path, commit_hash_str).unwrap();
}

pub fn get_parent_from_HEAD () -> String {
    let cwd = env::current_dir().unwrap();
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    let head_content = fs::read_to_string(&head_path).unwrap();
    if head_content.len() > 0 {
        head_content
    } else {
        String::new()
    }
}

pub fn get_parent_from_ref () -> String {
    let cwd = env::current_dir().unwrap();
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    let head_content = fs::read_to_string(&head_path).unwrap();
    let (_, branch_ref_path) = head_content.split_once(':').unwrap();
    let ref_path = format!("{}/.xip/{}", cwd.display(), branch_ref_path.trim());
    /* read the latest commit hash from that current branch */
    /* but what if it's the very first commit */
    match fs::read_to_string(&ref_path) {
        Ok(parent_hash) => { parent_hash.clone() },
        Err(_) => { String::new() }
    }
}

fn get_state () -> String {
    let cwd = env::current_dir().unwrap();
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    let head_content = fs::read_to_string(&head_path).unwrap();
    if head_content.contains("ref: ") {
        String::from("branched")
    } else {
        String::from("detached")
    }
}

pub fn build_commit (index: &Index, message: &String) {
    let root_hash = tree::write_tree(index); 
    let state = get_state();
    let mut parent_hash = String::new();
    /* check the state (detached or branched) */
    match state.as_str() {
        "detached" => {
            /* if detached state: get parent hash from .xip/HEAD */
            parent_hash = get_parent_from_HEAD();
        }, 
        "branched" => {
            /* else get parent hash from the ref pointed by HEAD */
            parent_hash = get_parent_from_ref();
        },
        _ => {}
    }
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    let timestamp = since_the_epoch.as_secs();
    let mut commit_content = String::new();
    if parent_hash.len() > 0 {
        commit_content = format!("tree {}\nparent {}\nauthor HVSR hvsr@gmail.com {} +0000\ncommitter HVSR hvsr@gmail.com {} +0000\n\n{}",
            root_hash, parent_hash, timestamp, timestamp, message);
    } else {
        commit_content = format!("tree {}\nauthor HVSR hvsr@gmail.com {} +0000\ncommitter HVSR hvsr@gmail.com {} +0000\n\n{}",
            root_hash, timestamp, timestamp, message);

    }
    let commit_hash_str = hash_commit(&commit_content);
    let compressed = compress_commit(&commit_content);
    write_commit(&commit_hash_str, compressed);
    /* if detached state update HEAD, else update the branch ref */
    match state.as_str() {
        "detached" => {
            update_head(&commit_hash_str);
        }, 
        "branched" => {
            update_branch_ref(&commit_hash_str);
        }, 
        _ => {}
    }
    println!("{}", commit_content);
}
