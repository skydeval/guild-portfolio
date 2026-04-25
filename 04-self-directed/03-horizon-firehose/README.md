# horizon-firehose

**Repository:** [skydeval/horizon-firehose](https://github.com/skydeval/horizon-firehose)

## What is Horizon Firehose?

A Rust-based ATProto firehose consumer with CBOR/CAR decoding, per-relay
failover, and Redis stream publishing. It exists because I had an operational
need: my privacy-focused ATProto client suite was running a Python firehose
consumer, and I wanted a Rust-native replacement I could trust at scale.

This was my first ground-up Rust project — earlier work was porting two
existing productions, which eased me into the language; horizon-firehose was
the first time I built and shipped something in Rust from scratch. It's also
the project where I made my first open-source contribution to a repo I
didn't own: four upstream issues filed and one PR merged into [proto-blue](https://github.com/dollspace-gay/proto-blue),
the SDK horizon-firehose depends on.

It's running in production now alongside the Python consumer, in parallel
rather than as a cutover, until the Python codebase stabilises enough that
the switch is low-risk. See [RETROSPECTIVE.md](./RETROSPECTIVE.md) for the
honest long-form on how the build went.