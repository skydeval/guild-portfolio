# Workshop: proto-blue contributions

This entry documents my Phase 3 / workshop contribution: four issues and a merged PR to [`dollspace-gay/proto-blue`](https://github.com/dollspace-gay/proto-blue), the Rust SDK my horizon-firehose project depends on. Confirmed with doll that this counts as the workshop signal for the Navigators Guild apprentice path.

## What proto-blue is

proto-blue is a Rust SDK for the AT Protocol, written by doll (`dollspace-gay`). It's a workspace of crates covering CBOR/CAR encoding, WebSocket transport for the firehose, repo decoding, lexicon validation, and HTTP/XRPC clients. It's the foundation horizon-firehose builds on — without proto-blue I'd have written the CBOR decoder and firehose connection from scratch.

## How the contribution happened

I wasn't looking to contribute to proto-blue. I was building [horizon-firehose](https://github.com/skydeval/horizon-firehose), which is its own portfolio project (a Rust-based ATProto firehose consumer with multi-relay failover and Redis publishing). proto-blue was the dependency that made horizon-firehose feasible at all.

Building against an early version of someone else's SDK surfaces real friction. I kept hitting cases where proto-blue almost did what horizon-firehose needed but didn't quite expose the right knob. Each time, I had a choice: hack around it locally, or file an issue upstream and let the SDK get better for everyone. I filed.

## The four issues

I asked doll first whether I could even depend on proto-blue. doll said yes. The contribution loop then emerged organically as horizon-firehose's adversarial reviews surfaced limitations.

1. **`WebSocketKeepAlive::recv` pins to a single URL.** The SDK's keep-alive wrapper has an internal reconnect loop that re-binds to the original URL on every retry, which makes multi-relay failover impossible from the consumer side. My workaround was wrapping `recv` with a 60-second `read_timeout` and forcing the outer supervisor to re-evaluate. Filed with a clear repro and a request for either a lower-level API or a configurable per-attempt timeout.

2. **`WebSocketConfig` not exposed.** proto-blue-ws wraps `tokio-tungstenite` but doesn't surface its `WebSocketConfig`, so consumers can't set `max_message_size` at the wire level. horizon-firehose enforces a 5MB cap (the AT Protocol firehose spec maximum) at the decoder layer instead, but defense-in-depth would be wire-level rejection before bytes are ever decoded. Filed as a feature request.

3. **TLS `ClientConfig` not exposed.** Same shape as the previous one: proto-blue-ws builds its own TLS connector internally, so consumers can't add custom CA roots. horizon-firehose has a `tls_extra_ca_file` option that was inert-with-WARN until this was addressed. Bundled into a discussion with #2 around general "expose connection configurability."

4. **Varint overflow in `read_car`.** Different crate, separate filing — `proto-blue-repo`'s CAR reader decodes a varint-prefixed length without bounds checking, which can panic on malformed input. Suggested `checked_add` plus a cap against remaining buffer length. horizon-firehose has a `catch_unwind` wrapper as defense-in-depth, but the actual bug shouldn't trigger anymore.

## The PR

[**Pull request #5**](https://github.com/dollspace-gay/proto-blue/pull/5) — widened the `WebSocketTransport` trait bound from `Send` to `Send + Sync`. One-line fix. horizon-firehose needed it because the supervisor task holds a transport across an `await` point on a shared state, which requires `Sync` for the future to be `Send`. Without the bound, the consumer code wouldn't compile.

doll merged it in under 30 minutes.

## What doll shipped

doll responded by upgrading proto-blue from 0.1 (the SHA I'd been pinned to) to 0.2.4, addressing the cluster of issues:

- Added SDK-native `per_recv_timeout_ms` and `max_reconnect_attempts` config, closing issue #1 above. horizon-firehose dropped the `read_timeout` workaround in favor of native config.
- Exposed `WebSocketConfig`, closing #2.
- Exposed enough of the TLS `ClientConfig` to make `tls_extra_ca_file` actually plumb, closing #3.
- Patched the varint overflow upstream, closing #4.

Plus PR #5 merged.

## What I learned

This contribution loop felt different from my Phase 1 and Phase 2 work in a specific way. Those phases had me working alone in my own codebase — every decision was mine, every constraint was self-imposed. proto-blue forced the opposite: I could see a place I wanted the SDK to behave differently, and the path to that wasn't "rewrite it" but "make a clear case to someone else and trust them with the design." The fix was always partly out of my hands.

That changed how I wrote the issues. Each one had to stand on its own — a maintainer reading them cold needed to understand the problem, the workaround I was using, why the workaround wasn't sufficient long-term, and what shape the fix could take. I wasn't writing them for me; I was writing them so doll could decide quickly whether and how to act.

The PR was its own thing. A one-line trait change is small in scope but felt large in courage — it's the first time I've put code into another developer's repo and asked them to trust it. The under-30-minute merge was reassuring in a way that's hard to overstate. The issues had been incoming bug reports; the PR was outgoing repair, and it landed.

## Artifacts

- proto-blue repo: https://github.com/dollspace-gay/proto-blue
- PR #5: https://github.com/dollspace-gay/proto-blue/pull/5
- horizon-firehose (the consumer project that drove these contributions): https://github.com/skydeval/horizon-firehose