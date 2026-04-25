# cairn-mod

**Repository:** [skydeval/cairn-mod](https://github.com/skydeval/cairn-mod)

## What is Cairn Mod?

Cairn-mod is a standalone ATProto labeler server. It publishes a
[`app.bsky.labeler.service`](https://atproto.com/lexicons/app-bsky-labeler)
record, signs labels per the ATProto spec, accepts user reports, and
exposes an admin XRPC surface for moderators to act on them. It
exists because the ecosystem has Ozone (heavy, TypeScript,
Postgres-backed, opinionated web UI) and Skyware's labeler library
(minimal, no report intake, no audit trail), with a gap between them
for operators who want something compact but production-grade. Cairn-mod
is deliberately smaller than Ozone and deliberately more complete
than Skyware; it does not try to be either.