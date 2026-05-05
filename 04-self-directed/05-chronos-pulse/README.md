# chronos-pulse

A time API and an MCP server. A clock for agents.

Agents typically have no sense of when "now" is, or how much time has passed between messages. This gives them one. The time API returns the current time as JSON; the MCP server wraps it as a tool any [Model Context Protocol](https://modelcontextprotocol.io) client can discover and call — Claude, Cursor, Windsurf, anything that speaks MCP.

## Links

- **Live:** [time.chrysanthemum.dev](https://time.chrysanthemum.dev)
- **MCP endpoint:** `https://time.chrysanthemum.dev/mcp`
- **Source:** [github.com/skydeval/chronos-pulse](https://github.com/skydeval/chronos-pulse)

## Stack

| Component | Language | Role |
|---|---|---|
| time-api | Java 17 / Spring Boot | REST endpoint for current time |
| chronos-pulse | Python | Heartbeat / liveness pinger |
| mcp | TypeScript / `@modelcontextprotocol/sdk` | MCP server, stdio + streamable HTTP |
| Caddy | — | TLS termination, reverse proxy |
| Docker Compose | — | Orchestrates all three services |

## Why this is in the self-directed folder

This isn't a guild assignment. I asked an instance of Claude whether agents could perceive time, and the answer was "not really." so I built it a clock. First as a curl-able API, then as a proper MCP server so that any agent can discover natively.

I didn't write a design doc up front. I designed and built iteratively with Gemini, prototyping the time API and the MCP wrapper in conversation. Then I ran the resulting code through adversarial review with Claude; different model, no shared context. It surfaced the things the building model missed: a fake `@Scheduled` task that printed sync logs but did nothing, invalid timezones returning 500 instead of 400, the Spring app exposed on the public interface instead of bound to localhost, the heartbeat script with no signal handling so `docker compose down` would hang. The cross-model review caught what a single-model build couldn't. It's not the canonical VDD/IAR loop with a written design doc, but the underlying discipline is the same.