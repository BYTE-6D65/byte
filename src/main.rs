mod cli;
mod config;
mod exec;
mod forms;
mod fs;
mod projects;
mod state;
mod tui;

fn main() {
    eprintln!("[INFO] Byte starting...");

    if let Err(e) = run() {
        eprintln!("[ERROR] Error: {}", e);
        std::process::exit(1);
    }

    eprintln!("[INFO] Byte completed");
}

fn run() -> anyhow::Result<()> {
    cli::run()
}
