use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::Mutex;
use jwalk::WalkDir;

/// Calculate the total physical size of all files in a directory recursively
pub fn calculate_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    let seen_inodes = Mutex::new(HashSet::new());

    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| {
            let nlink = metadata.nlink();
            let inode = metadata.ino();

            if nlink > 1 {
                let mut seen = seen_inodes.lock().unwrap();
                if !seen.insert(inode) {
                    return 0; // Already counted this hard link
                }
            }

            // APFS physical size: calculate actual 512-byte blocks allocated
            metadata.blocks() * 512
        })
        .sum()
}

/// Get the user's home directory path
pub fn get_home_path() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .expect("HOME directory not set")
}
