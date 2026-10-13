use std::collections::{HashSet, HashMap};
use std::env;
use crate::commit;

/* Key: (base_tree_hash, our_tree_hash, their_tree_hash) */
pub type TreeCacheKey = (String, String, String);
/* final merge of a specific directory */
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
    /* tree skipping: absolutely nothing changed inside 
     * this folder on either branch */
    if (base_tree == our_tree) && (base_tree == their_tree) {
       return CachedTreeMerge {
           merged_tree_hash: base_tree.clone(), 
           has_conflict: false
       };
    }
}

pub fn ort_merge (theirs: &String) {
    /* TODO: find the merge base: 
     *      determine the shared history 
     *      from which the two branches diverged */
    let cwd = env::current_dir().unwrap();
    let ref_path = format!("{}/.xip/refs/heads/{}", cwd.display(), theirs);
    /* get current branch parent hash */
    let ours = commit::get_parent_from_ref(); 
    let mut base = String::new();
    let mut their_commit_hash = String::new(); 
    /* figure out whether 'theirs' is a valid branch name */
    match fs::exists(&ref_path) {
        /* 'theirs' is inside '.xip/refs/heads/' */
        Ok(true) => {
            their_commit_hash = fs::read_to_string(&ref_path)
                .unwrap().trim().to_string();
            base = match resolve_base(&ours, &their_commit_hash) {
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

    let our_tree = tree::get_root_hash(&ours);
    let their_tree = tree::get_root_hash(&their_commit_hash);

    /* TODO: reconcile file structure: 
     *      handle renames, directory changes, 
     *      file/directory conflicts, and 
     *      competing changes to paths */


    /* TODO: merge file contents:
     *      apply three-way content merging where 
     *      possible and mark unresolved conflicts */

    let merged = merge(&base, &ours, &their_commit_hash);
    /* ... */

    /* TODO: produce the result:  
     *      write the merged tree, update the index, 
     *      and let Git complete or pause the merge 
     *      depending on the result */
}
