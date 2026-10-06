use sha1::{Sha1, Digest};
use hex::encode;
use std::fs;
use std::env;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write; // Required to write data into the encoder
use std::fs::File;
use memmap2::Mmap;

pub struct Blob;

impl Blob {
    pub fn format_blob (contents: &[u8]) -> Vec<u8> {
        let mut blob: Vec<u8> = Vec::new();
        let header = format!("blob {}\0", contents.len());
        blob.extend_from_slice(header.as_bytes());
        blob.extend_from_slice(contents);
        blob
    }

    pub fn hash_blob (blob: &[u8]) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(blob);
        hasher.finalize().into()
    }

    pub fn compress_blob (blob: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default()); 
        encoder.write_all(&blob)
            .expect("Unable to compress the blob");
        encoder.finish()
            .expect("Unable to finalize the compression on the blob")
    } 

    pub fn add_blob (path: String) -> [u8; 20] {
        let cwd = env::current_dir().unwrap();
        let meta = fs::symlink_metadata(&path).expect("Error reading metadata");
        let mut contents = Vec::new();
        if meta.is_symlink() {
            let link = fs::read_link(&path).expect("Unable to read symlink contents");             
            contents = link.to_str().unwrap().as_bytes().to_vec();
        } else {
            let mut file = File::open(&path).expect("Cannot open file when creating blob");
            let file_mmap = unsafe { Mmap::map(&file).expect("Failed to map file (blob)") };
            contents = file_mmap.to_vec();
        }
        let blob = Blob::format_blob(&contents);
        let hash = Blob::hash_blob(&blob);
        let compressed = Blob::compress_blob(&blob);
        let hash_str = hex::encode(hash);
        let dir_path = format!("{}/.xip/objects/{}", cwd.display(), &hash_str[..2]);
        let full_blob_path = format!("{}/.xip/objects/{}/{}", cwd.display(), &hash_str[..2], &hash_str[2..]);
        fs::create_dir_all(&dir_path)
            .expect("Unable to create directory for blob object");
        fs::write(full_blob_path, compressed)
            .expect("Unable to write blob object");
        /* this hash will be used as the filename for the blob
        * inside the index */
        hash
    }
}
