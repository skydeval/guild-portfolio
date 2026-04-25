# cairn-mod

**Repository:** [skydeval/cairn-mod](https://github.com/skydeval/cairn-mod) · **crates.io:** [cairn-mod v1.0.0](https://crates.io/crates/cairn-mod)

Cairn-mod is a standalone ATProto labeler server. It publishes a
`app.bsky.labeler.service` record, signs labels per the ATProto spec,
accepts user reports, and exposes an admin XRPC surface for moderators
to act on them. It exists because the ecosystem has Ozone (heavy,
TypeScript, Postgres-backed, opinionated web UI) and Skyware's labeler
library (minimal, no report intake, no audit trail), with a gap between
them for operators who want something compact but production-grade.
Cairn-mod is deliberately smaller than Ozone and deliberately more
complete than Skyware; it does not try to be either.

v1.0 shipped 26 hours of focused work over 2.5 days, idea to crates.io.
That timeline was only possible because the methodology was real: a full
design document with five review rounds before any code, every issue
tracked before work started in [doll's chainlink](https://github.com/dollspace-gay/chainlink),
every session scoped to a single issue, 297+ tests including byte-exact
crypto parity against `@atproto/crypto`. The §14.1 production checklist
walkthrough caught three release-blocking bugs that the test suite missed,
which is the lesson the [retrospective](RETROSPECTIVE.md) is built around.

v1.0 is a correct foundation, not a feature-complete Ozone alternative.
The trajectory toward Ozone-alternative status continues over subsequent
releases.