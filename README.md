# xip: Rust Implementation of a Git-like Version Control System

Key core capabilities, basic workflows, and performance optimizations are already functioning, though several areas remain incomplete or in draft form.

## Core Progress & Highlights

1. **Repository Initialization (`init.rs`)**
   * Successfully initializes the `.xip/` structure, including `objects/`, `refs/heads/`, and a `HEAD` file referencing 'main'.  
2. **Index File Management (`index.rs`, `index_header.rs`, `index_entry.rs`)**
   * Binary parsing and serialization of Xip index headers (DIRC, version 2) and file entries.  
   * Calculates padding bytes to maintain strict Git 8-byte alignment.  
   * Uses memory-mapped files (`memmap2`) for efficient index reads.  
3. **Staging Area (`stage.rs`, `object.rs`)**
   * Concurrent blob generation and hashing using `rayon` for untracked files.  
   * Supports creating SHA-1 content-addressed blob objects with zlib compression.  
   * Handles updates to modified files, staging untracked files, and purging deleted files from the index.  
4. **Status Operations (`status.rs`, `utils.rs`)**
   * Traverses the working directory recursively to detect modified, untracked, and deleted files.  
   * Evaluates file modification using `mtime` and file size, falling back on blob hashing comparisons.  
5. **Tree & Commit Generation (`tree.rs`, `commit.rs`)**
   * Recursively converts flat staged paths into subtrees and hashes/writes tree objects to disk.  
   * Writes compressed commit objects containing tree hashes, parent hashes, timestamps, author details, and commit messages.  
   * Correctly tracks both branched and detached HEAD states.  
6. **Branching & Checkout (`branch.rs`, `checkout.rs`)**
   * Branch listing, creation, and switching (`xip branch`, `xip switch`).  
   * Checkout reclaims or restores file states from trees/blobs in parallel.

---

## Known Logical Bugs 

There are a few dangerous logical traps hiding in the state transitions. Here are the four most critical logical bugs currently identified:

1. **The "Unsaved Work Destroyer" (`checkout.rs`)** 
   When `build_working_dir` is called, it maps out the new tree and directly calls `fs::write` to place the blobs on disk. However, there is no check to see if the user has uncommitted, unsaved work in their current directory. If they do, `checkout` will silently overwrite their hard work with the older committed versions.   
2. **The "Ghost Files" (`checkout.rs`)** 
   When checking out a previous commit, the code successfully writes the files that belong to that older commit. But what happens to brand new files the user created after that commit? The `checkout` logic never deletes them. They will remain in the working directory as untracked "ghost" files, mixing two different timelines together.   
3. **The "Premature Branch" Panic (`branch.rs`)** 
   In `branch::create`, the code reads `.xip/HEAD` to find the current branch, and then immediately tries to read the commit hash from that branch's ref file to copy it. But if a user initializes a brand new repo and types `xip branch feature` before making their very first commit, that branch ref file won't exist yet. The `.unwrap()` will cause a fatal panic.   
4. **The "All-or-Nothing" Commit Blocker (`main.rs`)** 
   In the CLI routing for `Commit`, the code checks if `status.untracked`, `status.modified`, or `status.deleted` have any files. If they do, it blocks the commit entirely and tells the user to run `add`. But Git allows you to stage just one file and commit it, even if you have 10 other unstaged files. The current logic forces the working directory to be 100% clean before it allows a commit.   

---

## Areas Needing Attention or Completion

1. **Unimplemented Log Module (`log.rs`)**
   * Log functions (`parse_commit`, `build_cmap`, `get_log`) are currently stubs with empty bodies or incomplete logic.  
   * The Log subcommand is commented out in `main.rs`.  
2. **Error Handling & Edge Cases**
   * Heavy reliance on `.unwrap()` and `.expect()` throughout filesystem operations, which can cause unhandled panics if `.xip` files are corrupt or missing.  
   * Hardcoded user and email (HVSR hvsr@gmail.com) during commit creation rather than pulling from a configuration file.

---

## Next Steps & Suggestions

1. **Implement `xip log`**
   * Complete the parser in `log.rs` to walk back parent commit hashes starting from `HEAD`.  
2. **Configuration Support**
   * Add a `.xip/config` or system environment reader to dynamically get the author name and email.  
3. **Refactor Errors**
   * Swap out critical `.unwrap()` calls for proper `Result<T, E>` error propagation to avoid crashes on bad states.
