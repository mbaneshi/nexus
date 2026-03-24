//! Nexus CLI — home command center.
//!
//! Single binary with subcommands for discovery, config management, AI queries,
//! TUI dashboard, and web server.

mod commands;

use clap::{Parser, Subcommand};
use color_eyre::eyre;

#[derive(Parser)]
#[command(
    name = "nexus",
    version,
    about = "Home command center — discovery, configs, AI"
)]
struct Cli {
    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan home directory and index all files
    Scan {
        /// Root directory to scan (default: ~)
        #[arg(short, long)]
        root: Option<String>,
    },

    /// Search indexed files via FTS5
    Search {
        /// Search query
        query: String,

        /// Filter by category (config, code, document, image, etc.)
        #[arg(short, long)]
        category: Option<String>,

        /// Maximum results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show home directory statistics
    Stats,

    /// Manage config tools in ~/.config
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Ask AI a question about your filesystem
    Ask {
        /// Your question
        question: String,
    },

    /// Launch TUI dashboard
    Tui,

    /// Start web server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3141")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// List all discovered config tools
    List,

    /// Show config files for a tool
    Show {
        /// Tool name (e.g., nvim, git, fish)
        tool: String,
    },

    /// Backup config files (snapshot to database)
    Backup {
        /// Tool name (or all if omitted)
        tool: Option<String>,

        /// Label for this snapshot
        #[arg(short, long)]
        label: Option<String>,
    },

    /// List snapshots
    Snapshots {
        /// Filter by tool name
        tool: Option<String>,
    },

    /// Restore config files from a snapshot
    Restore {
        /// Snapshot ID
        id: i64,
    },

    /// Initialize nexus config at ~/.config/nexus/config.toml
    Init,

    /// Show nexus config file path
    Path,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Load config and open database
    let config = nexus_core::config::load()?;
    let conn = nexus_core::db::open(&config.database)?;

    match cli.command {
        Commands::Scan { root } => commands::scan::run(&conn, &config, root.as_deref(), cli.json),
        Commands::Search {
            query,
            category,
            limit,
        } => commands::search::run(&conn, &query, category.as_deref(), limit, cli.json),
        Commands::Stats => commands::stats::run(&conn, cli.json),
        Commands::Config { action } => commands::config::run(&conn, action, cli.json),
        Commands::Ask { question } => commands::ask::run(&conn, &config, &question),
        Commands::Tui => {
            nexus_tui::run(conn)?;
            Ok(())
        }
        Commands::Serve { port } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(nexus_server::run(&config.server.host, port, conn))?;
            Ok(())
        }
    }
}
