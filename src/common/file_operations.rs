use std::fs::File;
use std::fs::OpenOptions;
use std::io::Result;
use std::path::PathBuf;

pub const GRUB_CFG_PATH: &str = "grub/grub.cfg";
pub const GRUB_ENV_PATH: &str = "grub/grubenv";

pub fn open_file_ro(path: PathBuf) -> Result<File> {
    OpenOptions::new().read(true).open(path)
}

pub fn open_file_wo(path: PathBuf) -> Result<File> {
    OpenOptions::new().write(true).open(path)
}
