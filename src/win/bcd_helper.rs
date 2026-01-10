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

pub(crate) fn show_bcd_list() -> Result<()> {
    let entries = get_bcd_entries()?;
    println!("The firmware boot entries(BCD):");
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
    Ok(())
}

pub(crate) fn set_bcd_entry(entry: String) -> Result<()> {
    let entries = get_bcd_entries()?;
    let find_entry = entries.iter().skip(1).any(|i| {
        i.description.clone().unwrap().to_lowercase() == entry.to_lowercase()
            || i.id.clone().unwrap().to_lowercase() == entry.to_lowercase()
    });
    if !find_entry {
        Err(Error::new(ErrorKind::NotFound, "BCD entry not found"))
    } else {
        if Command::new("bcdedit.exe")
            .args(&[
                "/set",
                "{fwbootmgr}",
                "displayorder",
                entry.as_str(),
                "/addfirst",
            ])
            .status()
            .is_err()
        {
            return Err(Error::new(ErrorKind::Other, "set BCD entry failed"));
        }
        Ok(())
    }
}

pub(crate) fn get_grub_location(description: Option<String>) -> Result<Option<String>> {
    let mut grub_keywords: Vec<String> = vec![
        "grub".to_string(),
        "debian".to_string(),
        "ubuntu".to_string(),
    ];

    if let Some(desc) = description {
        grub_keywords.push(desc.to_lowercase());
    }

    for entry in get_bcd_entries()?.into_iter().filter(|e| e.entry_on_disk()) {
        let entry_desc = match entry.description {
            Some(d) => d.to_lowercase(),
            None => continue,
        };

        for keyword in &grub_keywords {
            if entry_desc.contains(keyword) {
                if let Some(device) = &entry.device {
                    if let Some((_, value)) = device.split_once('=') {
                        return Ok(Some(value.trim().to_string()));
                    }
                }
            }
        }
    }
    Ok(None)
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
