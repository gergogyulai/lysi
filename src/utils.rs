use std::fs;
use std::path::Path;

pub fn move_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    match fs::rename(src, dst) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            fs::copy(src, dst)?;
            fs::remove_file(src)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}
