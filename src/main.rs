/* custom git like version 
 * control, written in Rust */

mod index;
mod index_header;
mod index_entry;
mod object;
mod tree;
mod commit;
mod checkout;
mod stage;
mod status;
mod utils;
mod branch;
mod init;
// mod log;
use crate::index::Index;
use crate::index_entry::IndexEntry;
use crate::object::Blob;
use crate::status::Status;
// use crate::log::Log;

use std::fs;
use std::env;
use std::os::unix::fs::MetadataExt;
use std::fs::Metadata;
use std::collections::HashMap;
use std::fs::File;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xip")]
#[command(about = "impoverished git, written in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add { path: String }, 
    Commit {message: String }, 
    Status { dir_name: Option<String> },
    Checkout {target: String}, 
    Ls,
    Branch { branch_name: Option<String> },
    Switch { branch_name: String }
    // Log
}

fn main() {
    let cli = Cli::parse();
    let mut index = Index::fetch_index();
    match cli.command {
        Commands::Init => { init::init_repo(); },
        Commands::Add { path } => { stage::add_to_index(path, &mut index); },
        Commands::Ls => {
            if index.header.count == 0 {
                println!("stage something");
                return;
            }
            index.ls_files();
        },
        Commands::Commit { message } => { 
            /* warn user if modified, untracked and deleted files exist */
            let status = Status::get_dir_status(".", &index);
            if status.untracked.len() > 0 ||  status.modified.len() > 0 ||  status.deleted.len() > 0 {
                Status::print_status(&status);
                println!("unstaged changes, run 'xip add' to index");
            } else {
                commit::build_commit(&index, &message); 
            }
        },
        Commands::Checkout { target } => { checkout::checkout(&target); }, 
        Commands::Status { dir_name } => { 
            if let Some(dname) = dir_name {
                Status::print_status(&Status::get_dir_status(&dname.clone(), &index)); 
            } else {
                Status::print_status(&Status::get_dir_status(".", &index)); 
            }
        },
        Commands::Branch { branch_name } => { 
            if let Some(name) = branch_name {
                branch::create(&name);
            } else {
                branch::ls();
            }
        }, 
        Commands::Switch { branch_name }=> { branch::switch(&branch_name); }
        // Commands::Log => { println!("{}", log::Log::get_log()); }
    }
}
