use crate::index::Index;
use crate::index_entry::IndexEntry;
use std::collections::HashMap;
use sha1::{Sha1, Digest};
use std::fs;
use std::env;
use hex::encode;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;


enum TreeEntry {
    Blob(String),
    Tree(HashMap<String, TreeEntry>)
}

/* parses the index, builds and returns a recursive HashMap */
fn build_tree_hmap (index: &Index) -> TreeEntry {
    /* create the the root tree node */
    let mut root = TreeEntry::Tree(HashMap::new());
    /* loop through the flat list of staged files */
    for entry in &index.entries {
        /* break "linux/arch/alpha/boot/tools/mkbb.c" 
         * into ["linux", "arch", "alpha", "boot", "tools", "mkbb.c"] */
        let mut pieces_vec: Vec<&str> = entry.path.split('/').collect();
        let mut pieces = pieces_vec.as_slice();
        /* current acts as the cursor/pointer */
        let mut current = &mut root;
        let hash: String = hex::encode(entry.hash);
        while true {
            if let TreeEntry::Tree(current_map) = current {
                if pieces.len() == 1 {
                    /* BASE CASE: we're holding the final file: insert the Blob here */
                    let file_name = pieces[0].to_string();
                    current_map.insert(file_name, TreeEntry::Blob(hash));
                    /* end the loop and move on to the next index entry */
                    break;
                } else {
                    /* we're holding a directory: find/create the directory */
                    let dir_name = pieces[0].to_string();
                    let next = current_map.entry(dir_name)
                        .or_insert(TreeEntry::Tree(HashMap::new()));
                    /* or_insert() returns a mutable reference: next is already a pointer */
                    /* go one-level deep */
                    current = next;
                    /* move forward with pieces */
                    pieces = &pieces[1..];
                }
            }
        }
    }
    root
}

fn write_tree_object (hash: &String, tree_contents: &String) {
    let cwd = env::current_dir().unwrap();
    let tree_object_path = format!("{}/.xip/objects/{}", cwd.display(), &hash[0..]);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(tree_contents.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();
    fs::write(tree_object_path, compressed).unwrap();
}

fn hash_tree (root_map: &HashMap<String, TreeEntry>) -> (String, String) {
    /* every time we iterate over a a HashMap it returns randomly
     * ordered file/directory names which can generate wrong hashes 
     * so, traverse the hashmap (tree) and create a list of [names] 
     * all the files and directories and sort this list */
    let mut file_list: Vec<(&String, &TreeEntry)> = root_map.into_iter().collect();
        file_list.sort_by(|a, b| a.0.cmp(&b.0));
    let mut tree_content = String::new();
    for (fname, tree_entry) in file_list {
        /* Root (HashMap)
        └── 📁 "linux" (TreeEntry::Tree)
            └── 📁 "arch" (TreeEntry::Tree)
                └── 📁 "alpha" (TreeEntry::Tree)
                    └── 📁 "boot" (TreeEntry::Tree)
                        └── 📁 "tools" (TreeEntry::Tree)
                            └── 📄 "mkbb.c" (TreeEntry::Blob containing the file's hash) */
        match tree_entry {
            /* match got "linux", pauses and dives into the 
             * child directory - all the way down the chain to "mkbb.c" */
            TreeEntry::Blob(hash) => {
                let line = format!("blob {} {}\n", fname, hash);
                tree_content.push_str(&line);
            },
            TreeEntry::Tree(map) => {
                /* works bottom-up: the deepest layers 
                 * must finish before the higher layers can resume */
                let (sub_dir_hash, sub_dir_contents) = hash_tree(map);
                let line = format!("tree {} {}\n", fname, sub_dir_hash);
                tree_content.push_str(&line);
                write_tree_object(&sub_dir_hash, &sub_dir_contents);
            }
        }
    }
    let mut hasher = Sha1::new();
    hasher.update(&tree_content);
    (hex::encode(hasher.finalize()), tree_content)
}

/* writes the root tree object */
pub fn write_tree (index: &Index) -> String {
    let root = build_tree_hmap(index);
    let mut root_hash: String = String::new();
    let mut root_contents: String = String::new();
    if let TreeEntry::Tree(ref root_map) = root {
        (root_hash, root_contents) = hash_tree(root_map);
    }
    write_tree_object(&root_hash, &root_contents);
    root_hash
}
