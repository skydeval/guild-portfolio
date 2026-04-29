use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use fs2::FileExt;
use is_terminal::IsTerminal;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

const STORAGE_PATH: &str = "tracker.json";
const LOCK_PATH: &str = "tracker.json.lock";
const TMP_PATH: &str = "tracker.json.tmp";
const SCHEMA_VERSION: u32 = 1;

// ANSI color escape codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

// Set once at the top of main() based on --no-color, NO_COLOR env, and isatty.
// Stdout and stderr are checked independently so redirecting one without the
// other doesn't leak ANSI codes into the redirected stream.
static USE_COLOR_STDOUT: AtomicBool = AtomicBool::new(true);
static USE_COLOR_STDERR: AtomicBool = AtomicBool::new(true);

fn use_color() -> bool {
    USE_COLOR_STDOUT.load(Ordering::Relaxed)
}

fn use_color_stderr() -> bool {
    USE_COLOR_STDERR.load(Ordering::Relaxed)
}

fn paint(code: &str, text: &str) -> String {
    if use_color() {
        format!("{}{}{}", code, text, RESET)
    } else {
        text.to_string()
    }
}

fn paint_stderr(code: &str, text: &str) -> String {
    if use_color_stderr() {
        format!("{}{}{}", code, text, RESET)
    } else {
        text.to_string()
    }
}

#[derive(Parser)]
#[command(
    name = "tracker",
    version,
    about = "A personal issue tracker CLI",
    long_about = "A personal issue tracker CLI. Tracks tasks in a local JSON file in \
                  the current directory. Issues have a status (open/in-progress/done), \
                  a priority (low/medium/high), and zero or more labels.\n\n\
                  Quick start:\n  \
                  tracker create \"Fix login\" --priority high --label bug\n  \
                  tracker list\n  \
                  tracker status 1 in-progress\n  \
                  tracker show 1"
)]
struct Cli {
    /// Disable ANSI colors in output (also respects NO_COLOR and non-tty stdout)
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new issue
    #[command(long_about = "Create a new issue.\n\n\
                            Examples:\n  \
                            tracker create \"Fix login bug\"\n  \
                            tracker create \"Add dark mode\" --priority high --label feature\n  \
                            tracker create \"Audit deps\" --description \"Quarterly review\" --label security")]
    Create {
        /// The issue title
        title: String,
        /// Priority level: low, medium, or high
        #[arg(long, default_value = "medium")]
        priority: Priority,
        /// Optional longer description
        #[arg(long)]
        description: Option<String>,
        /// Label to attach. Pass --label multiple times for multiple labels.
        #[arg(long = "label")]
        labels: Vec<String>,
    },
    /// List issues, with optional filters
    #[command(long_about = "List issues. Defaults to open issues; combine filters to narrow.\n\n\
                            Examples:\n  \
                            tracker list\n  \
                            tracker list --status done\n  \
                            tracker list --status all\n  \
                            tracker list --priority high\n  \
                            tracker list --label bug\n  \
                            tracker list --label bug --label backend\n  \
                            tracker list --status open --priority high --label backend")]
    List {
        /// Filter by status: open, in-progress, done, or all (defaults to open)
        #[arg(long)]
        status: Option<StatusFilter>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<Priority>,
        /// Filter to issues that have this label. Pass --label multiple times
        /// to require all listed labels (AND semantics, matching --label on create).
        #[arg(long = "label")]
        labels: Vec<String>,
    },
    /// Show full details of an issue
    Show {
        /// The issue id
        id: u32,
    },
    /// Change the status of an issue
    #[command(long_about = "Change the status of an issue.\n\n\
                            Statuses: open, in-progress, done\n\n\
                            Example: tracker status 1 in-progress")]
    Status {
        /// The issue id
        id: u32,
        /// The new status (open, in-progress, done)
        status: Status,
    },
    /// Delete an issue (asks for confirmation)
    Delete {
        /// The issue id
        id: u32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum Status {
    Open,
    InProgress,
    Done,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::InProgress => "in-progress",
            Status::Done => "done",
        }
    }
}

