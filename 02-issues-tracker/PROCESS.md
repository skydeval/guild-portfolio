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

# Review 2

## Genuine new finds Round 1 missed
- #4: `sanitize_text` only handles CSI escapes (ESC [). OSC sequences (ESC ]) bypass and re-emit on every list. Also strips `\n`, breaking multi-line descriptions, which contradicts the design doc's "longer description."
- #5: `sanitize_text(input.trim())` is the wrong order — trimming before sanitizing leaves inner whitespace if a leading control char surrounds it. Sanitize first, trim second.
- #7: `cmd_list` lowercases the label filter input but doesn't run it through `sanitize_text`. Asymmetric with creation, so certain filter inputs would never match stored labels.
- #11: `save_storage` leaks `tracker.json.tmp` if `fs::rename` fails. Add explicit cleanup on error.
- #18: stderr ANSI is gated by `stdout().is_terminal()`. Wrong — error output should be gated by stderr's own TTY status. Redirecting stderr to a file with stdout still on the TTY produces escape codes in the log.
- #27 + #28: Known Issues list still mentioned NO_COLOR support and id reuse, both fixed in Round 1.
- #34: "Nice work! 🎉" reads as condescending in the wrong context (e.g., a project you just inherited that has zero issues). Soften.

All addressed in commits below.

## Partial — noted, partly addressed
- #22: Round 1 #50 marked "custom error" overstatement as a nitpick. Fair point on a re-read. Updating the wording in PROCESS.md from "custom error printer" implications to the literal "match arm in main".
- #41: Round 1's "Marked invalid" rationales were tonal as much as substantive. Re-read with intent to engage rather than dismiss.

## Marked invalid
- #1: `fs2` is unmaintained but functions correctly; `fs4` or std::File::lock is a refactor, not a bug fix. Bookmark for v1.1.
- #2: lock file pollution per directory. Already in `.gitignore`; README notes it. Adversary wants more docs; partial in spirit but no behavior change.
- #3: silent lock blocking. Real but rare in single-user practice; a `try_lock_exclusive` first with a "waiting…" message is polish, not bug-fix.
- #6: `to_lowercase` Unicode order-of-ops. The actual policy decision is "do we accept non-ASCII labels at all?" Design doc doesn't restrict; current behavior is consistent if not perfect. Defer.
- #8: `Status` missing Ord derive. Still future-feature, still not a current bug.
- #9: enum-order-of-declaration coupling on Priority. `debug_assert!` is free; not blocking. Defer.
- #10: fsync the parent directory. Adversary admits "nearly impossible to hit for a personal tool."
- #12: legacy migration silently rewrites. Could add a print, but the migration is one-time and the user notices their data is intact. Trivial UX, not a bug.
- #13: legacy migration doesn't validate id uniqueness. Real edge case for hand-edited or corrupted legacy files. Defer.
- #14: u32 overflow message half-measure. Adversary's right that it's awkward; will let it panic and remove the check, or simply leave as-is. Not blocking.
- #15: legacy timestamps lie via Utc::now() default. `Option<DateTime>` migration is bigger than the round-2 budget for a fix that affects only pre-round-1 data.
- #16: cmd_status no-op audit narrative critique. No actual bug.
- #17: re-opens #4. Counted once.
- #19: confirms Round 1 fix landed. Not a finding.
- #20 + #44: version still 0.1.0 after seven layers. Versioning policy is a writeup question, not a bug. Will document the policy.
- #21, #24, #25: paste-version errors on the adversary's side. README exists, Cargo.lock is committed, DESIGN.md says "current directory." Adversary did not see the actual files.
- #23: missing Cargo.toml metadata (description, license, repository, authors). `cargo publish` constraint doesn't apply; not publishing.
- #26: design doc doesn't reflect envelope/lock/tmp implementation choices. Real point. Defer to a v1.1 doc pass.
- #29: stdin TTY check in confirm. Real edge case but the safe-cancel default is fine; adversary's own admission.
- #30: `--status all` deferred. One-line fix; legitimately small. Could add. Will add if there's appetite; otherwise v1.1.
- #31: error context lacks absolute path. Stylistic. "reading tracker.json" is comprehensible.
- #32: `tracker labels` command to discover existing labels. Feature request, not a bug. Defer to v1.1.
- #33: SIGPIPE panic on `tracker list | head`. Real Rust gotcha but obscure for a personal tool. Defer.
- #35: `#label` looks like Markdown header. Stylistic. Invalid.
- #36: chmod 600 on storage. Adversary's right the fix is small. The threat-model decision is "should this be the default?" — leaning no for portability across filesystems that don't honor mode bits. Defer.
- #37: undo/archive for delete. Combined with edit, a v1.1 concern; deferred.
- #38: bump to edition 2024. Stylistic.
- #39: clippy/rustfmt config. Valid portfolio polish; defer.
- #40: CI config. Chapter 03 territory.
- #42: invalid:valid ratio (24:10) interpretation. Round 1's adversary did over-claim; partial fault on both sides. The ratio in Round 2 is similar.
- #43: "chainlink" name-drop without context for portfolio audience. Adding a one-line clarifier in PROCESS.md.

