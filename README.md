# LiteTUI Sidecar — native presentation preview

This independent local Rust repository derives its native `tao`/`wry` window foundation from the Tempo starter at `C:/Projects/LiteTUI/Docs/example-apps/test-1`. It currently renders a **visual-only** interpretation of the approved Oscura Midnight mockup (`C:/Projects/light-mock-up/src/app/litetui-sidecar`). Timeline, week/month, settings categories, Browser/Artifacts navigation and dialogs are interactive samples, but nothing persists, schedules, executes, downloads, or browses externally. The standalone window is **not connected to LiteTUI**. A private stdin/stdout JSON-line channel now supports handshake and view switching with a per-launch capability. It does not carry settings/jobs; do not treat it as feature-complete IPC. Do not ship or publish this binary as a functional sidecar.

Run locally: `cargo run` (Windows with WebView2). Check: `cargo fmt --check; cargo test --offline; cargo clippy --offline --all-targets -- -D warnings`.

The native window icon is a locally generated gold stacked-line mark (`assets/windows/sidecar.ico`, `sidecar-window.rgba`), not the Tempo icon. Earth, Three.js, vendor marks, bundled fonts, launcher, timers and Tempo state were omitted because the sidecar does not use them. The former bundled Earth CC-BY model must not accidentally return without its author credit. License and provenance review of Tempo-derived window scaffolding remains a release gate.

The parent owns authoritative settings, jobs and execution; keep arbitrary browsing isolated from this trusted offline shell. The visual approval does not authorize shipping incomplete implementation.
