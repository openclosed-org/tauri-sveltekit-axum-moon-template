# Bootstrap Checklist

Use this list after creating a repository from the template.

## 1) Rebrand the App

- [ ] Update `apps/desktop-ui/src-tauri/tauri.conf.json`
  - [ ] `productName`
  - [ ] `identifier`
  - [ ] `app.windows[0].title`
- [ ] Update HTML title in `apps/desktop-ui/src/app.html`.
- [ ] Replace placeholder UI in `apps/desktop-ui/src/routes/+page.svelte`.

## 2) Prepare App Assets

- [ ] Add icon set under `apps/desktop-ui/src-tauri/icons/`.
- [ ] Verify bundle targets and metadata in `apps/desktop-ui/src-tauri/tauri.conf.json`.

## 3) Verify Toolchains

- [ ] Rust stable installed (`rustup show`).
- [ ] Node.js `24.0.0` available.
- [ ] Bun `1.3.11` available.

## 4) Install and Validate

- [ ] `bun install --cwd apps/desktop-ui`
- [ ] `cargo check --workspace --exclude desktop-ui-tauri`
- [ ] `cargo test --workspace --exclude desktop-ui-tauri`
- [ ] `cargo clippy --workspace --exclude desktop-ui-tauri -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `bun run --cwd apps/desktop-ui check`
- [ ] `bun run --cwd apps/desktop-ui lint`
- [ ] `bun run --cwd apps/desktop-ui build`

## 5) Security Baseline Before Release

- [ ] Replace `"csp": null` in `apps/desktop-ui/src-tauri/tauri.conf.json` with a strict CSP.
- [ ] Confirm no secrets are committed (`.env`, tokens, keys).
- [ ] Review `SECURITY.md` contact/reporting guidance.

## 6) Repository Metadata

- [ ] Update repository description and topics (see `ABOUT.md`).
- [ ] Confirm license choice in `LICENSE`.
- [ ] Keep `CHANGELOG.md` updated from day one.

## 7) Optional Next Steps

- [ ] Add a screenshot or demo GIF to `README.md`.
- [ ] Add issue labels/milestones for roadmap visibility.
- [ ] Add release automation if you plan tagged template releases.
