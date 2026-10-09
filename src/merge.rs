use std::fs;
use std::env;

pub fn merge (branch_name: &String) {
    let cwd = env::current_dir().unwrap();
    let head_path = format!("{}/.xip/HEAD", cwd.display());
    let head_content = fs::read_to_string(&head_path).unwrap();
    let (_, ref_path) = head_content.split_once(':').unwrap();
    let (_, current) = ref_path.trim().rsplit_once('/').unwrap();
}
