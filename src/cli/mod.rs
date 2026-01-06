use clap::Parser;

#[derive(Parser)]
#[command(name = "Boot Manager")]
#[command(about = "A tool to manage GRUB entries from Windows")]
pub(crate) enum Commands {
    List {
        #[arg(short, long)]
        grub: bool,

        #[cfg(windows)]
        #[arg(short, long)]
        bcd: bool,
    },
    Set {
        #[arg(short, long)]
        grub: Option<String>,

        #[cfg(windows)]
        #[arg(short, long)]
        bcd: Option<String>,
    },
}