/// Filter-only enum for `tracker list --status`. Wraps Status and adds an
/// "all" sentinel so users can list every issue regardless of status. Kept
/// separate from Status so the storage-level state type stays clean — "all"
/// is a query concept, not a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum StatusFilter {
    Open,
    InProgress,
    Done,
    All,
}

impl StatusFilter {
    fn matches(self, status: Status) -> bool {
        match self {
            StatusFilter::All => true,
            StatusFilter::Open => status == Status::Open,
            StatusFilter::InProgress => status == Status::InProgress,
            StatusFilter::Done => status == Status::Done,
        }
    }

    fn label(self) -> &'static str {
        match self {
            StatusFilter::Open => "open",
            StatusFilter::InProgress => "in-progress",
            StatusFilter::Done => "done",
            StatusFilter::All => "all",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn label(self) -> &'static str {
        match self {
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    /// Returns the priority label, painted with its color when colors are enabled.
    fn colored_label(self) -> String {
        let code = match self {
            Priority::High => RED,
            Priority::Medium => YELLOW,
            Priority::Low => DIM,
        };
        paint(code, self.label())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Issue {
    id: u32,
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_status")]
    status: Status,
    #[serde(default = "default_priority")]
    priority: Priority,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default = "default_now")]
    created_at: DateTime<Utc>,
    #[serde(default = "default_now")]
    updated_at: DateTime<Utc>,
}

fn default_status() -> Status {
    Status::Open
}

fn default_priority() -> Priority {
    Priority::Medium
}

fn default_now() -> DateTime<Utc> {
    Utc::now()
}

/// Storage envelope. Wraps the bookkeeping (schema_version, next_id) around the
/// issue list so we don't have to derive ids from current state on every create.
#[derive(Serialize, Deserialize, Debug)]
struct Storage {
    schema_version: u32,
    next_id: u32,
    issues: Vec<Issue>,
}

impl Default for Storage {
    fn default() -> Self {
        Storage {
            schema_version: SCHEMA_VERSION,
            next_id: 1,
            issues: Vec::new(),
        }
    }
}

/// Strip ANSI escape sequences and ASCII control characters from a
/// user-supplied string. Defends against escape injection that would otherwise
/// be re-emitted on every list call. Strips full sequences as units rather
/// than just the leading ESC byte (which would leave visible "[31m" remnants).
///
/// Handles three escape families:
///   - CSI: ESC [ ... <final byte 0x40-0x7E>     (used by SGR colors)
///   - OSC: ESC ] ... <terminator BEL or ESC \>  (used by terminal title hacks)
///   - DCS / SOS / PM / APC: ESC P/X/^/_ ... <ST: ESC \>
///
/// If `allow_newlines` is true, `\n` and `\r` are preserved (descriptions
/// support multi-line prose). Otherwise they're stripped along with other
/// control bytes (titles and labels are single-line).
fn sanitize_text(s: &str, allow_newlines: bool) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                // CSI: skip up to and including a final byte in 0x40-0x7E.
                Some(&'[') => {
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                // OSC: skip until BEL (0x07) or ESC \ (ST). Worst case, drop to EOF.
                Some(&']') => {
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '\x07' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                // DCS, SOS, PM, APC: skip until ST (ESC \).
                Some(&'P') | Some(&'X') | Some(&'^') | Some(&'_') => {
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                // Any other escape — drop the ESC and one following char (Fp/Fe/Fs).
                Some(_) => {
                    chars.next();
                }
                None => {}
            }
            continue;
        }
        // Strip control chars; allow tab and (optionally) newlines through.
        if c.is_control() {
            if c == '\t' {
                out.push(c);
                continue;
            }
            if allow_newlines && (c == '\n' || c == '\r') {
                out.push(c);
                continue;
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Normalize labels: sanitize, trim, lowercase, drop empties, dedupe. Stable
/// ordering via BTreeSet. Sanitize runs before trim so leading control chars
/// don't preserve their adjacent whitespace.
fn normalize_labels(input: Vec<String>) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for raw in input {
        let sanitized = sanitize_text(&raw, false);
        let cleaned = sanitized.trim().to_lowercase();
        if !cleaned.is_empty() {
            set.insert(cleaned);
        }
    }
    set.into_iter().collect()
}

/// Same normalization the create path applies, exposed so the list filter can
/// be symmetric. Round 2 #7: filter input previously only lowercased.
fn normalize_label_for_filter(raw: &str) -> String {
    let sanitized = sanitize_text(raw, false);
    sanitized.trim().to_lowercase()
}

/// Acquire an advisory exclusive file lock that lives for the whole command.
/// Created via a sidecar `.lock` file so we never conflict with the
/// write-then-rename happening on the data file.
fn acquire_lock() -> Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(LOCK_PATH)
        .with_context(|| format!("opening lock file {}", LOCK_PATH))?;
    file.lock_exclusive()
        .context("acquiring exclusive lock on tracker (another tracker process may be running)")?;
    Ok(file)
}

fn load_storage() -> Result<Storage> {
    if !Path::new(STORAGE_PATH).exists() {
        return Ok(Storage::default());
    }
    let raw = fs::read_to_string(STORAGE_PATH)
        .with_context(|| format!("reading {}", STORAGE_PATH))?;
    if raw.trim().is_empty() {
        return Ok(Storage::default());
    }

    // Try the modern envelope shape first.
    if let Ok(mut storage) = serde_json::from_str::<Storage>(&raw) {
        // Refuse to operate on files written by a future tracker. Without
        // this, an old binary would silently deserialize garbage or lose data.
        if storage.schema_version > SCHEMA_VERSION {
            return Err(anyhow!(
                "{} was written by a newer tracker (schema version {}). \
                 This binary supports up to schema version {}. \
                 Upgrade tracker or use a different file.",
                STORAGE_PATH,
                storage.schema_version,
                SCHEMA_VERSION
            ));
        }
        // Defense in depth against a hand-edited or corrupted next_id that
        // would collide with an existing id.
        let max_id = storage.issues.iter().map(|i| i.id).max().unwrap_or(0);
        if storage.next_id <= max_id {
            storage.next_id = max_id + 1;
        }
        return Ok(storage);
    }

    // Fall back to the legacy flat-array shape (pre-round-1 storage format).
    // Migrate forward by computing next_id from the max existing id + 1, which
    // matches the legacy behavior on first load. After this load, future writes
    // use the envelope and stable next_id.
    if let Ok(legacy_issues) = serde_json::from_str::<Vec<Issue>>(&raw) {
        let next_id = legacy_issues.iter().map(|i| i.id).max().unwrap_or(0) + 1;
        return Ok(Storage {
            schema_version: SCHEMA_VERSION,
            next_id,
            issues: legacy_issues,
        });
    }

    Err(anyhow!(
        "could not parse {}. The file may be corrupted. \
         Back up tracker.json and delete it to start fresh.",
        STORAGE_PATH
    ))
}

/// Atomic save: write to a temp file, fsync, then rename over the target.
/// rename(2) is atomic on POSIX, so a kill between truncate and write can never
/// leave the tracker file half-written.
fn save_storage(storage: &Storage) -> Result<()> {
    let json = serde_json::to_string_pretty(storage)
        .context("serializing storage")?;

    {
        let mut tmp = File::create(TMP_PATH)
            .with_context(|| format!("creating {}", TMP_PATH))?;
        tmp.write_all(json.as_bytes())
            .with_context(|| format!("writing {}", TMP_PATH))?;
        tmp.sync_all()
            .with_context(|| format!("syncing {}", TMP_PATH))?;
    }

    if let Err(e) = fs::rename(TMP_PATH, STORAGE_PATH) {
        // Clean up the tmp file so a rename failure doesn't leave debris.
        // Best-effort; if cleanup fails too, the original error is more useful.
        let _ = fs::remove_file(TMP_PATH);
        return Err(e).with_context(|| format!("renaming {} to {}", TMP_PATH, STORAGE_PATH));
    }
    Ok(())
}

/// Prompt the user with the given message and return true if they answer y/yes.
fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [y/N] ", prompt);
    io::stdout().flush().context("flushing stdout")?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).context("reading stdin")?;
    let answer = input.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
}

fn cmd_create(
    title: String,
    priority: Priority,
    description: Option<String>,
    labels: Vec<String>,
) -> Result<()> {
    let title = sanitize_text(&title, false);
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(anyhow!("title cannot be empty"));
    }
    let description = description.map(|d| {
        let cleaned = sanitize_text(&d, true);
        cleaned.trim().to_string()
    });

