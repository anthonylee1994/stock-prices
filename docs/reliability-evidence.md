# Reliability Fix Evidence

- Specification: [reliability-spec.md](reliability-spec.md), Tier 3.
- Spec approval: not obtained (autonomous run). The user authorized fixing the
  review findings; the acceptance specification has not had independent review.
- Reproduce: `python3 tools/verify.py` from the repository. Requires Unix for the
  signal checks, Python 3 and the existing Rust toolchain. Mutants run in a
  temporary source copy with a separate Cargo target directory.
- Verified source SHA256:
  `847573f11311e4634128ba284b0629708044cc81017ab951e5352bfa55b79b8e`.
  The entry point computes this from the source, tests, manifest/lockfile,
  formatter settings, README, specification and verification script.
- Toolchain: rustc 1.95.0 (59807616e 2026-04-14), cargo 1.95.0
  (f2d3ce0bd 2026-03-21), Python 3.14.5; run on macOS.

## Acceptance Results

| Behavior | Test / evidence | Result |
| --- | --- | --- |
| JSON 401/403 refresh despite `Forbidden` description | `refreshes_json_auth_statuses_without_matching_description` | Pass |
| Retry at most once; ordinary 429/500 errors do not refresh | `stops_after_one_retry_and_does_not_retry_other_statuses` | Pass |
| Eight expired requests share one replacement | `concurrent_expired_requests_share_one_refresh` | Pass: 2 sessions total, 17 quote calls including warmup |
| Eight cold requests share initialization | `concurrent_cold_requests_share_initialization` | Pass: 1 session, 8 quote calls |
| Cookie/crumb pair survives another request's refresh | `refreshing_does_not_change_an_in_flight_sessions_cookies` | Pass |
| Bootstrap failure permits a later recovery | `recovers_after_failed_session_initialization` | Pass |
| Duplicate query fields produce the specified JSON 400 without provider access | `rejects_duplicate_query_fields_as_json` | Pass |
| Missing/blank input, 405, 502, CORS and metadata remain compatible | Existing `tests/app_e2e.rs` tests | Pass |
| Foreign suffixes and the four explicit class aliases | `preserves_market_suffixes_and_normalizes_legacy_aliases` | Pass |
| Order, duplicates, trimming, alias mapping and idempotence | `normalization_preserves_symbols_and_is_idempotent` | Pass: 720 generated input combinations |
| SIGTERM drains an active quote request | `sigterm_drains_an_in_flight_quote_request` | Pass |
| Actual binary serves HTTP and exits successfully on SIGTERM/SIGINT | `serves_http_and_exits_cleanly_on_sigterm_and_sigint` | Pass |
| Quote mapping, timestamps, optional fields and delisted filtering | Existing mapping tests and mock HTTP quote tests | Pass |
| Public provider/service signatures, HTTP/1.1 and stderr-only upstream diagnostics | Source diff review; existing response assertions | Preserved; no automated ABI/log capture check |

## Final Verification Run

| Layer | Result |
| --- | --- |
| Full `cargo test --locked` | 28 passed: 16 library, 1 binary unit, 11 integration; 0 failures |
| `cargo fmt --check` | Pass |
| `cargo clippy --locked --all-targets -- -D warnings` | Pass, includes compilation/type checking |
| Targeted mutations | 7/7 killed by behavioral tests, including the parser mutant against the property test alone |
| Suite health / concurrency | Both concurrent tests and the signal/drain test passed in each of 10 repetitions |
| Real execution | Actual compiled binary: local HTTP 200 plus successful SIGTERM/SIGINT exit |
| Adversarial inputs | Duplicate query fields, Unicode whitespace, multiple dots, foreign suffixes, repeated symbols, rejected auth and failed bootstrap exercised |
| Dependencies / capability review | No dependency or lockfile changes, no credentials added; test tooling adds loopback servers and local test subprocesses |

## Limits and Run History

- Changed-line coverage was not measured: cargo-llvm-cov is not installed.
- Randomized ordering and ThreadSanitizer/loom were not run. Stable Rust tests,
  synchronized interleavings and 10 stress repetitions were used instead.
- The property test is a deterministic 720-case matrix, not an unbounded fuzzer.
- No live Yahoo, Linux container or Windows run was performed. The Unix signal
  behavior was exercised on macOS. No dependency audit was run because the
  dependency set did not change.
- No latency budget, backoff or rate limiting was added. Bootstrap still holds
  the session write lock during network access; HTTP timeouts do not bound lock
  waiting. Failed refresh attempts are not shared as cached failures.
- The initial RED run observed 7 failing tests reproducing the review findings.
  Cold initialization and bootstrap recovery already passed; mutations that
  disable caching and accept failed bootstrap responses verified their checks.
- The first verification-script run reused Cargo artifacts between temporary
  mutants and the normal build, causing a stale mutant binary to run during
  stress checks. The script now isolates mutant build output. Package build
  artifacts were cleared, and the entire final run above passed afterwards.
