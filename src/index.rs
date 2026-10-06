use crate::index_header::IndexHeader;
use crate::index_entry::IndexEntry;
use std::fs::File;
use memmap2::Mmap;
use std::env;


use std::fs;
use std::os::unix::fs::MetadataExt;
use sha1::{Sha1, Digest};
use hex::encode;

pub struct Index {
    pub header: IndexHeader,
    pub entries: Vec<IndexEntry>,
    checksum: [u8; 20]
}

impl Index {
    pub fn new () -> Index {
        let header = IndexHeader::new(); 
        let entries = Vec::<IndexEntry>::new();
        let checksum = [0; 20];
        Index { header, entries, checksum } 
    }

    pub fn parse_index (data: &[u8]) -> Index {
        let header = IndexHeader::parse_header(data);
        let mut cursor = 12;
        let mut entries: Vec<IndexEntry> = Vec::new();
        for _ in 0..header.count {
            let (entry, next_cursor) = IndexEntry::parse_entry(data, cursor); 
            entries.push(entry);  
            cursor = next_cursor;
        }
        let checksum: [u8; 20] = data[data.len() - 20..].try_into().unwrap();
        Index { header, entries, checksum }
    }

    pub fn fetch_index () -> Index {
        let cwd = env::current_dir().unwrap();
        let index_path = format!("{}/.xip/index", cwd.display());
        match File::open(&index_path) {
            Ok(file) => {
                let index_data = unsafe { Mmap::map(&file).expect("Failed to map index file") };
                Index::parse_index(&index_data)
            },
            Err(_) => Index::new()
        }
    }

    pub fn ls_files(&self) {
        for entry in self.entries.iter() {
            println!("{} -> {}", entry.path, hex::encode(entry.hash));
        }
    }

    pub fn to_bytes (&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&self.header.to_bytes());
        for entry in self.entries.iter() {
            bytes.extend_from_slice(&entry.to_bytes());
        }
        bytes.extend_from_slice(&self.checksum);
        bytes
    }
     
    pub fn add_entry (&mut self, path: String, hash: [u8; 20]) {
        let meta = std::fs::metadata(&path)
            .expect("Error reading file metadata");

        let existing_index = self.entries.iter().position(|entry| entry.path == path);
        match existing_index {
            Some(existing_index) => 
                self.entries[existing_index] = IndexEntry::new(meta, hash, path),
            None => {
                let entry = IndexEntry::new(meta, hash, path);
                self.entries.push(entry);
                self.header.count += 1;
            }
        }
    }
}
