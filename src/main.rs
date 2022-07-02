mod compiler_explorer;

use notify::{self, RecursiveMode, Watcher};
use std::{sync::mpsc::channel, time::Duration};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ce", about = "Run compiler explorer on local sources")]
struct Opts {
    #[structopt(name = "FILE")]
    file: std::path::PathBuf,

    #[structopt(name = "ARGS")]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opts = Opts::from_args();

    let (tx, rx) = channel();

    let mut watcher = notify::watcher(tx, Duration::from_millis(100))?;

    watcher.watch(opts.file, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv()? {
            notify::DebouncedEvent::Create(file) | notify::DebouncedEvent::Write(file) => {
                let file_contents = std::fs::read(file)?;
                let file_contents = String::from_utf8(file_contents)?;

                compiler_explorer::compile(&file_contents, &opts.args[..]).await?;
            }
            notify::DebouncedEvent::Remove(file) => {
                println!("File was removed: {:?}", file);
                break;
            }
            notify::DebouncedEvent::Error(e, f) => {
                println!("Error {:?} watching file: {:?}", e, f);
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
