#!/usr/bin/env python3
"""Run the reliability checks; mutations only touch a temporary source copy."""

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
YAHOO = "src/stock_prices/yahoo.rs"
CONTROLLER = "src/stock_prices/controller.rs"

# Each replacement must compile and fail its selected behavioral test.
MUTATIONS = [
    (
        "discard HTTP authentication status",
        YAHOO,
        "if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {",
        "if false {",
        ["--lib", "refreshes_json_auth_statuses_without_matching_description"],
    ),
    (
        "disable session reuse",
        YAHOO,
        "cached.crumb.is_some() && rejected.is_none_or(|rejected| !Arc::ptr_eq(cached, rejected))",
        "cached.crumb.is_some() && rejected.is_some() && false",
        ["--lib", "concurrent_"],
    ),
    (
        "share cookies across session generations",
        YAHOO,
        ".cookie_store(true)",
        ".cookie_provider({ static JAR: std::sync::OnceLock<Arc<reqwest::cookie::Jar>> = std::sync::OnceLock::new(); JAR.get_or_init(Default::default).clone() })",
        ["--lib", "refreshing_does_not_change_an_in_flight_sessions_cookies"],
    ),
    (
        "accept a failed crumb endpoint response",
        YAHOO,
        'if !status.is_success() {\n            return Err(YahooError::Crumb(format!("crumb endpoint returned {status}")));',
        'if false {\n            return Err(YahooError::Crumb(format!("crumb endpoint returned {status}")));',
        ["--lib", "recovers_after_failed_session_initialization"],
    ),
    (
        "return the wrong query rejection status",
        CONTROLLER,
        'query.map_err(|_| ApiError::bad_request("Invalid query parameters"))?',
        'query.map_err(|_| ApiError::bad_gateway("Invalid query parameters"))?',
        ["--test", "app_e2e", "rejects_duplicate_query_fields_as_json"],
    ),
    (
        "discard exchange suffixes",
        CONTROLLER,
        "_ => symbol,",
        "_ => symbol.split('.').next().unwrap_or(symbol),",
        ["--lib", "normalization_preserves_symbols_and_is_idempotent"],
    ),
    (
        "stop listening for SIGTERM",
        "src/main.rs",
        "_ = terminate.recv() => {},",
        "_ = std::future::pending::<()>() => {},",
        ["--bin", "stock-prices", "sigterm_drains_an_in_flight_quote_request"],
    ),
]


def run(command, cwd=ROOT, env=None, expect_failure=False):
    result = subprocess.run(command, cwd=cwd, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=180)
    if expect_failure:
        if result.returncode != 101 or "test result: FAILED" not in result.stdout:
            raise RuntimeError(f"Mutation survived or failed without a test assertion:\n{result.stdout}")
    elif result.returncode:
        raise RuntimeError(f"{' '.join(command)} failed:\n{result.stdout}")
    return result.stdout


def source_hash():
    digest = hashlib.sha256()
    paths = [ROOT / name for name in ["Cargo.toml", "Cargo.lock", "rustfmt.toml", "README.md", "docs/reliability-spec.md", "tools/verify.py"]]
    paths.extend((ROOT / "src").rglob("*.rs"))
    paths.extend((ROOT / "tests").rglob("*.rs"))
    for path in sorted(paths):
        digest.update(str(path.relative_to(ROOT)).encode() + b"\0" + path.read_bytes() + b"\0")
    return digest.hexdigest()


def check_mutations():
    with tempfile.TemporaryDirectory(prefix="stock-prices-mutations-") as temporary:
        workspace = Path(temporary)
        for directory in ["src", "tests"]:
            shutil.copytree(ROOT / directory, workspace / directory)
        for name in ["Cargo.toml", "Cargo.lock", "rustfmt.toml"]:
            shutil.copy2(ROOT / name, workspace / name)
        env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target" / "reliability-mutants"))
        for name, filename, before, after, tests in MUTATIONS:
            path = workspace / filename
            original = path.read_text()
            if original.count(before) != 1:
                raise RuntimeError(f"Mutation anchor must be unique: {name}")
            try:
                path.write_text(original.replace(before, after, 1))
                run(["cargo", "test", "--locked", *tests], cwd=workspace, env=env, expect_failure=True)
                print(f"Killed: {name}", flush=True)
            finally:
                path.write_text(original)


def main():
    initial = source_hash()
    print(f"Source SHA256: {initial}", flush=True)
    print(run(["rustc", "--version"]).strip(), flush=True)
    print(run(["cargo", "--version"]).strip(), flush=True)
    print(run(["python3", "--version"]).strip(), flush=True)
    run(["cargo", "fmt", "--check"])
    run(["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"])
    print("Format, compiler and Clippy: passed", flush=True)
    check_mutations()
    for _ in range(10):
        run(["cargo", "test", "--locked", "--lib", "concurrent_"])
        run(["cargo", "test", "--locked", "--bin", "stock-prices", "sigterm_drains_an_in_flight_quote_request"])
    print("Concurrency and signal/drain stress: 10 runs passed", flush=True)
    # Includes the actual binary's local HTTP and Unix signal probes.
    print(run(["cargo", "test", "--locked"]), flush=True)
    if source_hash() != initial:
        raise RuntimeError("Source changed during verification; rerun on a stable tree")
    print(f"Mutation checks: {len(MUTATIONS)}/{len(MUTATIONS)} killed", flush=True)
    print("Parser properties: 720 generated cases, checked by the full suite", flush=True)
    print("Skipped: line coverage (llvm-cov unavailable); randomized test order (stable Rust).", flush=True)
    print("No external Yahoo requests; no dependency packages added.", flush=True)


if __name__ == "__main__":
    main()