    let mut storage = load_storage()?;
    let id = storage.next_id;
    storage.next_id = storage
        .next_id
        .checked_add(1)
        .ok_or_else(|| anyhow!("ran out of issue ids"))?;

    let now = Utc::now();
    let issue = Issue {
        id,
        title,
        description,
        status: Status::Open,
        priority,
        labels: normalize_labels(labels),
        created_at: now,
        updated_at: now,
    };

    let title_for_print = issue.title.clone();
    storage.issues.push(issue);
    save_storage(&storage)?;
    println!(
        "{} issue #{}: {}",
        paint(GREEN, "Created"),
        id,
        title_for_print
    );
    Ok(())
}

fn cmd_list(
    status_filter: Option<StatusFilter>,
    priority_filter: Option<Priority>,
    label_filters: Vec<String>,
) -> Result<()> {
    let storage = load_storage()?;

    let effective_status = status_filter.unwrap_or(StatusFilter::Open);
    let normalized_labels: Vec<String> = label_filters
        .iter()
        .map(|s| normalize_label_for_filter(s))
        .filter(|s| !s.is_empty())
        .collect();

    let mut matches: Vec<&Issue> = storage
        .issues
        .iter()
        .filter(|i| effective_status.matches(i.status))
        .filter(|i| match priority_filter {
            Some(p) => i.priority == p,
            None => true,
        })
        .filter(|i| {
            // AND semantics: every requested label must be present on the issue.
            normalized_labels
                .iter()
                .all(|requested| i.labels.iter().any(|l| l == requested))
        })
        .collect();

    matches.sort_by_key(|i| (Reverse(i.priority), i.id));

    if matches.is_empty() {
        if storage.issues.is_empty() {
            println!("No issues yet. Create one with:");
            println!("  tracker create \"your first issue\"");
        } else {
            let no_other_filters =
                priority_filter.is_none() && normalized_labels.is_empty();
            if effective_status == StatusFilter::Open && no_other_filters {
                println!("No open issues.");
            } else {
                let mut parts = vec![format!("status={}", effective_status.label())];
                if let Some(p) = priority_filter {
                    parts.push(format!("priority={}", p.label()));
                }
                for label in &normalized_labels {
                    parts.push(format!("label={}", label));
                }
                println!("No issues match: {}.", parts.join(", "));
            }
        }
        return Ok(());
    }

    for issue in matches {
        let labels_str = if issue.labels.is_empty() {
            String::new()
        } else {
            let tags = issue
                .labels
                .iter()
                .map(|l| format!("#{}", l))
                .collect::<Vec<_>>()
                .join(" ");
            format!(" {}", paint(CYAN, &tags))
        };
        println!(
            "#{:<4} [{}] [{}] {}{}",
            issue.id,
            issue.priority.colored_label(),
            issue.status.label(),
            issue.title,
            labels_str,
        );
    }
    Ok(())
}

