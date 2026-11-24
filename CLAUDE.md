# Zaphire, notas para agentes

- Rust workspace in `crates/`, React UI in `ui/`. Read `docs/design.md` first.
- Build the app: `export CEF_PATH=$HOME/.local/share/cef` (fill it once with
  `export-cef-dir --force $CEF_PATH`), then `cargo build -p zaphire-app`.
  Run on Linux with `LD_LIBRARY_PATH=target/debug target/debug/zaphire`.
- Core crates (`zaphire-core`, `-perf`, `-ipc`, `-adblock`) build without CEF:
  `cargo test -p zaphire-core -p zaphire-perf -p zaphire-ipc -p zaphire-adblock`.
- UI: `cd ui && npm test && npm run build`. `npm run dev` runs against a mock core
  (`ui/src/lib/mock.ts`).
- `crates/zaphire-ipc/src/lib.rs` and `ui/src/lib/types.ts` must stay in sync.
- CEF gotchas found so far:
  - Only ONE Chrome-style BrowserView per window: main window views are Alloy.
  - `wrap_*_delegate!` unit structs (`struct X;`) drop the ViewDelegate impl:
    write `struct X {}`.
  - `PostData::elements` uses the Vec length as the count: pre-size it.
  - BoxLayout needs `cross_axis_alignment: STRETCH` or children get 0 height.
- UI libraries: see `.claude/skills/*` (Aceternity, Kobra, EasyUI, Pin UI, RareUI).
  No always-on animation in the shell; heavy effects only on the new-tab page.
- EGM artifact directory: docs/egm
