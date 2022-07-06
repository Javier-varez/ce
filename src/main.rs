mod compiler_explorer;
mod log;
mod tui;

use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio_stream::StreamExt;

use ::tui::{backend::CrosstermBackend, Terminal};

use notify::{self, RecursiveMode, Watcher};
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ce", about = "Run compiler explorer on local sources")]
struct Opts {
    #[structopt(short, long, default_value = "clang_trunk")]
    compiler: String,

    #[structopt(short = "u", long = "url", default_value = "https://godbolt.org")]
    compiler_explorer_url: String,

    #[structopt(short, long)]
    log: bool,

    #[structopt(short = "v", long = "vertical")]
    vertical_orientation: bool,

    #[structopt(short, long)]
    execute: bool,

    #[structopt(name = "FILE")]
    file: std::path::PathBuf,

    #[structopt(name = "ARGS")]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opts = Opts::from_args();

    if opts.log {
        log::configure_logger()?;
    }

    let orientation = if opts.vertical_orientation {
        println!("vertical orientation");
        tui::Orientation::Vertical
    } else {
        println!("horizontal orientation");
        tui::Orientation::Horizontal
    };

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    let mut ui = tui::Ui::new(orientation);
    ui.draw(&mut terminal)?;

    let cannonical_path = std::fs::canonicalize(&opts.file)?;
    let parent = cannonical_path.parent().unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(tx, Duration::from_millis(300))?;
    watcher.watch(parent, RecursiveMode::NonRecursive)?;

    let (async_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel();

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);
    tokio::task::spawn_blocking(move || loop {
        if let Ok(data) = rx.recv_timeout(Duration::from_millis(500)) {
            async_tx.send(data).unwrap();
        }
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
    });

    let file_contents = String::from_utf8(std::fs::read(&opts.file)?)?;
    let result = compiler_explorer::compile(
        &opts.compiler_explorer_url,
        &opts.compiler,
        &file_contents,
        &opts.args[..],
        opts.execute,
    )
    .await?;
    ui.set_data(result);
    ui.draw(&mut terminal)?;

    let mut event_stream = crossterm::event::EventStream::new();
    loop {
        let event = event_stream.next();
        let notify_ev = notify_rx.recv();

        tokio::select! {
                event = event => {
                    match event {
                        Some(Ok(Event::Key(KeyEvent { code: KeyCode::Esc, .. }))) => {
                            ::log::info!("Exiting");
                            break;
                        }
                        Some(Ok(Event::Key(event))) => {
                            ui.handle_key_event(event, &mut terminal)?;
                        }
                        Some(Ok(Event::Resize(_,_))) => {
                            ui.draw(&mut terminal)?;
                    }
                        _ => {}
                    }
                }
                notify_ev = notify_ev => {
                    ::log::debug!("Received file event: {:?}", notify_ev);
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
                            opts.execute,
                        )
                        .await?;

                        ui.set_data(result);
                        ui.draw(&mut terminal)?;
                    }
                    Some(notify::DebouncedEvent::Error(e, f)) => {
                        ::log::error!("Error {:?} watching file: {:?}", e, f);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Notify async threads about shutdown
    shutdown_tx.send(())?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
