# Aurora-Locus v0.2 cycle

The biggest thing I've shipped under doll's repo so far. v0.2 covers Postgres backend support, S3 blob storage activation, the admin/moderation parity floor + extension surface (`tools.aurora.{moderator,admin,superadmin,ops}.*`), and a complete admin UI overhaul — 28 pages, 21 substrate primitives, hash-chained audit log, forensic export with chain-of-custody, real-time event subscription, runtime settings infrastructure.

126 commits. Pushed at 8:20 AM. Merged by doll at 9:46 AM.

## Links

- **PR:** [dollspace-gay/Aurora-Locus#7](https://github.com/dollspace-gay/Aurora-Locus/pull/7)
- **Branch:** `v0.2` → `main`
- **Working fork:** [github.com/skydeval/Aurora-Locus](https://github.com/skydeval/Aurora-Locus)

## How it actually got built

Four design assessments before any code: `ADMIN_MODERATION_ASSESSMENT.md`, `POSTGRES_BACKEND_ASSESSMENT.md`, `BLOB_STORAGE_S3_ASSESSMENT.md`, and `docs/AURORA_ADMIN_UI_DESIGN.md` (the UI doc alone is ~42K words across 14 sections). The assessments named the parity gaps, the extension surface, the phasing, and the contracts. The work that followed derived from them rather than improvising endpoint shapes mid-implementation.

The actual coding is AI-assisted with as little human-typed code as possible. Web Claude for architecture conversations, design doc drafting, kickoff prompts, and triage of CC reports. Claude Code for every code edit, commit, and file move. I wrote prompts in markdown code blocks, pasted them into CC, pasted CC's reports back into web Claude. Six phases plus a rate limiter exemption plus six smoke-surfaced bug fixes plus the cycle-end audit work, all through that loop.

## What the design doc bought

The doc was load-bearing. When CC ran an autonomous overnight session against the spec, it shipped four full phases — emitEvent + batch operations, getModerationMetrics + getQueueStats, getAuditTrail + exportAccountForensic + audit chain core, subscribeModEvents, runtime settings — without me in the loop. The discipline that made that safe was the doc itself: every endpoint specified as Rust types, every substrate primitive specified with its accessibility contract, every phase with a "done when" milestone. CC had answers to design questions without needing to wake me up.

## Cycle-end audits

The design doc commits to three audits before merge.

**Decoupling sweep** ran clean. No references to my other projects' names in code, comments, identifiers, or documentation. The discipline held across 200+ commits because it was named in the architecture principles up front.

**Functional verification** was the browser smoke test. Three rounds of bug fixes came out of it: a serde flatten + u32 deserialization issue affecting five query handlers, a session role display bug where the UI read from `getSession` instead of localStorage, a Roles page that expected pre-grouped data the server returned flat, and three more in a second round (real-time indicator stuck reconnecting, capabilities advertising only 2 of 16, an action endpoint registered as GET when the client sent POST). All real bugs the smoke caught.

**Adversarial review** was the most useful audit. I spun up a fresh Claude instance with no conversation history, gave it the repo and the design doc, and asked it to find where the implementation diverged from spec and where it would attack. The first pass surfaced 36 confirmed-real findings — including audit chain integrity gaps (per-row verification only, no transitive check), forensic export hash that only covered the manifest not the bundle contents, a router fragment XSS, DPoP validation as a stub that always returned Ok, OAuth scope buckets misaligned with §8 commitments, and three production-critical tests that codified the divergent implementation rather than catching it.

I ran the adversarial review four more times. Each round used a fresh instance, the same prompt, and the repo as it stood after the previous round's fixes had landed. The first pass found 36; subsequent rounds found progressively fewer confirmed-real findings. Five passes was thorough, but given the scope of this project I needed to eliminate load-bearing bugs specifically, and when I reached a report with zero I knew the audits had been warranted. When the next pass also found zero load-bearing bugs, I knew I didn't need to keep running them.

The fresh-instance review caught what the building instance couldn't because the building instance had been rationalizing gaps along the way. The fresh instance had no investment in any of the choices. Same pattern as the cross-model review on the chronos-pulse work — different context, different blind spots.

## What I'd do differently

Write the UI design doc before starting the build, not halfway through. I had the four other assessments up front and that's why the server-side work shipped clean. The UI work didn't have an equivalent doc until midway through implementation, which meant the substrate decisions and the page contracts were getting made inline with the build, then retrofitted into a doc, then retrofitted further when smoke testing surfaced what didn't compose. Sequencing the doc first would have eliminated most of the round-1 and round-2 smoke bugs — they were almost all symptoms of UI/server contracts that hadn't been pinned down before the code got written.

The other half of the lesson: when I did finish the UI design doc, I didn't adversarially review it before building against it. My normal loop is design → attack the design → implement. I skipped the middle step on the UI doc and the gaps it left came back to bite me during the adversarial review pass — the audit chain integration gap on batch endpoints, the forensic export hash coverage, the OAuth scope misalignment with §8 commitments. All of those are things an adversarial pass on the doc would have surfaced before any code shipped.

## Why this is in self-directed

Doll's repo is a Rust ATProto PDS. v0.2 is the apprentice work that informs the path forward for the moderation ecosystem I'm building separately. Doll merging it within 86 minutes of the push signals not only that the code works, but that the design discipline holds at the size where I want to do my own original work.