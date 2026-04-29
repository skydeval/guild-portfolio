# Issue Tracker - Design Document

## Purpose
A personal issue tracker CLI for managing project tasks.
Single user, runs in the terminal.

## Features
1. Create an issue with a title and optional description
2. List all open issues
3. Mark an issue as "in progress" or "done"
4. Set priority on issues (low, medium, high)
5. Add labels to issues (bug, feature, etc.)
6. Filter the list by status, priority, or label
7. List closed issues separately
8. Delete an issue

## Technology
- Rust
- Data stored in a local JSON file in the project directory
- CLI interface using subcommands (like git: `tracker create "title"`, `tracker list`, etc.)

## Interface
- `tracker create "Fix the login bug" --priority high --label bug`
- `tracker list` (shows open issues, sorted by priority)
- `tracker list --status done` (shows closed issues)
- `tracker list --label bug` (filter by label)
- `tracker status <id> in-progress` (change status)
- `tracker status <id> done` (mark complete)
- `tracker show <id>` (show full details)
- `tracker delete <id>` (remove an issue)

## Out of Scope
- Multiple users or sharing
- Due dates or calendar integration
- Subissues or hierarchy (keep it flat for now)
- Time tracking
