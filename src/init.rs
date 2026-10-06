use std::fs;

pub fn init_repo () {
    fs::create_dir_all(".xip/objects").expect("Unable to init objects dir");
    fs::create_dir_all(".xip/refs/heads/").expect("Unable to init refs dir"); 
    fs::write(".xip/HEAD", "ref: refs/heads/main").expect("Unable to create main branch ref");
}
