# Reliability Fix Specification

Scope: the four review findings and the market-suffix parsing issue.
Tier 3: session concurrency and HTTP contract changes.
Spec approval: not obtained (autonomous run under the user's instruction to fix
the reviewed issues). No separate spec review has taken place.

## Acceptance Criteria

1. JSON 401 and 403 responses with description `Forbidden` refresh credentials
   and retry once. A second rejection ends the request; 429/500 do not refresh.
2. Eight simultaneous requests using an expired session create exactly one new
   session. Cookies and crumbs stay paired for requests using either session.
   Cold concurrent requests also share initialization. Session bootstrap failure
   is recoverable by a later request.
3. `GET /quotes?symbols=AAPL&symbols=MSFT` returns HTTP 400 and JSON
   `{message: "Invalid query parameters", error: "Bad Request", statusCode: 400}`
   without invoking the provider. Missing and blank input retain their existing
   message and status.
4. The real binary responds to a local HTTP request and exits successfully on
   SIGTERM as well as SIGINT. Existing requests drain when Axum shuts down.
5. `BRK.A`, `BRK.B`, and `BF.A`, `BF.B` are explicit legacy aliases. Other
   dotted symbols pass through (including `7203.T`, `VOD.L`, `0700.HK`).
   Whitespace and empty entries are removed as before; normalization is
   idempotent and preserves order and duplicates.

## Failure Model and Invariants

- Lost auth status: local HTTP error fixtures, bounded retry assertions.
- Duplicate refresh or cookie/crumb races: synchronized concurrent requests and
  isolated-session regression assertions, repeated runs.
- Inconsistent errors: router-level status/body/content-type assertions.
- Interrupted deployment requests: Unix subprocess signal tests and a draining
  request test, with bounded waits and child-process cleanup.
- Corrupted foreign symbols: explicit examples and generated invariant cases.
- Preserve existing 400/405/502 bodies, quote fields, filtering, CORS, HTTP/1.1,
  and stderr-only upstream diagnostics; retain and run the existing suite.
- Public `QuoteProvider` and `StockPricesService` signatures remain unchanged.

## Setup and Verification

Use existing Rust, Cargo, Axum, Reqwest and Tokio dependencies; add no packages.
Allow local loopback mock servers and test subprocesses, with no Yahoo requests.
Add regression tests, this spec, an evidence report and reproducible verification
scripts. Do not commit or deploy. Run formatting, tests, Clippy, deterministic
parser properties, concurrency repetitions and targeted mutation checks.
Record tool versions/source fingerprint and skipped verification layers.
