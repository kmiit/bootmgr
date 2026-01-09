mod bcd_helper;
mod volume_helper;

use crate::common::file_operations;
use crate::interface::{GrubEntry, Handle, Interface};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Result, Write};
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

fn is_admin() -> bool {
    unsafe {
        let mut token_handle = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut return_size = size_of::<TOKEN_ELEVATION>() as u32;

        let success = GetTokenInformation(
            token_handle,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
            return_size,
            &mut return_size,
        )
        .is_err();

        CloseHandle(token_handle).expect("Failed to close token handle");

        !success && elevation.TokenIsElevated != 0
    }
}

fn rerun_as_administrator() {
    unsafe {
        let exe_path = std::env::current_exe().unwrap();
        let exe_w: Vec<u16> = make_os_str(exe_path.to_str().unwrap());
        let args: Vec<String> = std::env::args().skip(1).collect();
        let args_string = args.join(" ");
        let params_w: Vec<u16> = make_os_str(&args_string);

        let result = ShellExecuteW(
            Option::from(HWND::default()),
            w!("runas"),
            PCWSTR(exe_w.as_ptr()),
            PCWSTR(params_w.as_ptr()),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
        if result.0 as u32 <= 32 {
            panic!("Failed to elevate, error={:?}", result.0);
        }

        std::process::exit(0);
    }
}

impl Interface for Handle {
    fn check_permission(&self) -> bool {
        is_admin()
    }
    fn rerun_as_superuser(&self) {
        rerun_as_administrator();
    }

    fn get_grub_entry(&self) -> Result<Vec<GrubEntry>> {
        let grub_cfg = self.get_file(file_operations::GRUB_CFG_PATH.to_string())?;
        let grub_env = self.get_file(file_operations::GRUB_ENV_PATH.to_string())?;
        self.parse_grub_entries(grub_cfg, grub_env)
    }

    fn get_file(&self, path: String) -> Result<File> {
        let device = bcd_helper::get_grub_location(self.grub_desc.clone())?;
        if device.clone().unwrap().as_str() == "" {
            panic!("Failed to get grub location");
        }

        let volume_guid = volume_helper::get_volume_guid_path(device.unwrap().as_str())?;
        let mount_point = volume_helper::mount_volume_temporarily(volume_guid.as_str())?;
        let file = file_operations::open_file_ro(mount_point.join(path))?;
        volume_helper::unmount_volume(mount_point).expect("Umount failed");
        Ok(file)
    }

    fn write_file(&self, path: String, content: String) -> Result<()> {
        let device = bcd_helper::get_grub_location(self.grub_desc.clone())?;
        if device.clone().unwrap() == "" {
            panic!("Failed to get grub location");
        }
        let volume_guid = volume_helper::get_volume_guid_path(device.unwrap().as_str())?;
        let mount_point = volume_helper::mount_volume_temporarily(volume_guid.as_str())?;
        let mut file = file_operations::open_file_wo(mount_point.join(path))?;
        file.write_all(content.as_bytes())?;
        volume_helper::unmount_volume(mount_point).expect("Umount failed");
        Ok(())
    }

    fn show_fw_entry(&self) {
        bcd_helper::show_bcd_list();
    }

    fn set_fw_entry(&self, entry: String) -> Result<()> {
        println!("Set BCD firmware entry to {}", entry);
        bcd_helper::set_bcd_entry(entry)
    }
}

pub fn make_os_str(string: &str) -> Vec<u16> {
    OsStr::new(string).encode_wide().chain(Some(0)).collect()
}
