use crate::common::file_operations;
use regex::Regex;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Result};
use std::process::exit;

pub(crate) trait Interface {
    /// Check if the current user has permission to run the program
    /// # Returns
    /// * `bool` - true if the user has permission, false otherwise
    fn check_permission(&self) -> Result<bool>;

    /// Rerun the program as a superuser
    fn rerun_as_superuser(&self) -> Result<()>;

    /// Get the grub entries from the grub.cfg file
    /// # Returns
    /// * `Result<Vec<GrubEntry>>` - A vector of GrubEntry objects
    fn get_grub_entry(&mut self) -> Result<Vec<GrubEntry>> {
        let grub_cfg = self.get_file(file_operations::GRUB_CFG_PATH)?;
        let grub_env = self.get_file(file_operations::GRUB_ENV_PATH)?;
        self.parse_grub_entries(grub_cfg, grub_env)
    }

    /// Get a file at the given path
    /// # Arguments
    /// * `path` - The path to the file
    /// # Returns
    /// * `Result<File>` - A File object representing the file
    fn get_file(&mut self, path: &str) -> Result<File>;

    /// Parse the grub.cfg file to get the grub entries
    /// # Arguments
    /// * `cfg` - A File object representing the grub.cfg file
    /// * `env` - A File object representing the grubenv file
    /// # Returns
    /// * `Result<Vec<GrubEntry>>` - A vector of GrubEntry objects
    fn parse_grub_entries(&self, mut cfg: File, env: File) -> Result<Vec<GrubEntry>> {
        let default_entry_id = self.parse_grub_env(env)?;
        let mut cfg_content = String::new();
        cfg.read_to_string(&mut cfg_content)?;
        let mut entries = Vec::new();

        let menuentry_regex =
            Regex::new(r"menuentry\s+'([^']+)'(?:.*?\$menuentry_id_option\s+'([^']+)')?").unwrap();
        let submenu_regex =
            Regex::new(r"submenu\s+'([^']+)'(?:.*?\$menuentry_id_option\s+'([^']+)')?").unwrap();

        let mut current_submenu_name: Option<String> = None;

        for line in cfg_content.lines() {
            if let Some(captures) = submenu_regex.captures(line) {
                let submenu_name = captures[1].to_string();
                current_submenu_name = Some(submenu_name);
                continue;
            }
            if let Some(captures) = menuentry_regex.captures(line) {
                let entry_name = captures[1].to_string();
                let entry_id = captures.get(2).map(|m| m.as_str().to_string()).unwrap();
                let is_default = entry_id == default_entry_id;

                let entry = GrubEntry {
                    entry_name,
                    entry_id,
                    entry_in_submenu: current_submenu_name.is_some(),
                    entry_is_default: is_default,
                };
                entries.push(entry);
                continue;
            }

            if line.trim() == "}" && current_submenu_name.is_some() {
                current_submenu_name = None;
            }
        }

        Ok(entries)
    }

    /// Write content to a file at the given path
    /// # Arguments
    /// * `path` - The path to the file
    /// * `content` - The content to write to the file
    /// # Returns
    /// * `Result<()>` - Ok if successful, Err otherwise
    fn write_file(&mut self, path: &str, content: &str) -> Result<()>;

    /// Parse the grubenv file to get the default grub entry id
    /// /// # Arguments
    /// * `file` - A File object representing the grubenv file
    /// /// # Returns
    /// * `Result<String>` - The default grub entry id
    fn parse_grub_env(&self, mut file: File) -> Result<String> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let prefix = "saved_entry=";
        let mut entry_id = String::new();
        for line in content.lines() {
            if line.starts_with(prefix) {
                entry_id = line[prefix.len()..].trim().to_string();
            }
        }
        Ok(entry_id)
    }

    /// Set the default grub entry in the grubenv file
    /// # Arguments
    /// * `grub_entry` - A GrubEntry object representing the grub entry to set as default
    /// # Returns
    /// * `Result<()>` - Ok if successful, Err otherwise
    fn set_default_grub_entry(&mut self, grub_entry: &GrubEntry) -> Result<()> {
        println!("Set default GRUB entry: {:?}", grub_entry.entry_name);
        let mut env_file = self.get_file(file_operations::GRUB_ENV_PATH)?;
        let mut content = String::new();
        env_file.read_to_string(&mut content)?;
        for line in content.lines() {
            if line.starts_with("saved_entry=") {
                let new_content =
                    content.replace(line, &format!("saved_entry={}", grub_entry.entry_id));
                self.write_file(file_operations::GRUB_ENV_PATH, &*new_content)?;
            }
        }
        Ok(())
    }

    /// Show the grub entries
    fn show_grub_entry(&mut self) -> Result<()> {
        let entries = self.get_grub_entry()?;
        println!("Grub entry:");
        for i in entries {
            println!(
                "{}",
                format!(
                    "{} {}{} ({})",
                    if i.entry_is_default { "*" } else { " " },
                    if i.entry_in_submenu { "  " } else { "" },
                    i.entry_name,
                    i.entry_id
                )
            );
        }
        Ok(())
    }

    /// Set the grub entry by id or index
    /// # Arguments
    /// * `entry_id` - The id or index of the grub entry to set as default
    fn set_grub_entry(&mut self, entry_id: String) -> Result<()> {
        let entries = self.get_grub_entry()?;

        let entry = match entry_id.parse::<usize>() {
            Ok(index) => entries.get(index),
            Err(_) => entries.iter().find(|e| e.entry_id == entry_id),
        }
        .ok_or(Error::new(ErrorKind::NotFound, "GRUB entry not found"))?;

        self.set_default_grub_entry(entry)
    }

    /// Show the firmware boot entries
    fn show_fw_entry(&self) -> Result<()>;

    /// Set the firmware boot entry
    /// # Arguments
    /// * `entry` - The firmware boot entry to set
    fn set_fw_entry(&self, entry: String) -> Result<()>;

    /// Get the location of the GRUB installation
    /// # Returns
    /// * `Result<String>` - The location of the GRUB installation
    fn get_grub_loc(&mut self) -> Result<String>;
}

#[derive(Default)]
pub struct Handle {
    pub grub_desc: Option<String>,
    pub grub_loc: Option<String>,
}

impl Handle {
    pub(crate) fn new() -> Self {
        let s = Self::default();
        match s.check_permission() {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("No admin permission, restarting as administrator");
                let _ = s.rerun_as_superuser();
                exit(1);
            }
            Err(e) => {
                eprintln!("Failed to check permission: {}", e);
                exit(1);
            }
        }
        s
    }
}

/// A struct representing a GRUB entry
/// Fields:
/// * `entry_name` - The name of the GRUB menuentry
/// * `entry_id` - The menuentry_id_option of the GRUB menuentry
/// * `entry_in_submenu` - Whether the GRUB entry is in a submenu
/// * `entry_is_default` - Whether the GRUB entry is the default entry
#[derive(Clone, Debug)]
pub struct GrubEntry {
    pub entry_name: String,
    pub entry_id: String,
    pub entry_in_submenu: bool,
    pub entry_is_default: bool,
}

pub struct TempMount {
    pub(crate) device: String,
    pub(crate) mount_point: String,
}
