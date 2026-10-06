use std::collections::HashMap;
use crate::checkout;
use crate::commit;

struct Commit {
    tree: String,
    parent1: String,
    parent2: String,
    author: String,
    author_email: String,
    timestamp: u64,
    timezone_offset: i16,
    message: String
}

pub struct Log {
    commit_map: HashMap<String, Commit>
}

impl Log {
    /* parse the contents of the commit object
        * file and return a commit struct */
    fn parse_commit (commit: &String) -> Commit {
        /* open commit object file, decompress it and
        * get it's contents as a string */
    }

    /* call parse_commit iteratively and build the hashmap */
    fn build_cmap () -> HashMap<String, Commit> {
        let cmap = HashMap::new();
        let head = commit::get_parent();
        /* TODO: move read_object into utils */
        let latest_commit_contents = checkout::read_object(&head);
    }

    /* iterate through HashMap and print commits */
    pub fn get_log () -> String {
    
    }
}
