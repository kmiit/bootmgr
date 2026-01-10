use crate::interface::TempMount;
use crate::win::make_os_str;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use windows::Win32::Storage::FileSystem::{
    DDD_NO_BROADCAST_SYSTEM, DDD_RAW_TARGET_PATH, DDD_REMOVE_DEFINITION, DefineDosDeviceW,
};
use windows::core::PCWSTR;

pub(crate) fn mount_volume_temporarily(mount_point: &str, device_path: &str) -> Result<()> {
    let mount_w: Vec<u16> = make_os_str(mount_point);
    let device_w: Vec<u16> = make_os_str(device_path);

    unsafe {
        DefineDosDeviceW(
            DDD_RAW_TARGET_PATH | DDD_NO_BROADCAST_SYSTEM,
            PCWSTR(mount_w.as_ptr()),
            PCWSTR(device_w.as_ptr()),
        )
    }
        .map_err(|e|  {
            Error::new(
                ErrorKind::Other,
                format!("DefineDosDeviceW mount volume failed: {:?}", e),
            )
        })
}

pub(crate) fn unmount_volume(mount_path: &str, device: &str) -> Result<()> {
    let mount_w = make_os_str(mount_path);
    let device_w = make_os_str(device);

    unsafe {
        DefineDosDeviceW(
            DDD_REMOVE_DEFINITION | DDD_RAW_TARGET_PATH,
            PCWSTR(mount_w.as_ptr()),
            PCWSTR(device_w.as_ptr()),
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
        let mount_point = "GRUB_TEMP_MOUNT_POINT";
        mount_volume_temporarily(mount_point, device_path)?;
        Ok(Self {
            device: device_path.to_string(),
            mount_point: mount_point.to_string(),
        })
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from(r"\\.\").join(&self.mount_point)
    }

    pub fn unmount(&self) -> Result<()> {
        unmount_volume(&self.mount_point, &self.device)
    }
}

impl Drop for TempMount {
    fn drop(&mut self) {
        let _ = self.unmount();
    }
}
