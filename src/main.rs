mod compiler_explorer;
mod tui;

use notify::{self, RecursiveMode, Watcher};
use std::{sync::mpsc::channel, time::Duration};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ce", about = "Run compiler explorer on local sources")]
struct Opts {
    #[structopt(short, long, default_value = "clang_trunk")]
    compiler: String,

    #[structopt(short = "u", long = "url", default_value = "https://godbolt.org")]
    compiler_explorer_url: String,

    #[structopt(name = "FILE")]
    file: std::path::PathBuf,

    #[structopt(name = "ARGS", default_value = "-Os -std=c++20")]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opts = Opts::from_args();

    let (tx, rx) = channel();

    let mut watcher = notify::watcher(tx, Duration::from_millis(300))?;

    let cannonical_path = std::fs::canonicalize(&opts.file)?;
    // We watch the directory of the file for changes in our file.
    let parent = cannonical_path.parent().unwrap();
    watcher.watch(parent, RecursiveMode::NonRecursive)?;

    tui::init().unwrap();

    loop {
        match rx.recv()? {
            notify::DebouncedEvent::Create(file) | notify::DebouncedEvent::Write(file)
                if std::fs::canonicalize(&file)? == cannonical_path =>
            {
                let file_contents = std::fs::read(file)?;
                let file_contents = String::from_utf8(file_contents)?;

                compiler_explorer::compile(
                    &opts.compiler_explorer_url,
                    &opts.compiler,
                    &file_contents,
                    &opts.args[..],
                )
                .await?;
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
