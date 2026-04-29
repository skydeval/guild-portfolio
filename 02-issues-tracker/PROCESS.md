# What I built & why:

Built a personal issue tracker CLI in Rust as the phase 2 portfolio project. Tracks tasks in a local JSON file with status, priority, labels, descriptions, and timestamps.

# Process

1) presented design doc and initial prompt

prompt: i want to build the issue tracker from my design doc. start with `cargo new tracker` and add support for `tracker create "title"` and `tracker list`. save to a JSON file in the current directory. plain rust, single main.rs. use clap for argument parsing, serde for JSON, anyhow for error handling. issues should have an id and a title for now.

verified: clean cargo build, create + list both worked, tracker.json appeared in current directory and round-tripped correctly through restart

2) prompt: add a status field on each issue (open, in-progress, done). new issues default to open. add a `tracker status <id> <new-status>` command. `tracker list` should default to showing only open issues.

verified: status changes persisted to JSON, list filtered to open by default, layer 1 issues without a status field loaded as open via serde default attribute (no migration needed)

3) prompt: add priority levels (low, medium, high). `tracker create` takes an optional `--priority` flag, defaults to medium. list sorts by priority descending so high-priority issues appear first. show priority in the list output with ANSI color: red for high, yellow for medium, dim for low.

verified: --priority flag worked, sort put high-priority first, ANSI colors rendered in terminal, layer 2 issues without priority field loaded as medium via serde default

4) prompt: add labels to issues. `tracker create` takes a repeatable `--label` flag. labels render in cyan after the title in list output. add `tracker list --label <name>` to filter. normalize labels to lowercase and dedupe within a single issue (lesson from the bookmark manager's tag handling).

verified: --label repeats correctly, filter narrowed list to matching issues, normalization deduped "Bug" / "BUG" / "bug" into one entry per issue

5) prompt: extend `tracker list` with `--status` and `--priority` filters alongside the existing `--label`. all three should compose with AND semantics. default behavior (no `--status` flag) still shows open issues only.

verified: combined filters worked correctly, empty state names which filters are active so you can see what to relax

6) prompt: add `tracker show <id>` for full detail view including timestamps. add `tracker delete <id>` with a y/N confirmation prompt. add the description field that's been in the design doc but hasn't been implemented yet. status changes should update an updated_at timestamp. reject empty titles on create.

verified: show prints all fields including timestamps via chrono, delete prompts and respects N as cancel, status changes update updated_at, empty title rejected with clear error

7) prompt: polish pass. green for success messages (Created, Updated, Deleted). custom error that puts errors in red with the cause. warmer empty state: "No open issues. Nice work!" with the 🎉. richer --help long_about descriptions on every command with example invocations. bold the title in show and in the delete confirmation prompt.

verified: green/red colors render in terminal, --help shows quickstart + per-command examples, corruption error displays clean cause chain, no functional regressions from the polish

# What I learned

I've been using chainlink in projects for a while now without really understanding how it works underneath, just trusting that it kept things organized. Building a smaller version of an issue tracker myself made a lot of that make sense. I get now why the design choices in chainlink are what they are; why filtering by label vs status vs priority matters, why having a `show` command for the full detail vs a compact `list` are both needed.

The other thing this layer of the curriculum drove home: when you're working with AI, it's very easy to drift out of scope. The agent finds something interesting, follows that, and you end up shipping a feature you didn't set out to make or fixing a bug that wasn't on the docket. Having an issue tracker keeps the scope in view. Like having a list when you go shopping; without it you can easily wind up getting nothing you need and everything you didn't. But with one, you can get exactly what you needed and nothing more, easily.

This was also a much shorter and tighter project than I expected. Phase 1 was about learning to build with an agent. Phase 2 is about learning to keep the agent on track without getting lost in the sauce. The tracker itself is small, but the methodology lesson it carries is integral.

# Known Issues
- no way to edit title or description after creation
- hardcoded colors - no --no-color flag or NO_COLOR env var
- no pagination if you add many issues
- sequential numeric IDs reused after delete (delete the highest-numbered issue, next create reuses that id)