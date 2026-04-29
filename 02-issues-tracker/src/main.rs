use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

const STORAGE_PATH: &str = "tracker.json";

// ANSI color escape codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

#[derive(Parser)]
#[command(
    name = "tracker",
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
                            tracker list --priority high\n  \
                            tracker list --label bug\n  \
                            tracker list --status open --priority high --label backend")]
    List {
        /// Filter by status (defaults to open)
        #[arg(long)]
        status: Option<Status>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<Priority>,
        /// Filter to issues that have this label
        #[arg(long)]
        label: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum Priority {
    // Order matters: derived Ord uses declaration order, so Low < Medium < High.
    // We use Reverse() when sorting so High appears first.
    Low,
    Medium,
    High,
}

impl Priority {
    fn colored_label(self) -> String {
        match self {
            Priority::High => format!("{}high{}", RED, RESET),
            Priority::Medium => format!("{}med{}", YELLOW, RESET),
            Priority::Low => format!("{}low{}", DIM, RESET),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
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

/// Normalize labels: trim, lowercase, drop empties, dedupe. Stable ordering via BTreeSet.
fn normalize_labels(input: Vec<String>) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for raw in input {
        let cleaned = raw.trim().to_lowercase();
        if !cleaned.is_empty() {
            set.insert(cleaned);
        }
    }
    set.into_iter().collect()
}

fn load_issues() -> Result<Vec<Issue>> {
    if !Path::new(STORAGE_PATH).exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(STORAGE_PATH)
        .with_context(|| format!("reading {}", STORAGE_PATH))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    let issues: Vec<Issue> = serde_json::from_str(&raw)
        .with_context(|| format!("parsing {} (file may be corrupted)", STORAGE_PATH))?;
    Ok(issues)
}

fn save_issues(issues: &[Issue]) -> Result<()> {
    let json = serde_json::to_string_pretty(issues)
        .context("serializing issues")?;
    fs::write(STORAGE_PATH, json)
        .with_context(|| format!("writing {}", STORAGE_PATH))?;
    Ok(())
}

fn next_id(issues: &[Issue]) -> u32 {
    issues.iter().map(|i| i.id).max().unwrap_or(0) + 1
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
    if title.trim().is_empty() {
        return Err(anyhow!("title cannot be empty"));
    }
    let mut issues = load_issues()?;
    let id = next_id(&issues);
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
    println!(
        "{}Created{} issue #{}: {}",
        GREEN, RESET, issue.id, issue.title
    );
    issues.push(issue);
    save_issues(&issues)?;
    Ok(())
}

fn cmd_list(
    status_filter: Option<Status>,
    priority_filter: Option<Priority>,
    label_filter: Option<String>,
) -> Result<()> {
    let issues = load_issues()?;

    // Default status filter is Open if none specified (preserves layer 2 default).
    let effective_status = status_filter.unwrap_or(Status::Open);
    let normalized_label = label_filter.as_ref().map(|s| s.trim().to_lowercase());

    let mut matches: Vec<&Issue> = issues
        .iter()
        .filter(|i| i.status == effective_status)
        .filter(|i| match priority_filter {
            Some(p) => i.priority == p,
            None => true,
        })
        .filter(|i| match &normalized_label {
            Some(label) => i.labels.iter().any(|l| l == label),
            None => true,
        })
        .collect();

    matches.sort_by_key(|i| (Reverse(i.priority), i.id));

    if matches.is_empty() {
        if issues.is_empty() {
            println!("No issues yet. Create one with:");
            println!("  tracker create \"your first issue\"");
        } else {
            // Specialize the empty state when filtering for open issues with no other filters
            // — this is the most common "everything is done" case.
            let no_other_filters = priority_filter.is_none() && normalized_label.is_none();
            if effective_status == Status::Open && no_other_filters {
                println!(
                    "No open issues. {}Nice work!{} 🎉",
                    GREEN, RESET
                );
            } else {
                let mut parts = vec![format!("status={}", effective_status.label())];
                if let Some(p) = priority_filter {
                    parts.push(format!("priority={}", p.label()));
                }
                if let Some(label) = &normalized_label {
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
            format!(
                " {}{}{}",
                CYAN,
                issue
                    .labels
                    .iter()
                    .map(|l| format!("#{}", l))
                    .collect::<Vec<_>>()
                    .join(" "),
                RESET,
            )
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
    let issues = load_issues()?;
    let issue = issues
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow!("no issue with id #{}", id))?;

    println!("{}#{}{}  {}{}{}", BOLD, issue.id, RESET, BOLD, issue.title, RESET);
    println!("  status:     [{}]", issue.status.label());
    println!("  priority:   [{}]", issue.priority.colored_label());
    if !issue.labels.is_empty() {
        let tags = issue
            .labels
            .iter()
            .map(|l| format!("#{}", l))
            .collect::<Vec<_>>()
            .join(" ");
        println!("  labels:     {}{}{}", CYAN, tags, RESET);
    } else {
        println!("  labels:     {}none{}", DIM, RESET);
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
        println!("{}", desc);
    }
    Ok(())
}

fn cmd_status(id: u32, new_status: Status) -> Result<()> {
    let mut issues = load_issues()?;
    let issue = issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| anyhow!("no issue with id #{}", id))?;
    let old_status = issue.status;
    issue.status = new_status;
    issue.updated_at = Utc::now();
    println!(
        "{}Updated{} issue #{}: {} -> {}",
        GREEN,
        RESET,
        id,
        old_status.label(),
        new_status.label()
    );
    save_issues(&issues)?;
    Ok(())
}

fn cmd_delete(id: u32) -> Result<()> {
    let mut issues = load_issues()?;
    let pos = issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| anyhow!("no issue with id #{}", id))?;

    let title = issues[pos].title.clone();
    let prompt = format!(
        "Delete issue #{}: {}{}{}?",
        id, BOLD, title, RESET
    );
    if !confirm(&prompt)? {
        println!("Cancelled.");
        return Ok(());
    }

    issues.remove(pos);
    save_issues(&issues)?;
    println!("{}Deleted{} issue #{}.", GREEN, RESET, id);
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
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
            label,
        } => cmd_list(status, priority, label),
        Command::Show { id } => cmd_show(id),
        Command::Status { id, status } => cmd_status(id, status),
        Command::Delete { id } => cmd_delete(id),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Custom error printer: red label, then the chain of contexts.
            eprintln!("{}error:{} {}", RED, RESET, e);
            for cause in e.chain().skip(1) {
                eprintln!("  {}caused by:{} {}", DIM, RESET, cause);
            }
            ExitCode::FAILURE
        }
    }
}