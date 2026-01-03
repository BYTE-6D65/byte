mod cli;
mod config;
mod exec;
mod forms;
mod fs;
mod log;
mod projects;
mod state;
mod path;
mod tui;

fn main() {
    // Initialize logger first
    log::Logger::init();

    if let Err(e) = run() {
        eprintln!("[ERROR] Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    cli::run()
}
