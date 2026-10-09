# xip: impoverished git in Rust

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

## Areas Needing Attention or Completion

1. **Error Handling & Edge Cases**
   * Heavy reliance on `.unwrap()` and `.expect()` throughout filesystem operations, which can cause unhandled panics if `.xip` files are corrupt or missing.
   * Hardcoded user and email (HVSR hvsr@gmail.com) during commit creation rather than pulling from a configuration file.

---

## Next Steps & Suggestions

1. **Implement merge: the ORT strategy**
   * Introduce the Ostensibly Recursive's Twin (ORT) strategy to efficiently handle multi-tree branch merges and conflict resolution, integrating the architecture guidelines detailed in the README.md file. 
2. **Implement `xip log`**
   * Complete the parser in `log.rs` to walk back parent commit hashes starting from `HEAD`.
3. **Configuration Support**
   * Add a `.xip/config` or system environment reader to dynamically get the author name and email.
4. **Refactor Errors**
   * Swap out critical `.unwrap()` calls for proper `Result<T, E>` error propagation to avoid crashes on bad states.