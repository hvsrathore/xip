use std::collections::{HashSet, HashMap};
use std::env;
use crate::commit;

/* key: (base_tree_hash, our_tree_hash, their_tree_hash) */
pub type TreeCacheKey = (String, String, String);
/* final merge of a specific directory */
#[derive(Debug, Clone)]
pub struct CachedTreeMerge {
    merged_tree_hash: String, 
    has_conflict: bool
}
/*  the ORT cache itself */
pub type OrtCache = HashMap<TreeCacheKey, CachedTreeMerge>;

/* get_diff() returns a list of Hunks */
struct Hunk {
    /* the first line affected in the base file */
    base_start: usize,
    /* the last line affected in the base file */
    base_end: usize,
    /* the new text replacing those lines */
    new_text: String
}

/*  find the common ancestor by walking 'ours' parent hashes 
 *  into a HashSet for O(1) lookups, then walking 'theirs' 
 *  parents until we find a match */
fn resolve_base (ours: &String, theirs: &String) -> Option<String> {
    let mut ancestors: HashSet<String> = HashSet::new();
    /* walk backward from branch 'ours' */
    let mut parent = Some(ours.to_string());
    while let Some(parent_hash) = parent {
        ancestors.insert(parent_hash.clone());
        parent = commit::get_parent_from_commit(&parent_hash);
    }
    parent = Some(theirs.to_string());
    while let Some(parent_hash) = parent {
        if ancestors.contains(&parent_hash) {
            return Some(parent_hash);
        }
        parent = commit::get_parent_from_commit(&parent_hash);
    }
    /* a fallback in case the loop finishes without a match */
    None
}
/* merge file contents: apply three-way content merging where 
 * possible and mark unresolved conflicts */
fn merge_file_content (base_hash: Option<&String>, our_hash: Option<&String>, their_hash: Option<&String>) -> Option<String> {
    let mut base_text = String::new();
    let mut our_text = String::new();
    let mut their_text = String::new();
    /* TODO: helper */
    if let Some(hash) = base_hash { base_text = object::read_blob(hash); } 
    if let Some(hash) = our_hash { our_text = object::read_blob(hash); }    
    if let Some(hash) = their_hash { their_text = object::read_blob(hash); }
    /* TODO: helper */
    let our_changed_hunks = get_diff(&base_text, &our_text);
    let their_changed_hunks = get_diff(&base_text, &their_text);

    for our_hunk in &our_changed_hunks {
        for their_hunk in &their_changed_hunks {
            if (our_hunk.base_start < their_hunk.base_end) && (their_hunk.base_start < our_hunk.base_end) {
                return None;
            }         
        }
    }
    /* TODO: helper */
    let merged_text = apply_patches(&base_text, our_changed_hunks, their_changed_hunks);
    /* TODO: helper */
    let new_hash = object::write_blob(&merged_text);
    Some(new_hash)
}

