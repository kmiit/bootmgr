use std::io::{Error, ErrorKind, Result};
use std::process::Command;

#[derive(Default, Debug)]
struct BcdEntry {
    pub id: Option<String>,
    pub device: Option<String>,
    pub path: Option<String>,
    pub description: Option<String>,
    pub locale: Option<String>,
    pub inherit: Option<String>,
    pub default: Option<String>,
    pub resumeobject: Option<String>,
    pub displayorder: Vec<Option<String>>,
    pub toolsdisplayorder: Option<String>,
    pub timeout: Option<u32>,
}

impl BcdEntry {
    pub fn entry_on_disk(&self) -> bool {
        self.device != None && self.path != None
    }
}

pub(crate) fn show_bcd_list() {
    let entries = get_bcd_entries().expect("Failed to get BCD entries");
    for i in &entries {
        println!(
            "{}",
            format!(
                "{}{} {} ({})",
                if i.id == entries[0].id { ">" } else { "  " },
                if i.id == entries[0].displayorder[0] {
                    "*"
                } else {
                    " "
                },
                i.description.clone().unwrap(),
                i.id.clone().unwrap()
            )
        );
    }
}

pub(crate) fn set_bcd_entry(entry: String) -> Result<()> {
    let entries = get_bcd_entries()?;
    let find_entry = entries.iter().skip(1).any(|i| {
        i.description.clone().unwrap().to_lowercase() == entry.to_lowercase()
            || i.id.clone().unwrap().to_lowercase() == entry.to_lowercase()
    });
    if !find_entry {
        eprintln!("BCD entry {} not found", entry);
        Err(Error::new(ErrorKind::NotFound, "BCD entry not found"))
    } else {
        let status = Command::new("bcdedit.exe")
            .args(&[
                "/set",
                "{fwbootmgr}",
                "displayorder",
                entry.as_str(),
                "/addfirst",
            ])
            .status();
        if status.is_err() {
            Err(Error::new(ErrorKind::Other, "set BCD entry failed"))
        } else {
            Ok(())
        }
    }
}

pub(crate) fn get_grub_location(description: Option<String>) -> Result<Option<String>> {
    let desc = description.unwrap_or_else(|| "grub".to_string()).to_lowercase();
    let entries = get_bcd_entries()?;
    let mut device: Option<String> = None;
    for i in entries {
        if !i.entry_on_disk() {
            continue;
        }
        if i.description
            .clone()
            .unwrap()
            .to_lowercase()
            .contains(desc.as_str())
        {
            device = Option::from(
                i.device.unwrap().split('=').collect::<Vec<&str>>()[1]
                    .trim()
                    .to_string(),
            );
            return Ok(device);
        }
    }
    Ok(device)
}

fn get_bcd_entries() -> Result<Vec<BcdEntry>> {
    let output = run_bcdedit_enum()?;
    Ok(parse_bcd_entries(output))
}

fn run_bcdedit_enum() -> Result<String> {
    let output = Command::new("bcdedit.exe")
        .args(&["/enum", "firmware"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn parse_bcd_entries(output: String) -> Vec<BcdEntry> {
    let sections = split_sections(output.as_str());
    let mut entries = Vec::new();
    entries.push(parse_entry(sections[0].clone(), Vec::new()));
    for section in sections.iter().skip(1) {
        entries.push(parse_entry(
            section.clone(),
            entries[0].displayorder.clone(),
        ));
    }
    entries
}

fn parse_entry(section: String, order: Vec<Option<String>>) -> BcdEntry {
    let mut ret = BcdEntry::default();
    let mut lines_iter = section
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .peekable();

    while let Some(line) = lines_iter.next() {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();

        if parts.len() < 2 {
            continue;
        }

        let key = parts[0];
        let value = Some(parts[1].trim().to_string());

        match key {
            "device" => ret.device = value,
            "path" => ret.path = value,
            "description" => ret.description = value,
            "locale" => ret.locale = value,
            "inherit" => ret.inherit = value,
            "default" => ret.default = value,
            "resumeobject" => ret.resumeobject = value,
            "toolsdisplayorder" => ret.toolsdisplayorder = value,
            "timeout" => {
                ret.timeout = value.unwrap().trim().parse::<u32>().ok();
            }
            "displayorder" => {
                ret.displayorder.push(value);
                while let Some(next_line) = lines_iter.peek() {
                    let next_parts: Vec<&str> = next_line.splitn(2, ' ').collect();
                    if next_parts.len() == 1 {
                        ret.displayorder
                            .push(Option::from(next_parts[0].to_string()));
                        lines_iter.next();
                    } else {
                        break;
                    }
                }
            }
            _ => {
                // Handle the 'identifier' key separately
                if order.contains(&value) || value == Some("{fwbootmgr}".to_string()) {
                    ret.id = value;
                }
                if ret.id == Some("{fwbootmgr}".to_string()) {
                    ret.description = Some("UEFI Loader".to_string());
                }
            }
        }
    }
    ret
}

fn split_sections(input: &str) -> Vec<String> {
    let processed_input = input.replace("\r\n", "\n");
    processed_input
        .split("\n\n")
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
