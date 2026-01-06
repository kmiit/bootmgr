use crate::cli::Commands;
use crate::interface::{Handle, Interface};
use clap::Parser;

mod interface;
mod cli;
mod common;

#[cfg(windows)]
mod win;

fn main() {
    let handle = Handle::new();
    let cmd = Commands::parse();
    let mut arg_p = false;

    match cmd {
        Commands::List {
            grub,
            #[cfg(windows)]
            bcd,
        } => {
            if grub {
                arg_p = true;
                handle.show_grub_entry()
            }
            #[cfg(windows)]
            if bcd {
                arg_p = true;
                handle.show_bcd_entry()
            }
        }
        Commands::Set {
            grub,
            #[cfg(windows)]
            bcd,
        } => {
            if let Some(grub_entry) = grub {
                arg_p = true;
                handle.set_grub_entry(grub_entry);
            }
            #[cfg(windows)]
            if let Some(bcd_entry) = bcd {
                arg_p = true;
                handle.set_bcd_entry(bcd_entry)
            }
        }
    }
    if !arg_p {
        println!("No arguments provided, use --help for more information");
    }
}