/* the core logic of the ORT merge engine:
* 1. tree skipping (O(1) early exit if nothing changed)
* 2. memoization (O(1) cache lookups for previously merged directories)
* 3. 3-Way Hash Resolution (identifying additions, deletions, 
*    and conflicts without touching file contents) */
fn merge (base_tree: &String, our_tree: &String, their_tree: &String, cache: &mut OrtCache) -> CachedTreeMerge {
    /* case 1 (tree skipping): absolutely nothing changed inside 
     * this folder on either branch */
    if (base_tree == our_tree) && (base_tree == their_tree) {
       return CachedTreeMerge {
           merged_tree_hash: base_tree.clone(), 
           has_conflict: false
       };
    } 
    /* case 2 (memoization): one of the branches modified this directory,
     * but what if the merge is already performed? */
    let cache_key = (base_tree.clone(), our_tree.clone(), their_tree.clone());
    if let Some(cached) = cache.get(&cache_key) {
        return cached.clone(); 
    }
    /* case 3 (3-way hash resolution): identifying additions, deletions, 
     * and conflicts without touching file contents */

    /* a brand new directory state, and we 
     * actually have to compute the merge! */
    let mut base_file_map: HashMap<String, String> = HashMap::new();
    let mut our_file_map: HashMap<String, String> = HashMap::new();
    let mut their_file_map: HashMap<String, String> = HashMap::new();
    /* TODO helper: */
    tree::read_tree(&base_tree, &mut base_file_map);
    tree::read_tree(&our_tree, &mut our_file_map);
    tree::read_tree(&their_tree, &mut their_file_map);
    /* master list of every unique file path across 
     * all three branches, ensuring we don't miss files 
     * that were newly added in just one branch. */
    let mut file_set: HashSet<String> = HashSet::new();
    /* extract keys from maps and combine them into a single data structure */
    for file_map in [&base_file_map, &our_file_map, &their_file_map] {
        file_set.extend(file_map.keys().cloned());
    }
    /* evaluate what happened to each file one by one, 
     * to do this: loop through our file_set: for each 
     * filename in that loop, figure out if it was modified, 
     * added, deleted, or left alone by each branch. */
    let mut has_conflict = false;
    let mut merged_files: HashMap<String, String> = HashMap::new();
    /* transformation for the 3-way hash comparison 
     * loop, it evaluates all possibilities simultaneously 
     * and identifies conflicts: 
     *      handle renames, directory changes, 
     *      file/directory conflicts, and 
     *      competing changes to paths */
    for file in file_set {
        let base_hash = base_file_map.get(&file);
        let our_hash = our_file_map.get(&file);
        let their_hash = their_file_map.get(&file);

        let resolved_hash = if our_hash == their_hash {
            our_hash 
        } else if base_hash == our_hash {
            their_hash 
        } else if base_hash == their_hash {
            our_hash 
        } else {
            if let Some(merged_hash) = merge_file_content(&base_hash, &our_hash, &their_hash) {
                merged_hash
            } else {
                has_conflict = true;
                None
            }
        };

        if let Some(hash) = resolved_hash {
            merged_files.insert(file, hash.clone());
        }
    }
    /* construct and return CachedTreeMerge */
    /* write the new tree object and get it's unique hash */
    /* TODO helper: */
    let merged_tree_hash = tree::write_tree(&merged_files);
    let result = CachedTreeMerge { merged_tree_hash, has_conflict };
    /* save it to the cache */
    cache.insert(cache_key, result.clone());
    result
}

/* merge wrapper */
pub fn ort_merge (theirs: &String) {
    /* find the merge base: 
     *      determine the shared history 
     *      from which the two branches diverged */
    let cwd = env::current_dir().unwrap();
    let ref_path = format!("{}/.xip/refs/heads/{}", cwd.display(), theirs);
    /* get current branch parent hash */
    let our_commit_hash = commit::get_parent_from_ref(); 
    let mut base = String::new();
    let mut their_commit_hash = String::new(); 
    /* figure out whether 'theirs' is a valid branch name */
    match fs::exists(&ref_path) {
        /* 'theirs' is inside '.xip/refs/heads/' */
        Ok(true) => {
            their_commit_hash = fs::read_to_string(&ref_path)
                .unwrap().trim().to_string();
            base = match resolve_base(&our_commit_hash, &their_commit_hash) {
                Some(base_hash) => base_hash,
                None => {
                    println!("unrelated history, nothing to do");
                    return;
                }
            };
        }, 
        Ok(false) => {
            println!("not a branch");
            return;
        },
        Err(_) => {
            panic!("Error checking existence of branch");
        }
    }

    /* TODO: helper */
    let our_tree = tree::get_root_hash(&our_commit_hash);
    let their_tree = tree::get_root_hash(&their_commit_hash);
    let ort_cache = HashMap::new();
    let merged = merge(&base, &our_commit_hash, &their_commit_hash, &mut ort_cache);

    /* TODO: produce the result:  
     *      write the merged tree, update the index, 
     *      and let Git complete or pause the merge 
     *      depending on the result */

    /* create the merge commit with TWO parents */
    let commit_message = format!("merged branch '{}' into current", theirs);
    let new_commit_hash = commit::write_merge_commit(
        &merged.merged_tree_hash, 
        &our_commit_hash,          // Parent 1 (Ours)
        &their_commit_hash,        // Parent 2 (Theirs)
        &commit_message
    );

    /* Update the current branch pointer to the new merge commit
    (Assuming a helper to update HEAD/refs) */
    commit::update_ref("HEAD", &new_commit_hash);

    /* finally, physically update the files on the hard drive */
    checkout::checkout(&merged.merged_tree_hash);
}
}
