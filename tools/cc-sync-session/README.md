# cc-sync-session

A CLI tool to sync Claude Code session files from the local storage to a repository for version control.

## Purpose

Claude Code stores session files locally in `~/.claude/projects/`. This tool helps you:
- Back up session files to a git repository
- Track changes to session history over time
- Share session logs with team members
- Maintain a one-way sync from the global session directory to your repository

## Installation

```bash
cargo install --path .
```

## Usage

### Initialize a repository

First, initialize your repository for session syncing:

```bash
# In a git repository
cc-sync-session init

# Or specify a repository directory
cc-sync-session init --repo-dir /path/to/your/repository
```

This creates `.claude/ccss_sessions/` directory in your repository with a `.gitkeep` file.

### Sync sessions

Basic usage (auto-detects repository):
```bash
# Run from within a repository that has been initialized
cc-sync-session sync
```

With explicit repository:
```bash
cc-sync-session sync --repo-dir /path/to/your/repository
```

With custom source directory:
```bash
cc-sync-session sync --source-dir ~/.claude/projects --repo-dir /path/to/your/repository
```

Using environment variable for source directory:
```bash
export CC_SYNC_SESSION_SOURCE_DIR=~/.claude/projects
cc-sync-session sync
```

Dry run to preview changes:
```bash
cc-sync-session sync --dry-run
```

Verbose output:
```bash
cc-sync-session sync --verbose
```

Enable debug logging:
```bash
RUST_LOG=debug cc-sync-session sync
```

## How it Works

1. **Repository Initialization**: Use `init` command to create `.claude/ccss_sessions/` directory in your repository. This marks the repository as ready for session syncing.

2. **Auto-detection**: The `sync` command can automatically find your repository by looking for directories with both `.git` and `.claude/ccss_sessions` in the current directory or parent directories.

3. **Directory Name Conversion**: Claude Code stores sessions with directory names where `/` is replaced with `-`. For example, `/Users/yuta/project` becomes `-Users-yuta-project`. This tool converts them back to the original path structure.

4. **One-way Sync**: Files are only copied from the source (Claude Code's session directory) to the target (your repository). This prevents accidental corruption of Claude Code's data.

5. **Timestamp-based Updates**: Only files that are newer in the source are copied, making subsequent syncs faster.

6. **Timestamp Marking**: After copying, the tool updates the file's timestamp in the target to track when it was last synced.

## Example

If your Claude Code sessions directory contains:
```
~/.claude/projects/
├── -Users-yuta-github.com-myproject/
│   ├── session.json
│   └── conversations/
│       └── conv1.json
└── -Users-yuta-work-project/
    └── session.json
```

After running:
```bash
cd ~/my-sessions-backup
cc-sync-session init
cc-sync-session sync
```

Your repository will contain:
```
~/my-sessions-backup/
└── .claude/
    └── ccss_sessions/
        ├── .gitkeep
        ├── Users/yuta/github.com/myproject/
        │   ├── session.json
        │   └── conversations/
        │       └── conv1.json
        └── Users/yuta/work/project/
            └── session.json
```

## Command Line Options

### Global options
- `-v, --verbose`: Enable verbose output (logs to stderr)

### `init` subcommand
- `-r, --repo-dir <PATH>`: Repository directory (defaults to current directory or parent with .git)

### `sync` subcommand
- `-s, --source-dir <PATH>`: Source directory containing Claude Code sessions (defaults to `$CC_SYNC_SESSION_SOURCE_DIR` or `~/.claude/projects/`)
- `-r, --repo-dir <PATH>`: Target repository directory (defaults to current directory or parent with .git and .claude/ccss_sessions)
- `-d, --dry-run`: Run in dry-run mode (show what would be done without making changes)

## Environment Variables

- `CC_SYNC_SESSION_SOURCE_DIR`: Default source directory when `--source-dir` is not specified
- `RUST_LOG`: Control log level (e.g., `RUST_LOG=info`, `RUST_LOG=debug`). When `-v` is used, defaults to `info`

## Pre-commit Hook Integration

You can use cc-sync-session as a [pre-commit](https://pre-commit.com/) hook to automatically sync Claude Code sessions before each commit.

### Setup

1. Install pre-commit:
   ```bash
   pip install pre-commit
   ```

2. Install cc-sync-session:
   ```bash
   cargo install --path /path/to/cc-sync-session
   ```

3. Create a `.pre-commit-config.yaml` file in your repository:
   ```yaml
   repos:
     - repo: https://github.com/higumachan/claude-code-work-arounds
       rev: main  # or specify a tag/commit
       hooks:
         - id: sync-claude-code-sessions
   ```

4. Install the pre-commit hook:
   ```bash
   pre-commit install
   ```

5. Initialize your repository for session syncing:
   ```bash
   cc-sync-session init
   ```

Now, every time you commit, cc-sync-session will automatically sync your Claude Code sessions to the repository.

### Available Hooks

- `sync-claude-code-sessions`: Runs sync before each commit
- `sync-claude-code-sessions-dry-run`: Preview what would be synced (manual stage only)

### Bypassing the Hook

If you need to commit without syncing sessions:
```bash
git commit --no-verify
```

## Logging

The tool uses the `log` crate with `env_logger` for logging. All logs are written to stderr.

- Without `-v` flag: Only warnings and errors are displayed
- With `-v` flag: Info level logs are enabled, showing:
  - Directory creation
  - File copies
  - Skipped files
  - Warnings about failed operations
- `RUST_LOG` environment variable can override the log level

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --test unit_tests

# Run integration tests only
cargo test --test integration_tests
```

### Architecture

The tool is designed with a clean architecture using dependency injection:

- `FileSystem` trait: Abstracts file system operations
- `MockFileSystem`: In-memory implementation for testing
- `RealFileSystem`: Actual file system implementation
- `SessionSyncer`: Core sync logic that works with any `FileSystem` implementation

This design allows for comprehensive unit testing without touching the actual file system.