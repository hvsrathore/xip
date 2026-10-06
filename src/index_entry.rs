use std::os::unix::fs::MetadataExt;

pub struct IndexEntry {
    /* the last time a file's metadata changed */
    pub ctime_sec: u32,
    /* ctime nanosecond fractions */
    pub ctime_nsec: u32,
    /* the last time a file's data changed */
    pub mtime_sec: u32,
    /* mtime nanosecond fractions */
    pub mtime_nsec: u32,
    /* device' ID the file sits on */
    dev: u32,
    /* file inode number in fs */
    ino: u32,
    /* 	File permissions and type (e.g., regular file, executable, symlink) */
    mode: u32,
    /* user ID of the file owner */
    uid: u32,
    /* group ID of the file owner */
    gid: u32,
    /* the truncated 32-bit size of the file on disk */
    pub file_size: u32,
    /* 160-bit SHA-1 of the staged blob */
    pub hash: [u8; 20],
    /* 16-bit bitfield (includes merge stage, assume-unchanged, path length) */
    flags: u16,
    /* the relative path string (e.g., src/main.c) relative to the working tree */
    pub path: String,
}

fn read_u32 (data: &[u8], cursor: usize) -> (u32, usize) {
    /* Slice 4 bytes and convert to [u8; 4] */
    let bytes: [u8; 4] = data[cursor..cursor + 4].try_into().unwrap();
    /* Return the decoded number and the new cursor */
    (u32::from_be_bytes(bytes), cursor + 4)
}

impl IndexEntry {
    pub fn new (meta: std::fs::Metadata, hash: [u8; 20], path:String) -> IndexEntry {
        IndexEntry { 
            ctime_sec: meta.ctime() as u32,
            ctime_nsec: meta.ctime_nsec() as u32,
            mtime_sec: meta.mtime() as u32,
            mtime_nsec: meta.mtime_nsec() as u32,
            dev: meta.rdev() as u32,
            ino: meta.ino() as u32, 
            mode: meta.mode() as u32, 
            uid: meta.uid() as u32, 
            gid: meta.gid() as u32,
            file_size: meta.len() as u32,
            hash: hash,
            flags: path.len() as u16, 
            path: path
        }
    }

    pub fn parse_entry (data: &[u8], cursor: usize) -> (IndexEntry, usize) {
        let (ctime_sec, cursor) = read_u32(data, cursor);
        let (ctime_nsec, cursor) = read_u32(data, cursor);
        let (mtime_sec, cursor) = read_u32(data, cursor);
        let (mtime_nsec, cursor) = read_u32(data, cursor);
        let (dev, cursor) = read_u32(data, cursor);
        let (ino, cursor) = read_u32(data, cursor);
        let (mode, cursor) = read_u32(data, cursor);
        let (uid, cursor) = read_u32(data, cursor);
        let (gid, cursor) = read_u32(data, cursor);
        let (file_size, mut cursor) = read_u32(data, cursor);

        let hash: [u8; 20] = data[cursor..cursor + 20].try_into().unwrap();
        cursor += 20;
        let flags: u16 = u16::from_be_bytes(data[cursor..cursor + 2].try_into().unwrap());
        cursor += 2;
        /* cursor should now be @ byte 62 */
        let mut null_index = 0;
        for (i, &byte) in data[cursor..].iter().enumerate() {
            null_index = i;
            if byte == 0x00 {
                break;
            }
        }
        let path_bytes = &data[cursor..cursor + null_index];
        let path = String::from_utf8(path_bytes.to_vec()).unwrap();
        /* Git requires the total byte length of every entry to be a multiple of 8. 
         * The fixed fields (from ctime_sec down to flags) take exactly 62 bytes. 
         * The filename takes null_index bytes.
         * The math to calculate the exact number of null padding bytes Git added 
         * is: 8 - ((62 + null_index) % 8) */
        let padding = 8 - ((62 + null_index) % 8);
        cursor = cursor + null_index + padding;
        /* populate the struct entries */
        let entry = IndexEntry { 
            ctime_sec, ctime_nsec,
            mtime_sec, mtime_nsec,
            dev, ino, mode, uid, gid,
            file_size, hash, flags, path
        };
        (entry, cursor)
    }

    pub fn to_bytes (&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        let path_len = self.path.len();
        let pad_len = 8 - ((62 + path_len) % 8);
        let padding = vec![0; pad_len];

        bytes.extend_from_slice(&self.ctime_sec.to_be_bytes());
        bytes.extend_from_slice(&self.ctime_nsec.to_be_bytes());
        bytes.extend_from_slice(&self.mtime_sec.to_be_bytes());
        bytes.extend_from_slice(&self.mtime_nsec.to_be_bytes());
        bytes.extend_from_slice(&self.dev.to_be_bytes());
        bytes.extend_from_slice(&self.ino.to_be_bytes());
        bytes.extend_from_slice(&self.mode.to_be_bytes());
        bytes.extend_from_slice(&self.uid.to_be_bytes());
        bytes.extend_from_slice(&self.gid.to_be_bytes());
        bytes.extend_from_slice(&self.file_size.to_be_bytes());
        bytes.extend_from_slice(&self.hash);
        bytes.extend_from_slice(&self.flags.to_be_bytes());
        bytes.extend_from_slice(self.path.as_bytes());
        bytes.extend_from_slice(&padding);
        bytes
    }
}
