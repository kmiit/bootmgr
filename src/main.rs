use crate::cli::Commands;
use crate::interface::{Handle, Interface};
use clap::Parser;

mod cli;
mod common;
mod interface;

#[cfg(windows)]
mod win;

fn main() {
    let mut handle = Handle::new();
    let cmd = Commands::parse();
    let mut arg_p = false;

    match cmd {
        Commands::List {
            grub,
            firmware,
            description,
        } => {
            handle.grub_desc = description;
            if grub {
                arg_p = true;
                handle.show_grub_entry()
            }
            if firmware {
                arg_p = true;
                handle.show_fw_entry()
            }
        }
        Commands::Set {
            grub,
            firmware,
            description,
        } => {
            handle.grub_desc = description;
            if let Some(grub_entry) = grub {
                arg_p = true;
                handle.set_grub_entry(grub_entry);
            }
            if let Some(fw_entry) = firmware {
                arg_p = true;
                handle.set_fw_entry(fw_entry).unwrap()
            }
        }
    }
    if !arg_p {
        println!("No needed arguments provided, use --help for more information");
    }
}
