# TypeSafe adapter

Bounded Rust client for [TypeSafe's evaluation API](https://docs.typesafe.ai/api).
The launcher passes a query and locally executable candidates; Jev returns a score
for each candidate. This crate has no Omega or desktop dependency and executes no
actions.

`Client::from_env()` reads `TYPESAFE_API_KEY`. `rank` uses `jev-1.13.0`, a three-level
relevance rubric, a three-second timeout, no redirects, and no automatic retries.
Responses must match the requested model and answer set, with finite scores in
0–2. Unknown candidates cannot enter the returned ranking.

Run `cargo test -p typesafe` for request/response contract fixtures. See the
[launcher guide](../../plugins/launcher/README.md#optional-semantic-search) for
configuration, disclosure of sent metadata, limits, and behavior on failure.