## Acknowledged but deferred (already in scope discussion)
- #1, #2, #6, #9, #12, #13, #15, #29, #30, #31, #32, #33, #36, #37, #39: see above; v1.1 candidates.
- #38, #40: out of scope for phase 2.

# Review 3

Round 3 was triaged against an explicit stopping rule: continue if valid finds ≥ reaches, stop otherwise. Round 3 returned 73 findings — higher than round 2's 44 — but only 4 met the "genuine bug worth fixing" bar, with ~50 reaches/invalid. The valid:reach ratio is the exit signal per chapter 02-the-methodology/01-how-we-build.md ("the harshest possible critic has run out of legitimate complaints"). Stopping after addressing the four genuine finds.

## Genuine new finds Round 2 missed
- #5: `Storage::schema_version` was deserialized but never enforced. A future tracker writing schema 2 with an incompatible shape would silently deserialize as garbage. Now refused with a clear "upgrade tracker" message.
- #7: `next_id` collision possible if a corrupted or hand-edited storage file has `next_id ≤ max(existing ids)`. Defense-in-depth: load_storage now ensures `next_id > max_id` after parse.
- #26: no way to list issues across all statuses. Real one-line gap deferred twice in earlier rounds. Added a StatusFilter enum with an "all" sentinel; storage-level Status stays clean.
- #39: --label filter on `tracker list` only accepted a single value, asymmetric with --label on `tracker create`. Now repeatable with AND semantics.

All addressed in commits below.

## Reaches and invalid
Round 3 raised 73 findings, of which the bulk fall into one of these patterns:

- Already-deferred items the prior rounds explicitly triaged: length caps (#16-19), edit/add/remove labels post-creation (#34-37), --yes flag (#30), stdin TTY check (#31), file permissions (#71-72), no tests (#69), no CI (#40 from round 2). The "deferred to v1.1" or "out of scope for chapter 02" rationales from rounds 1 and 2 still apply; revisiting them here doesn't change the answer.
- Style or polish: visual rhythm of brackets and color (#41-43), description wrapping (#28), label syntax `#` vs `@` (#35 from round 2), color allocation perf (#50). Not bugs.
- Edge-case sanitizer concerns: bounded escape consumption (#13-14), `--` as title (#24), commas in labels (#23). Real but obscure for a personal tool.
- Adversary-side errors: README claimed missing (#63), Cargo.lock claimed missing (#60), process log claimed to be the bookmark manager's (#70). All present and correct in the repo; the adversary appears to have lost or misread the artifacts.

The full triage is not enumerated in this section because the cost of itemizing 70 reaches outweighs the value. The pattern is captured above.

## Methodology note
Three adversarial rounds was deliberate. Round 1 found 12 valid out of 52 (23%). Round 2 found 10 valid out of 44 (23%). Round 3 found 4 valid out of 73 (~5%). The drop in valid-find rate combined with the rise in absolute count is the curve expected when consulting chapter 02. Stopping here.