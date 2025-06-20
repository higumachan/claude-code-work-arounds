use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cc_sync_session::{RealFileSystem, SessionSyncer, SyncOptions};
use log::warn;
use cc_sync_session::file_path_converter::dir_path_to_claude_code_stype;

#[derive(Parser, Debug)]
#[command(name = "cc-sync-session")]
#[command(about = "Sync Claude Code session files to a repository", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a repository for session syncing
    Init {
        /// Repository directory (defaults to current directory or parent with .git)
        #[arg(short = 'r', long)]
        repo_dir: Option<PathBuf>,
    },
    
    /// Sync session files to the repository
    Sync {
        /// Source directory containing Claude Code sessions
        /// (defaults to $CC_SYNC_SESSION_SOURCE_DIR or ~/.claude/projects/)
        #[arg(short, long)]
        source_dir: Option<PathBuf>,
        
        /// Target repository directory where sessions will be synced
        /// (defaults to current directory or parent with .git and .claude/ccss_sessions)
        #[arg(short = 'r', long)]
        repo_dir: Option<PathBuf>,
        
        /// Run in dry-run mode (show what would be done without making changes)
        #[arg(short, long)]
        dry_run: bool,
    },
}

/// Find a repository directory by looking for .git and .claude/ccss_sessions
fn find_repo_dir(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    
    loop {
        let git_dir = current.join(".git");
        let ccss_dir = current.join(".claude").join("ccss_sessions");

        log::debug!("Checking directory: {} git: {}, ccss: {}",
                    current.display(),
                    git_dir.exists(),
                    ccss_dir.exists());

        if git_dir.exists() && ccss_dir.exists() {
            return Some(current);
        }

        if !current.pop() {
            break;
        }
    }
    
    None
}

/// Find a git repository directory by looking for .git
fn find_git_repo(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    
    loop {
        let git_dir = current.join(".git");
        
        if git_dir.exists() {
            return Some(current);
        }
        
        if !current.pop() {
            break;
        }
    }
    
    None
}

fn init_command(repo_dir: Option<PathBuf>) -> Result<()> {
    let repo_dir = match repo_dir {
        Some(dir) => dir,
        None => {
            let current_dir = std::env::current_dir()
                .context("Failed to get current directory")?;
            find_git_repo(&current_dir)
                .context("No git repository found in current directory or parent directories")?
        }
    };
    
    // Create .claude/ccss_sessions directory
    let ccss_dir = repo_dir.join(".claude").join("ccss_sessions");
    std::fs::create_dir_all(&ccss_dir)
        .context("Failed to create .claude/ccss_sessions directory")?;
    
    // Create .gitkeep file
    let gitkeep_path = ccss_dir.join(".gitkeep");
    std::fs::write(&gitkeep_path, "")
        .context("Failed to create .gitkeep file")?;
    
    println!("Initialized session sync directory at: {}", ccss_dir.display());
    println!("Created: {}", gitkeep_path.display());
    
    Ok(())
}

fn sync_command(source_dir: Option<PathBuf>, repo_dir: Option<PathBuf>, dry_run: bool, verbose: bool) -> Result<()> {
    // Determine repository directory
    let repo_dir = match repo_dir {
        Some(dir) => dir,
        None => {
            let current_dir = std::env::current_dir()
                .context("Failed to get current directory")?;
            find_repo_dir(&current_dir)
                .context("No repository with .claude/ccss_sessions found. Run 'cc-sync-session init' first")?
        }
    };

    log::info!("Using repository directory: {}", repo_dir.display());
    let repo_dir_cc_style = dir_path_to_claude_code_stype(repo_dir.clone())?;
    log::debug!("Converted repository directory to Claude Code style: {}", repo_dir_cc_style);

    // Determine source directory
    let source_root_dir = match source_dir {
        Some(dir) => dir,
        None => {
            // Check environment variable first
            if let Ok(env_source) = std::env::var("CC_SYNC_SESSION_SOURCE_DIR") {
                PathBuf::from(env_source)
            } else {
                let home = dirs::home_dir()
                    .context("Failed to get home directory")?;
                home.join(".claude").join("projects")
            }
        }
    };

    let source_dir = source_root_dir.join(&repo_dir_cc_style);

    log::info!("Using source directory: {}", source_dir.display());

    
    // Target directory is .claude/ccss_sessions
    let target_dir = repo_dir.join(".claude").join("ccss_sessions");
    
    // Validate directories
    if !source_dir.exists() {
        anyhow::bail!("Source directory does not exist: {}", source_dir.display());
    }
    
    if !source_dir.is_dir() {
        anyhow::bail!("Source path is not a directory: {}", source_dir.display());
    }
    
    if !target_dir.exists() {
        anyhow::bail!("Target directory does not exist: {}. Run 'cc-sync-session init' first", target_dir.display());
    }
    
    // Print operation summary
    println!("Syncing Claude Code sessions:");
    println!("  Source: {}", source_dir.display());
    println!("  Target: {}", target_dir.display());
    if dry_run {
        println!("  Mode: DRY RUN (no changes will be made)");
    }
    println!();
    
    // Create syncer and run sync
    let filesystem = RealFileSystem::new();
    let syncer = SessionSyncer::new(filesystem);
    
    let options = SyncOptions {
        dry_run,
        verbose,
    };
    
    let result = syncer.sync(&source_root_dir, repo_dir_cc_style.as_str(), &target_dir, &options)
        .context("Failed to sync sessions")?;
    
    // Print results
    println!("\nSync completed:");
    println!("  Files copied: {}", result.files_copied);
    println!("  Files skipped: {}", result.files_skipped);
    println!("  Directories created: {}", result.directories_created);
    
    if !result.errors.is_empty() {
        println!("\nErrors encountered:");
        for error in &result.errors {
            warn!("{}", error);
            eprintln!("  - {}", error);
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logger
    let log_level = if cli.verbose {
        "info"
    } else {
        "warn"
    };
    
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .target(env_logger::Target::Stderr)
        .init();
    
    match cli.command {
        Commands::Init { repo_dir } => init_command(repo_dir),
        Commands::Sync { source_dir, repo_dir, dry_run } => {
            sync_command(source_dir, repo_dir, dry_run, cli.verbose)
        }
    }
}