fn cmd_show(id: u32) -> Result<()> {
    let storage = load_storage()?;
    let issue = storage
        .issues
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow!("no issue with id #{}", id))?;

    let id_str = format!("#{}", issue.id);
    let title_str = issue.title.clone();
    println!("{}  {}", paint(BOLD, &id_str), paint(BOLD, &title_str));
    println!("  status:     [{}]", issue.status.label());
    println!("  priority:   [{}]", issue.priority.colored_label());
    if !issue.labels.is_empty() {
        let tags = issue
            .labels
            .iter()
            .map(|l| format!("#{}", l))
            .collect::<Vec<_>>()
            .join(" ");
        println!("  labels:     {}", paint(CYAN, &tags));
    } else {
        println!("  labels:     {}", paint(DIM, "none"));
    }
    println!(
        "  created:    {}",
        issue.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "  updated:    {}",
        issue.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    if let Some(desc) = &issue.description {
        println!();
        println!("{}", paint(DIM, "--- description ---"));
        println!("{}", desc);
    }
    Ok(())
}

fn cmd_status(id: u32, new_status: Status) -> Result<()> {
    let mut storage = load_storage()?;
    let issue = storage
        .issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow!("no issue with id #{}", id))?;

    if issue.status == new_status {
        println!(
            "Issue #{} is already {}. No change.",
            id,
            new_status.label()
        );
        return Ok(());
    }

    let old_status = issue.status;
    issue.status = new_status;
    issue.updated_at = Utc::now();
    save_storage(&storage)?;
    println!(
        "{} issue #{}: {} -> {}",
        paint(GREEN, "Updated"),
        id,
        old_status.label(),
        new_status.label()
    );
    Ok(())
}

