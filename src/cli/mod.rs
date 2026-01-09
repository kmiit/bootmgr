use clap::Parser;

#[derive(Parser)]
#[command(name = "Boot Manager")]
#[command(about = "A tool to manage boot entries order")]
pub(crate) enum Commands {
    List {
        #[arg(short, long, help = "List the GRUB boot entries")]
        grub: bool,

        #[arg(short, long, help = "List the firmware boot entries")]
        firmware: bool,

        #[arg(short, long, help = "Description for the entry of grub")]
        description: Option<String>,
    },
    Set {
        #[arg(short, long, help = "Set the GRUB entry by id or index")]
        grub: Option<String>,

        #[arg(short, long, help = "Set the BCD entry by identifier")]
        firmware: Option<String>,

        #[arg(short, long, help = "Description for the entry of grub")]
        description: Option<String>,
    },
}
