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

# Review 1

## Genuine valid finds
- #1 + #2: atomicity. tracker.json writes are not atomic — concurrent invocations can clobber each other, and a kill mid-write corrupts the file. addressed via write-to-tmp + rename + advisory file lock.
- #3 + #4: "Created" / "Updated" prints before save_issues, so a save failure shows a success message followed by a red error and no actual save. moved println after save.
- #9: title not trimmed before storage. `tracker create "  foo  "` stored leading/trailing whitespace. fixed.
- #10: control characters and ANSI escapes accepted in titles. piping a list to a log file would write raw escape codes. stripped on input.
- #15: setting a status to its current value still bumped updated_at and printed "Updated #1: done -> done". now a no-op with a clear message.
- #20 + #21: hardcoded ANSI with no NO_COLOR support, no isatty detection, no --no-color flag. this was already in known issues; addressed now.
- #22: priority shown as "med" in output but accepted as "medium" on input. reconciled.
- #26: show command prints description with no separator from the metadata block. added a header.
- #45: no README in the issue tracker folder. added.
- #46: clap doesn't expose --version. trivial enable.

All addressed in commits below.

## Design doc drift
- #33: design doc said "List closed issues separately" but implementation just uses --status. updated doc to reflect actual behavior.
- #34: design doc implies dedicated "mark complete" affordance; only tracker status <id> done exists. updated doc.
- #35: design doc said "the project directory" — implementation uses CWD. updated doc to say "current directory".

## Stable IDs
- #5: next_id derived from max-existing-id, so deleting the highest issue causes id reuse. this was in known issues. addressed by storing next_id in the JSON envelope and never reusing.

## Partial — noted but not fixed
- #6: sort tiebreaker is id ascending within priority. adversary prefers updated_at; mine is defensible (oldest open issue surfaces first within a priority bucket). not blocking.
- #16: no add/remove labels post-creation. extension of the editing gap already in known issues; deferred to v1.1.
- #17: tracker.json path is CWD-relative. real but in known issues already; --file flag and XDG paths are v1.1.
- #18: no chmod 600 on the file. relevant for sensitive task notes; design doc's trust model is single-user-local. added a sentence to the design doc; chmod is deferred.
- #27: show duplicates timestamps when never modified. cosmetic; defer.
- #28: filtered empty-state message includes the implicit status=open default. minor UX confusion; defer.
- #29 + #30: no --status all / list valid IDs on error. real gaps; v1.1.
- #31: parse error has no recovery instruction. one-line fix; will pick up in next polish pass.
- #37: default_now for legacy data assigns Utc::now() to issues that loaded without timestamps. one-time event per user; cleaner with Option<DateTime> but practical impact is small.
- #39: to_lowercase vs to_ascii_lowercase. only matters with non-ASCII labels we don't strictly disallow yet. small.
- #43: ANSI helper function instead of inline format!. partly addressed by the NO_COLOR fix; full refactor is style.
- #51: plaintext JSON with no integrity check. added a trust-model sentence to design doc; integrity check is out of scope for v1.

## Marked invalid
- #7: length limits on title/description/labels. design doc has none; 50MB description is a self-inflicted edge case for a personal tool, not a bug.
- #8: regex restriction on label content. "spaces in labels are ugly" is opinion; design doc allows free-form labels.
- #11: warn on duplicate title. "nice to have," not a bug.
- #12: EOF on stdin in confirm() returns Ok(false). non-interactive use isn't a stated goal.
- #13: --yes flag. scripting affordance not in design doc.
- #14: tracker status naming convention. design doc literally specifies "tracker status <id> in-progress"; the adversary's preference is taste.
- #19: backup before destructive ops. complexity, not insurance.
- #23: column width assumes small IDs. 9999 issues in a personal tracker is implausible.
- #24: ANSI in column padding. priority labels are all 3-4 visible chars currently; not breaking.
- #25: no result count in list. nice-to-have, deferred to UX polish if it becomes annoying.
- #32: error context is verb-only ("reading tracker.json"). reads fine in practice.
- #36: Status missing Ord derive. not a current bug; future-feature concern.
- #38: BTreeSet alphabetizes labels. intentional, lesson from the bookmark manager.
- #40 + #49: no automated tests. chapter 02 doesn't introduce tests; chapter 03/05-shipping-it does.
- #41: find_issue helper. adversary's own words: "fine for this size."
- #42: Issue::new constructor. stylistic.
- #44: confirm() case handling. "Yes please" being accepted is fine.
- #47: no LICENSE file. curriculum doesn't require it; bookmark-manager subdirectory doesn't have one either.
- #48: "What I learned" conflates lessons. disagree — the two paragraphs are different angles on the same point: the technical problem (drift) and the meta-pattern (phase 2's role in the curriculum).
- #50: "custom error" is an overstatement of "custom error printer". semantic nitpick.
- #52: path traversal on configurable storage path. depends on #17 which is deferred.