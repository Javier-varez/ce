mod compiler_explorer;
mod tui;

use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::{enable_raw_mode, Clear, ClearType},
};
use tokio_stream::StreamExt;

use ::tui::{backend::CrosstermBackend, Terminal};

use notify::{self, DebouncedEvent, RecursiveMode, Watcher};
use std::{path::Path, time::Duration};
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

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), Clear(ClearType::All))?;

    let cannonical_path = std::fs::canonicalize(&opts.file)?;
    let parent = cannonical_path.parent().unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(tx, Duration::from_millis(300))?;
    watcher.watch(parent, RecursiveMode::NonRecursive)?;

    let (async_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::task::spawn_blocking(move || loop {
        async_tx.send(rx.recv().unwrap()).unwrap();
    });

    let mut event_stream = crossterm::event::EventStream::new();
    loop {
        let event = event_stream.next();
        let notify_ev = notify_rx.recv();

        tokio::select! {
                event = event => {
                    match event {
                        Some(Ok(Event::Key(KeyEvent { code: KeyCode::Esc, .. }))) => {
                            println!("Exiting");
                            break;
                        }
                        _ => {}
                    }
                }
                notify_ev = notify_ev => {
                    match notify_ev {
                    Some(notify::DebouncedEvent::Create(file)) | Some(notify::DebouncedEvent::Write(file))
                        if std::fs::canonicalize(&file)? == cannonical_path => {
                        let file_contents = std::fs::read(file)?;
                        let file_contents = String::from_utf8(file_contents)?;

                        let result = compiler_explorer::compile(
                            &opts.compiler_explorer_url,
                            &opts.compiler,
                            &file_contents,
                            &opts.args[..],
                        )
                        .await?;

                        tui::update(&mut terminal, &result)?;
                    }
                    Some(notify::DebouncedEvent::Error(e, f)) => {
                        println!("Error {:?} watching file: {:?}", e, f);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