fn cmd_delete(id: u32) -> Result<()> {
    let mut storage = load_storage()?;
    let pos = storage
        .issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| anyhow!("no issue with id #{}", id))?;

    let title = storage.issues[pos].title.clone();
    let prompt = format!("Delete issue #{}: {}?", id, paint(BOLD, &title));
    if !confirm(&prompt)? {
        println!("Cancelled.");
        return Ok(());
    }

    storage.issues.remove(pos);
    save_storage(&storage)?;
    println!("{} issue #{}.", paint(GREEN, "Deleted"), id);
    Ok(())
}

/// Decide whether to emit ANSI colors on the given stream. Honors --no-color,
/// the NO_COLOR env var (https://no-color.org/), and the per-stream isatty.
fn determine_color(no_color_flag: bool, stream_is_terminal: bool) -> bool {
    if no_color_flag {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    stream_is_terminal
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    USE_COLOR_STDOUT.store(
        determine_color(cli.no_color, std::io::stdout().is_terminal()),
        Ordering::Relaxed,
    );
    USE_COLOR_STDERR.store(
        determine_color(cli.no_color, std::io::stderr().is_terminal()),
        Ordering::Relaxed,
    );

    // Hold an exclusive advisory lock for the duration of the command. This
    // prevents concurrent tracker invocations from clobbering each other.
    let _lock = acquire_lock()?;

    match cli.command {
        Command::Create {
            title,
            priority,
            description,
            labels,
        } => cmd_create(title, priority, description, labels),
        Command::List {
            status,
            priority,
            labels,
        } => cmd_list(status, priority, labels),
        Command::Show { id } => cmd_show(id),
        Command::Status { id, status } => cmd_status(id, status),
        Command::Delete { id } => cmd_delete(id),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Color the "error:" label red unless stderr is being captured.
            // Note this uses paint_stderr (not paint) so redirecting only
            // stderr to a file doesn't leave ANSI codes in the log.
            eprintln!("{} {}", paint_stderr(RED, "error:"), e);
            for cause in e.chain().skip(1) {
                eprintln!("  {} {}", paint_stderr(DIM, "caused by:"), cause);
            }
            ExitCode::FAILURE
        }
    }
}