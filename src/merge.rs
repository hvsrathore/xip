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
     * and identifies conflicts: */
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
            has_conflict = true;
            None 
        };

        if let Some(hash) = resolved_hash {
            merged_files.insert(file, hash.clone());
        }
    }
    /* construct and return CachedTreeMerge */
    /* write the new tree object and get it's unique hash */
    let merged_tree_hash = tree::write_tree(&merged_files);
    let result = CachedTreeMerge { merged_tree_hash, has_conflict };
    /* save it to the cache */
    cache.insert(cache_key, result.clone());
    result
}

pub fn ort_merge (theirs: &String) {
    /* TODO: find the merge base: 
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

    /* TODO: analyze both sides: 
     *      compare the base tree with each 
     *      branch's tree to identify additions, deletions, 
     *      renames, and modifications */

    let our_tree = tree::get_root_hash(&our_commit_hash);
    let their_tree = tree::get_root_hash(&their_commit_hash);

    /* TODO: reconcile file structure: 
     *      handle renames, directory changes, 
     *      file/directory conflicts, and 
     *      competing changes to paths */


    /* TODO: merge file contents:
     *      apply three-way content merging where 
     *      possible and mark unresolved conflicts */

    let merged = merge(&base, &our_commit_hash, &their_commit_hash);
    /* ... */

    /* TODO: produce the result:  
     *      write the merged tree, update the index, 
     *      and let Git complete or pause the merge 
     *      depending on the result */
}
