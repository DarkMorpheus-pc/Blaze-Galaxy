//! Durable replacement and advisory locks. Lock files must never be unlinked:
//! replacing their inode would allow two processes to own different locks.
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("missing parent directory"))?;
    std::fs::create_dir_all(parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path).map_err(|err| err.error)?;
    File::open(parent)?.sync_all()
}

pub fn open_lock(path: &Path) -> io::Result<File> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)
}

pub fn try_lock(path: &Path) -> io::Result<File> {
    let file = open_lock(path)?;
    file.try_lock().map_err(io::Error::from)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_excludes_other_instances_and_releases_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instance.lock");
        let first = try_lock(&path).unwrap();
        assert_eq!(
            try_lock(&path).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        drop(first);
        assert!(try_lock(&path).is_ok());
    }

    #[test]
    fn readers_never_see_a_partial_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state");
        let a = vec![b'a'; 65536];
        let b = vec![b'b'; 32768];
        atomic_write(&path, &a).unwrap();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for _ in 0..30 {
                    atomic_write(&path, &b).unwrap();
                    atomic_write(&path, &a).unwrap();
                }
            });
            for _ in 0..300 {
                let bytes = std::fs::read(&path).unwrap();
                assert!(bytes == a || bytes == b);
            }
        });
    }
}
