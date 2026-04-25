# anarchy.lgbt registry

**Repository:** [skydeval/anarchy-registry](https://github.com/skydeval/anarchy-registry) · **Live:** [anarchy.lgbt](https://anarchy.lgbt)

A custom Bluesky handle provider for the queer internet. Users claim a
subdomain like `yourname.anarchy.lgbt`, bind it to their Bluesky DID, and
manage it with a secret key. No accounts, no email, no recovery; just a
key and a handle. This is the Rust port of an October 2025 Cloudflare
Worker that ran six months in production.

The port is the project where I learned to think about threat models
rather than checklists. User tokens have 119 bits of entropy, so SHA-256
would be cryptographically fine, but the admin password is human-chosen,
so it gets argon2id with tuned cost parameters. Hash comparisons are
constant-time. The Worker returned distinguishable status codes for
different failure modes (401, 403, 429); the port returns identical 404s
for everything an attacker could probe. Blocked registrations are
indistinguishable from legitimately-taken handles. The admin path returns
404 for wrong passwords. An attacker scanning the service learns nothing
about what's behind any endpoint.

The full list of security improvements, along with the architectural ones
(a single 3000-line JavaScript file with duplicate function definitions
became a layered Rust crate with isolated modules), is in the [repo's README](https://github.com/skydeval/anarchy-registry).
The design doc that preceded the code went through two adversarial review
passes covering 17 attacks before any production code was written. The
service supports 21 pride themes covering pride flags, mental health
awareness flags, and plural/system flags. The project was queer-specific
infrastructure from day one, not a generic registry with rainbow paint.