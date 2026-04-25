# goodgirls.onl registry

**Repository:** [skydeval/goodgirls-registry](https://github.com/skydeval/goodgirls-registry) 
**Live:** [goodgirls.onl](https://goodgirls.onl)

A custom Bluesky handle provider — Rust port of an October 2025 Cloudflare
Worker. The port took 8 hours of build time but is preceded by a design
document with five rounds of adversarial review and 53 findings; every
implementation decision traces to a specific weakness in the original.

What that translates to in code: the original Worker had handle theft via
the manage endpoint (any authenticated user could overwrite any subdomain),
unlimited handle accumulation per DID, premature secret deletion that
orphaned other handles, race conditions on registration (Workers KV has no
transactions), zero input validation, no logging, and zero tests. The
Rust port closes all of these — ownership verification, one-handle-per-DID
constraint, sqlite with `BEGIN IMMEDIATE` transactions, full input validation
with reserved-word and length checks, structured `tracing` logs, and 70 tests
(57 unit + 13 integration). The full decision log is in the repo's
[DESIGN.md](https://github.com/skydeval/goodgirls-registry/blob/main/DESIGN.md).

This is the project where I went hardest on the methodology side. Five
adversarial review rounds is more than the curriculum asks for; I stopped
when reviewers were finding more reaches than real issues. It's also the
project where I committed to WCAG 2.2 AA compliance with documented
intentional non-coverage of the two AAA criteria I couldn't reach without
hurting the design.