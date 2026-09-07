# Repository Guidelines

## Project Structure & Module Organization

呢個 repo 係用 Rust + Axum 寫嘅股票報價 API。Runtime source code 放喺 `src/`，binary 入口係 `src/main.rs`，library root 係 `src/lib.rs`，router wiring 喺 `src/app.rs`。Root endpoint 喺 `src/app_controller.rs`，錯誤轉換喺 `src/error.rs`。股票報價功能集中喺 `src/stock_prices/`，包括 controller、service、types 同 Yahoo Finance client。Integration tests 放喺 `tests/`。Build output 會產生喺 `target/`，唔好直接改。`api/axum.rs` 係 Vercel Function entrypoint，佢只係包一層 `VercelLayer` 再重用 `create_app()`——router 或 controller 有改動，唔需要同步改佢。

## Build, Test, and Development Commands

- `cargo build`: compile debug binary。
- `cargo build --release`: compile optimized binary 到 `target/release/stock-prices`。
- `cargo run`: 跑 server，預設係 `http://localhost:3000`。
- `PORT=3100 cargo run`: 用自訂 port 本地開發。
- `cargo test`: 跑 unit tests 同 integration tests。
- `cargo fmt`: 用 rustfmt format 全部 code（settings 喺 `rustfmt.toml`）。
- `cargo clippy --all-targets`: 跑 linter。

## Coding Style & Naming Conventions

跟返 repo 現有 Rust 寫法。Feature code 按 module 分組，例如 `stock_prices/controller.rs`、`stock_prices/service.rs`。Module 同 file name 用 `snake_case`，type 用 `PascalCase`。Indentation 同 line width 由 `rustfmt.toml` 決定（4 spaces、`max_width = 200`）。API boundary 優先用明確 `pub` struct、trait 同 type alias。Controller 保持薄身；Yahoo Finance access 放喺 `yahoo.rs`，quote normalization 放喺 `service.rs`。

Controller 依賴 `QuoteProvider` trait 而唔係 concrete service，方便 test 換 stub。

## Testing Guidelines

Unit tests 用 `#[cfg(test)] mod tests`，放喺被測 code 同一個檔案。Integration tests 放喺 `tests/app_e2e.rs`，用 `tower::ServiceExt::oneshot` 直接打 Axum `Router`。Tests 唔應該出網——用 `QuoteProvider` stub 代替真 Yahoo Finance。改 request parsing、response shape、error handling 或 Yahoo Finance mapping 時，要同步加或更新 tests。

## Commit & Pull Request Guidelines

近期 commit 用短 conventional-style prefix，例如 `chore:`、`docs:`、`refactor:`。Commit subject 要具體同帶動作，例如 `fix: handle blank quote symbols`。Pull request 要有簡短描述、已跑嘅 test commands、相關 issue link；如果 API behavior 有變，要附 sample request 或 response。

## Security & Configuration Tips

唔好 commit secrets 或本地 environment files。呢個 API 依賴 Yahoo Finance response，所以 upstream failure 要 defensive 咁處理；除非係有意改 API contract，否則要保留現有 `400`、`405`、`502` 行為。Upstream error 嘅 cause chain 只可以 log 去 stderr，唔可以入 response body。

Yahoo `v7/finance/quote` 要 cookie + crumb session，`yahoo.rs` 已經處理埋 cache 同 refresh；佢亦特意 pin HTTP/1.1，因為 Yahoo edge 會斷我哋嘅 HTTP/2 stream。
