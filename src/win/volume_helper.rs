use crate::win::make_os_str;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::Win32::Foundation::{HANDLE, MAX_PATH};
use windows::Win32::Storage::FileSystem::{
    DeleteVolumeMountPointW, FindFirstVolumeW, FindNextVolumeW, FindVolumeClose, QueryDosDeviceW,
    SetVolumeMountPointW,
};
use windows::core::PCWSTR;

struct FindVolumeHandle(HANDLE);

impl FindVolumeHandle {
    fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for FindVolumeHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = FindVolumeClose(self.0);
        }
    }
}

pub(crate) fn get_volume_guid_path(nt_device_path: &str) -> Result<String> {
    let mut volume_name_buffer = vec![0u16; (MAX_PATH as usize) + 1];
    let mut device_path_buffer = vec![0u16; (MAX_PATH as usize) + 1];

    let find_handle = unsafe { FindFirstVolumeW(&mut *volume_name_buffer) }?;

    if find_handle.is_invalid() {
        return Err(Error::last_os_error());
    }

    let _find_guard = FindVolumeHandle::new(find_handle);

    loop {
        let vol_len = volume_name_buffer
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(volume_name_buffer.len());

        let current_guid_path = OsString::from_wide(&volume_name_buffer[..vol_len])
            .to_string_lossy()
            .to_string();

        let mut guid_to_query = current_guid_path.trim_end_matches('\\').to_string();
        const VOLUME_PREFIX: &str = "\\\\?\\";
        if guid_to_query.starts_with(VOLUME_PREFIX) {
            guid_to_query = guid_to_query[VOLUME_PREFIX.len()..].to_string(); // "Volume{GUID}"
        }

        let guid_path_for_query_w: Vec<u16> = make_os_str(&guid_to_query);

        let query_result = unsafe {
            QueryDosDeviceW(
                PCWSTR(guid_path_for_query_w.as_ptr()),
                Some(&mut *device_path_buffer),
            )
        };

        if query_result > 0 {
            let max_len = std::cmp::min(query_result as usize, device_path_buffer.len());
            let actual_len = device_path_buffer
                .iter()
                .take(max_len)
                .position(|&c| c == 0)
                .unwrap_or(max_len);

            let actual_device_path = OsString::from_wide(&device_path_buffer[..actual_len])
                .to_string_lossy()
                .to_string();

            let actual_device_path = actual_device_path
                .trim_end_matches(|c: char| {
                    c == '\u{0000}' || c == '\u{00A0}' || c == '\u{FEFF}' || c.is_whitespace()
                })
                .to_string();

            if actual_device_path == nt_device_path {
                return Ok(current_guid_path);
            }
        } else {
            let err = Error::last_os_error();
            eprintln!("QueryDosDeviceW failed: {:?}", err);
        }

        let next_result = unsafe { FindNextVolumeW(_find_guard.raw(), &mut *volume_name_buffer) };

        if next_result.is_err() {
            let error = Error::last_os_error();
            if error.kind() == ErrorKind::NotFound {
                break;
            } else {
                return Err(error);
            }
        }
    }

    Err(Error::new(
        ErrorKind::NotFound,
        format!("Could not find GUID path for NT device: {}", nt_device_path),
    ))
}

pub(crate) fn mount_volume_temporarily(volume_guid: &str) -> Result<PathBuf> {
    let vol = volume_guid.to_string();

    let mut mount_dir = env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let unique_name = format!("grub_mgr_mount_{}_{}\\", std::process::id(), now);
    mount_dir.push(unique_name);

    fs::create_dir(&mount_dir)?;

    let mount_w: Vec<u16> = make_os_str(mount_dir.to_str().unwrap());
    let vol_w: Vec<u16> = make_os_str(vol.as_str());

    let set_result =
        unsafe { SetVolumeMountPointW(PCWSTR(mount_w.as_ptr()), PCWSTR(vol_w.as_ptr())) };
    if let Err(ref e) = set_result {
        eprintln!(
            "SetVolumeMountPointW failed: {:?}\n  mount_str={:?}\n  vol={:?}\n  mount_w_len={} vol_w_len={}",
            e,
            mount_dir,
            vol,
            mount_w.len(),
            vol_w.len()
        );
    }

    match set_result {
        Ok(()) => Ok(mount_dir),
        Err(e) => {
            fs::remove_dir(&mount_dir)?;
            Err(Error::new(
                ErrorKind::Other,
                format!("SetVolumeMountPointW failed: {:?}", e),
            ))
        }
    }
}

pub(crate) fn unmount_volume(mount_path: PathBuf) -> Result<()> {
    let mount_w: Vec<u16> = make_os_str(mount_path.to_str().unwrap());
    let del_result = unsafe { DeleteVolumeMountPointW(PCWSTR(mount_w.as_ptr())) };

    match del_result {
        Ok(()) => {
            fs::remove_dir(mount_path)
                .map_err(|e| Error::new(ErrorKind::Other, format!("remove_dir failed: {}", e)))?;
            Ok(())
        }
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("DeleteVolumeMountPointW failed: {:?}", e),
        )),
    }
}
