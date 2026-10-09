use std::fs;
use std::env;
use crate::checkout;

pub fn create (branch_name: &String) {
    let cwd = env::current_dir().unwrap();
    /* find out which branch we are currently on by reading HEAD */
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    let current_branch_ref = fs::read_to_string(&head_path).unwrap();
    /* extract the relative path to the current branch's file (e.g., "refs/heads/main") */
    let (_, current_branch_ref_path) = current_branch_ref.split_once(':').unwrap();
    /* read the latest commit hash from that current branch */
    let current_branch_ref_full_path = format!("{}/.xip/{}", cwd.display(), current_branch_ref_path.trim());
    /* create the new branch file and initialize it with that same commit hash */
    let branch_file_path = format!("{}/.xip/refs/heads/{}", cwd.display(), branch_name);
    /* what if branch already exists? */
    if fs::exists(&branch_file_path).unwrap() {
        println!("'{}' exists, nothing to do", branch_name);
        return;
    }
    /* check if previous branch exists and contains a commit */
    if let Ok(current_branch_commit_hash) = fs::read_to_string(&current_branch_ref_full_path) {
        fs::write(branch_file_path, current_branch_commit_hash).unwrap();
        println!("'{}' created", branch_name);
    } else {
        println!("new repo, no prior commits");
    }
}

pub fn switch (branch_name: &String) {
    /* checkout a branch */ 
    checkout::checkout(&branch_name);
    println!("switched to '{}', do your thing", branch_name);
}

pub fn ls () {
    let cwd = env::current_dir().unwrap();
    let ref_dir_path = format!("{}/.xip/refs/heads", cwd.display());
    let branches = fs::read_dir(ref_dir_path).expect("Error opening the refs dir");
    /* get current branch name */
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    let head_content = fs::read_to_string(&head_path).unwrap();
    let (_, ref_path) = head_content.split_once(':').unwrap();
    let (_, current) = ref_path.trim().rsplit_once('/').unwrap();
    /* read dir returns an iterator of Result<DirEntry> objects */
    for branch in branches {
        let entry = branch.unwrap();
        let branch_name = entry.file_name().into_string().unwrap();
        if branch_name == current {
            println!("{} (current)", branch_name);
        } else {
            println!("{}", branch_name);
        }
    }
}
