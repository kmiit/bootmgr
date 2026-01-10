use crate::interface::TempMount;
use crate::win::make_os_str;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use windows::Win32::Storage::FileSystem::{
    DDD_NO_BROADCAST_SYSTEM, DDD_RAW_TARGET_PATH, DDD_REMOVE_DEFINITION, DefineDosDeviceW,
};
use windows::core::PCWSTR;

pub(crate) fn mount_volume_temporarily(device_path: &str) -> Result<String> {
    let device = device_path.to_string();
    let mount_dir = "GRUB_TEMP_MOUNT";

    let mount_w: Vec<u16> = make_os_str(mount_dir);
    let device_w: Vec<u16> = make_os_str(device.as_str());

    let set_result = unsafe {
        DefineDosDeviceW(
            DDD_RAW_TARGET_PATH | DDD_NO_BROADCAST_SYSTEM,
            PCWSTR(mount_w.as_ptr()),
            PCWSTR(device_w.as_ptr()),
        )
    };
    if let Err(ref e) = set_result {
        eprintln!(
            "DefineDosDeviceW failed: {:?}\n  mount_str={:?}\n  vol={:?}\n",
            e, mount_dir, device,
        );
    }

    match set_result {
        Ok(()) => Ok(mount_dir.to_string()),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("DefineDosDeviceW failed: {:?}", e),
        )),
    }
}

pub(crate) fn unmount_volume(mount_path: &str) -> Result<()> {
    let mount_w = make_os_str(mount_path);

    unsafe {
        DefineDosDeviceW(
            DDD_REMOVE_DEFINITION | DDD_RAW_TARGET_PATH,
            PCWSTR(mount_w.as_ptr()),
            None,
        )
    }
    .map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("DefineDosDeviceW remove mount point failed: {:?}", e),
        )
    })
}

impl TempMount {
    pub fn new(device_path: &str) -> Result<Self> {
        let mount_point = mount_volume_temporarily(device_path)?;
        Ok(Self { mount_point })
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from(r"\\.\").join(&self.mount_point)
    }
}

impl Drop for TempMount {
    fn drop(&mut self) {
        if let Err(e) = unmount_volume(&self.mount_point) {
            eprintln!("Warning: failed to unmount volume: {:?}", e);
        }
    }
}
