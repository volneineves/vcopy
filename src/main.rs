mod config;
mod i18n;
mod service;
mod setup;
mod storage;
mod ui;
mod watcher;

use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use config::Config;
use storage::{ClipItem, Storage};

#[derive(Parser)]
#[command(name = "vcopy", version, about = "Clipboard history manager")]
struct Cli {
    /// List history without opening the TUI
    #[arg(long)]
    list: bool,
    /// Search history without opening the TUI
    #[arg(long)]
    search: Option<String>,
    /// Delete a history item by id without opening the TUI
    #[arg(long)]
    delete: Option<i64>,
    /// Clear all history without opening the TUI
    #[arg(long)]
    clear: bool,
    /// Number of items to show for --list or --search
    #[arg(short = 'n', long, default_value_t = 20)]
    limit: usize,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the clipboard monitor loop (used by the systemd service)
    Daemon,
    /// Install and enable the systemd user service (auto-start on login)
    Install,
    /// Start the clipboard monitor in the background
    Start,
    /// Stop running clipboard monitor instances
    Stop,
    /// Restart the clipboard monitor in the background
    Restart,
    /// Disable and remove the systemd user service
    Uninstall,
    /// Show the daemon service status
    Status,
    /// List history items (pipe-friendly)
    List {
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Clear all history
    Clear,
    /// Show or change display language (en, pt, es)
    Lang {
        /// New language code to set (omit to show current)
        lang: Option<String>,
    },
    /// Configure shortcut key and terminal size
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Bootstrap i18n with the detected system locale so setup wizard messages
    // are shown in the right language before the config file exists.
    let detected = i18n::detect_system_lang();
    i18n::init(detected);

    // Load config (or run first-time setup); re-init if saved lang differs.
    let cfg = Config::load_or_setup()?;
    i18n::init(&cfg.language);

    let cli = Cli::parse();
    let storage = Storage::new()?;

    if handle_direct_command(&cli, &storage)? {
        return Ok(());
    }

    match cli.command {
        Some(Commands::Daemon) => {
            println!("{}", i18n::t().daemon_started);
            watcher::run(storage, cfg.history.clone()).await?;
        }
        Some(Commands::Install) => service::install()?,
        Some(Commands::Start) => service::start()?,
        Some(Commands::Stop) => service::stop()?,
        Some(Commands::Restart) => service::restart()?,
        Some(Commands::Uninstall) => service::uninstall()?,
        Some(Commands::Status) => service::status()?,
        Some(Commands::List { limit }) => {
            let items = storage.list(limit)?;
            for item in items {
                println!("{}", item.display_content());
            }
        }
        Some(Commands::Clear) => {
            storage.clear()?;
            println!("{}", i18n::t().history_cleared);
        }
        Some(Commands::Lang { lang: Some(code) }) => {
            let code = code.to_lowercase();
            if !["en", "pt", "es"].contains(&code.as_str()) {
                eprintln!("{}", i18n::t().lang_unknown);
                std::process::exit(1);
            }
            Config::set_language(&code)?;
            println!("{} {}", i18n::t().lang_changed, i18n::lang_label(&code));
        }
        Some(Commands::Lang { lang: None }) => {
            println!(
                "{} {} ({})",
                i18n::t().lang_current,
                i18n::lang_label(&cfg.language),
                cfg.language,
            );
        }
        Some(Commands::Config) => setup::run_wizard()?,
        None => {
            ui::run(storage)?;
        }
    }

    Ok(())
}

fn handle_direct_command(cli: &Cli, storage: &Storage) -> Result<bool> {
    if let Some(id) = cli.delete {
        if storage.delete(id)? {
            println!("Deleted item {id}.");
        } else {
            println!("Item {id} was not found.");
        }
        return Ok(true);
    }

    if cli.clear {
        storage.clear()?;
        println!("{}", i18n::t().history_cleared);
        return Ok(true);
    }

    if let Some(query) = &cli.search {
        for item in storage.search(query, cli.limit)? {
            println!("{}", format_cli_item(&item));
        }
        return Ok(true);
    }

    if cli.list {
        for item in storage.list(cli.limit)? {
            println!("{}", format_cli_item(&item));
        }
        return Ok(true);
    }

    Ok(false)
}

fn format_cli_item(item: &ClipItem) -> String {
    let copied_at = item
        .copied_at
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S");
    format!(
        "{:<5} {:<5} {}  {}",
        item.id,
        item.kind.label(),
        copied_at,
        truncate_for_cli(&item.display_content(), 120),
    )
}

fn truncate_for_cli(value: &str, max: usize) -> String {
    let value = value.replace('\n', " ");
    if value.len() > max {
        format!("{}...", &value[..max])
    } else {
        value
    }
}
