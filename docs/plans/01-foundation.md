# Milestone 1: Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A buildable Ramoso app (AppImage and Windows installer, built in Docker) that shows the empty frame in both themes and runs `git --version` through the real runner into the Console.

**Architecture:** React in the webview talks to Rust only through `src/lib/ipc.ts`. Rust's `git` module is the only code that starts Git: bundled binary, controlled environment, per-repo write queue, masking, known errors, and a Console entry for every call. Builds reuse the spike's Docker setups (branch `feat/platform-risks-spike`), driven by one tool manifest.

**Tech Stack:** Tauri 2, Rust 1.99, React, TypeScript, Vite, Tailwind CSS 4, shadcn/ui, Zustand, TanStack Query, Vitest, Playwright, pnpm 9.1.0, Docker (classic builder).

**Spec:** [docs/architecture.md](../architecture.md) (§5.1, §6, §7, §9, §10, §11), [docs/interface-spec.md](../interface-spec.md) (§1 Visual language, §2, §10 Git console, §11 Theme and Light theme), [roadmap.md](roadmap.md) milestone 1.

## Decisions taken for this plan

- **Tool manifest:** `build/tools.txt`, one line per tool: `name version url sha256`. Architecture §9 says “Updating Git is one line”; `KEY=value` would need three lines per tool, JSON several. Read by `build/fetch-tools.sh` with `while read`.
- **Runner:** the full `RunOpts` from architecture §5.1 (source, stdin, stream channel, cancel token, write flag), including Cancel.
- **Theme UI:** a minimal Settings screen with only Appearance → Theme, saved with the Tauri store plugin.
- **Git flow:** GitHub Flow. One short-lived branch and one pull request per task, off `main`, named by the Conventional Branch spec (`feat/tool-manifest`, `feat/git-runner`, `fix/…`, `chore/readme` for docs-only work). Each pull request is merged before the next task starts, so none grows large, and every task leaves `main` building with all tests green. Tasks 0 and 1 share the first pull request (`feat/scaffold`). Commit messages: Conventional Commits with only the first letter capital, `Type(scope): description` (`Feat: scaffold Tauri, React and test setup`).
- **Runner state:** `git::run(repo, args, opts)` keeps the architecture’s signature. Its state (Git location, base environment, queues, Console log, entry callback) lives in one `Runner` set once at startup by `git::init`. Tests call `git::init` with a temp setup; no Tauri needed.

## Global Constraints

- Git runs only through `git::run` with the bundled Git. No other code starts a Git process.
- Every Git command appears in the Console, including background ones (shown dimmed).
- Secrets never reach a file, log, Console entry or Git config in plain text, never appear in command-line arguments, and never reach the webview.
- Errors are reported, never silently dropped; when something is deliberately ignored, a comment says why.
- No template leftovers or unused dependencies; the Content Security Policy is strict, never `null`.
- No `/usr/lib/…` or `/usr/share/…` strings in Ramoso’s code (quick-sharun rewrites them).
- `src/lib/ipc.ts` is the only file that calls `invoke`, `listen` or a plugin’s JS API.
- Shortcut labels: Ctrl on Windows and Linux, ⌘ on macOS. Every shortcut is registered in `src/lib/keys.ts`.
- Dark and light themes both work for every new screen; dark is the default.
- Colour is never the only signal; every control is a real button with a label; focus rings visible; contrast 4.5:1.
- Buttons at least 32 px tall (Comfortable density).
- Dates as “17 Nov, 14:02”.
- Sizes from spec §2: title bar 48 px, rail 52 px (34 px tiles), sidebar 232 px, Console bar 30 px, Console panel 300 px.
- Console keeps the last 1,000 entries per session.
- Login-shell capture: `$SHELL -l -i -c` with a 5 s timeout.
- Fixed Git flags: `--no-pager -c core.quotepath=off -c color.ui=false -c credential.helper=`; `GIT_TERMINAL_PROMPT=0`; `GIT_OPTIONAL_LOCKS=0` for reads; `GIT_ASKPASS`/`SSH_ASKPASS` never inherited; Windows `CREATE_NO_WINDOW` (0x08000000).
- Code comments: blank line between steps, one short comment per main block, no spec or item numbers in comments.

## Review Focus

1. **The login shell prints noise, hangs, or returns non-UTF-8 values.** Expected: the environment after the marker is used; on timeout or failure the app’s own environment is used and a Console note says so. Tested in Task 4.
2. **A repository path with spaces and non-ASCII letters** (`répo with space`). Expected: Git runs there and paths in output are not quoted. Tested in Task 4.
3. **A token inside a command argument** (`https://user:ghp_…@github.com/x.git` passed to `ls-remote`), not just in output. Expected: masked in the Console entry’s command text as well as stdout and stderr. Tested in Task 5.
4. **Large output on both pipes at once** (a 5 MB file through `git show`). Expected: no deadlock; stdout complete. Tested in Task 6.
5. **More than 1,000 calls in a session.** Expected: the oldest entries drop; the count in the Console bar keeps counting all calls. Tested in Task 6.

---

## File Structure

```
build/
  tools.txt                  tool manifest (name version url sha256)
  fetch-tools.sh             downloads + checks every manifest entry into .cache/tools/
  docker-run.sh              the shared `docker run` flags (user, HOME, caches, mount)
  git-linux/Dockerfile       static Git (from spike), pruned, strip errors visible
  app-linux/Dockerfile       Arch image (from spike), debloated quick-sharun packages
  app-linux/build-appimage.sh
  app-linux/ramoso.desktop
  app-windows/Dockerfile     Debian + cargo-xwin + NSIS 3.11 (from spike)
  build-linux.sh             git-linux image → .cache/git-linux → AppImage → dist-app/
  build-windows.sh           fetch dugite Windows → installer → dist-app/
src-tauri/
  src/lib.rs                 builder, plugins, git::init, commands registered
  src/commands.rs            thin commands: git_version, console_entries
  src/error.rs               AppError { kind, message, gitStderr? }
  src/git/mod.rs             run(), RunOpts, Output, Source, init()
  src/git/env.rs             bundled root, base environment, login shell, CA, Windows rules
  src/git/mask.rs            secret masking
  src/git/errors.rs          GitError + known-error table
  src/git/queue.rs           per-repo write lock
  src/git/console.rs         ConsoleEntry, ring of 1,000, callback
  tests/runner.rs            integration tests with the bundled Git
src/
  main.tsx, index.css        Tailwind + theme tokens + Geist @font-face
  lib/ipc.ts                 typed invoke/listen + store access
  lib/queries.ts             TanStack Query keys and hooks
  lib/keys.ts                shortcut table + label()
  lib/format.ts              formatTime()
  stores/ui.ts               Zustand: screen, consoleOpen, theme
  app/Frame.tsx, TitleBar.tsx, Rail.tsx, Sidebar.tsx, ConsoleBar.tsx, ConsolePanel.tsx, theme.ts
  screens/settings/Settings.tsx
  components/ui/             shadcn components actually used
  assets/fonts/              Geist, Geist Mono (woff2) + OFL.txt
  assets/brand/              logo-dark.svg, logo-light.svg
e2e/
  mock.ts                    mockIPC setup injected before the app loads
  frame.spec.ts
LICENSE-MIT, LICENSE-APACHE, README.md
```

---

### Task 0: Branch

- [ ] **Step 1:** Before every task, `git switch main && git pull`, then `git switch -c <type>/<topic>`. Expected: `git status` shows the new branch, clean. Each task ends by pushing its branch and opening a pull request into `main`.

### Task 1: Scaffold

**Files:** Create `package.json`, `vite.config.ts`, `tsconfig.json`, `tsconfig.app.json`, `tsconfig.node.json`, `index.html`, `src/main.tsx`, `src/index.css`, `src/App.tsx`, `components.json`, `vitest.config.ts`, `playwright.config.ts`, `e2e/mock.ts`, `e2e/frame.spec.ts`, `src-tauri/*` (from `pnpm create tauri-app`, React + TS), `LICENSE-MIT`, `LICENSE-APACHE`. Modify `.gitignore`.

**Interfaces:**
- Produces: `pnpm dev`, `pnpm build`, `pnpm test` (Vitest), `pnpm e2e` (Playwright against `vite preview`), `pnpm tauri dev`. Path alias `@/` → `src/`. Rust crate `ramoso`, lib `ramoso_lib`, identifier `io.ramoso.app`, window label `main`.

- [ ] **Step 1: Generate** with `pnpm create tauri-app` (React, TypeScript, pnpm), then add Tailwind 4 per shadcn’s Vite guide (`tailwindcss`, `@tailwindcss/vite`, `@` alias in `vite.config.ts` and both tsconfigs), `pnpm dlx shadcn@latest init`, `zustand`, `@tanstack/react-query`, `vitest`, `@playwright/test`.
- [ ] **Step 2: Remove template leftovers:** greet command, demo assets, `.vscode/`, template CSS, logos, `withGlobalTauri`. `deb` bundle target removed (AppImage goes through quick-sharun).
- [ ] **Step 3: Config:** `tauri.conf.json` window `main`, 1440×900, `decorations: false` (no minimum size: spec §2 has a layout below 1000 px); CSP `default-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost` (Tauri adds its script hashes and nonces at build time). Capability `default.json`: `core:event:default`, `core:window:default`, `core:window:allow-close`, `allow-minimize`, `allow-toggle-maximize`, `allow-start-dragging`, `store:default` (added in Task 8). Release profile from the spike’s `Cargo.toml`.
- [ ] **Step 4: Failing check:** `e2e/frame.spec.ts`:

```ts
test('app loads with mocked Rust', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('application', { name: 'Ramoso' })).toBeVisible();
});
```
`e2e/mock.ts` exports `installMocks(page, handlers)`: `page.addInitScript` that calls `mockIPC(handler, { shouldMockEvents: true })` and `mockWindows('main')` before the app script runs.

- [ ] **Step 5:** Run `pnpm e2e`. Expected: FAIL (no element with that role).
- [ ] **Step 6:** `App.tsx` renders `<div role="application" aria-label="Ramoso">`.
- [ ] **Step 7:** Run `pnpm build && pnpm test && pnpm e2e && (cd src-tauri && cargo test)`. Expected: all pass, zero warnings.
- [ ] **Step 8: Licenses:** `LICENSE-MIT`, `LICENSE-APACHE` (holder: the Ramoso authors), `"license": "MIT OR Apache-2.0"` in `package.json` and `Cargo.toml`.
- [ ] **Step 9: Commit** `chore: scaffold Tauri, React and test setup`.

### Task 2: Tool manifest and shared Docker script

**Files:** Create `build/tools.txt`, `build/fetch-tools.sh`, `build/docker-run.sh`. Modify `.gitignore` (`.cache/`, `dist-app/`).

**Interfaces:**
- Produces: `build/fetch-tools.sh <name>...` → each file at `.cache/tools/<name>/<file>`, checksum verified, exit non-zero with the name on mismatch. `build/docker-run.sh <image> <command...>` → runs as the host user with `HOME=/tmp`, `CARGO_HOME=/workspace/.cache/cargo`, repo mounted at `/workspace`, workdir `/workspace`, extra `-e` from the caller’s `DOCKER_ENV`.

- [ ] **Step 1: Manifest** entries copied from the spike’s Dockerfiles and README: `curl 8.23.0`, `git 2.56.1` (kernel.org `.tar.xz`), `git-lfs` (Linux amd64 release, version from the spike’s copied binary, `git-lfs --version`), `dugite-windows` and `dugite-macos` (dugite-native v2.56.1, checksums from the release’s `.sha256` files).
- [ ] **Step 2: Failing check:** run `build/fetch-tools.sh git` with its sha256 changed by one character in a temp copy of the manifest (`TOOLS=/tmp/... build/fetch-tools.sh git`). Expected before the script exists: command not found.
- [ ] **Step 3: Write** `fetch-tools.sh` (POSIX sh, `set -eu`, `sha256sum -c`, skip download when the cached file already matches) and `docker-run.sh`.
- [ ] **Step 4: Verify:** the bad-checksum run exits non-zero and names `git`; the real run fetches all five; a second run downloads nothing.
- [ ] **Step 5: Commit** `build: add tool manifest and shared docker run script`.

### Task 3: Static, pruned Linux Git

**Files:** Create `build/git-linux/Dockerfile`, `build/build-git-linux.sh`.

**Interfaces:**
- Consumes: `build/tools.txt` (versions and checksums passed as `--build-arg`, read by the script).
- Produces: `.cache/git-linux/` with `bin/git`, `libexec/git-core/{git-remote-http,git-remote-https,git-http-fetch,git-lfs}` and the helper scripts Git needs; about 37 MB.

- [ ] **Step 1:** Copy the spike Dockerfile. Changes: versions and checksums from build args; `strip` only on files `file` reports as ELF, without `2>/dev/null || true`; Git LFS copied in from the fetched tarball; prune `libexec/git-core` to the binaries above plus the shell helpers (`git-sh-setup`, `git-sh-i18n`, `git-mergetool--lib`, `mergetools/`), `share/git-core/templates` kept.
- [ ] **Step 2:** Keep the “fail if anything links dynamically” step and extend it to `git-lfs`.
- [ ] **Step 3: Verify:** `build/build-git-linux.sh`, then `du -sh .cache/git-linux` ≈ 37 MB, `.cache/git-linux/bin/git --version` → `git version 2.56.1`, `.cache/git-linux/bin/git lfs version` works, `git -C /tmp/x init && git -C /tmp/x commit --allow-empty -m t` with that binary succeeds.
- [ ] **Step 4: Commit** `build: static pruned Linux Git from the tool manifest`.

### Task 4: Runner environment

**Files:** Create `src-tauri/src/git/mod.rs`, `src-tauri/src/git/env.rs`, `src-tauri/tests/runner.rs`.

**Interfaces:**
- Produces:
  - `pub struct RepoPath(PathBuf)`; `RepoPath::new(path) -> RepoPath` applies `plain_path` (`\\?\E:\…` → `E:\…`, UNC left alone).
  - `pub struct GitSetup { pub root: PathBuf, pub env: Env, pub notes: Vec<String> }`; `type Env = BTreeMap<String, String>`.
  - `pub fn bundled_root(resource_dir: &Path) -> PathBuf` (`$APPDIR/git` when `APPIMAGE` and `APPDIR` are set, else `resource_dir/git`).
  - `pub fn base_env(app_env: Env, login: Option<Env>) -> (Env, Vec<String>)`: login env if present else app env plus a note; removes `GIT_ASKPASS`, `SSH_ASKPASS` (any case on Windows); sets `GIT_SSL_CAINFO` to the first existing system bundle (spike list) unless already set; `GIT_TERMINAL_PROMPT=0`.
  - `pub fn parse_login_env(bytes: &[u8]) -> Option<Env>` and `pub fn login_shell_env(timeout: Duration) -> Option<Env>` (unix; `None` on Windows).
  - `pub fn env_get<'a>(env: &'a Env, key: &str) -> Option<&'a str>`: case-insensitive on Windows.
  - Git program: `bin/git` (unix), `cmd/git.exe` (Windows).

- [ ] **Step 1: Failing unit tests** in `env.rs`:

```rust
#[test] fn login_env_skips_noise_before_marker() {
    let out = b"motd junk\n__RAMOSO_ENV_START__PATH=/a\0HOME=/h\0";
    let env = parse_login_env(out).unwrap();
    assert_eq!(env["PATH"], "/a");
    assert_eq!(env.len(), 2);
}
#[test] fn login_env_drops_non_utf8_entries() { /* b"..MARKER..A=1\0B=\xff\0" → only A */ }
#[test] fn base_env_never_inherits_askpass() {
    let app = env(&[("GIT_ASKPASS", "x"), ("SSH_ASKPASS", "ksshaskpass"), ("PATH", "/a")]);
    let (env, _) = base_env(app, None);
    assert!(!env.contains_key("GIT_ASKPASS") && !env.contains_key("SSH_ASKPASS"));
    assert_eq!(env["GIT_TERMINAL_PROMPT"], "0");
}
#[test] fn base_env_without_login_shell_adds_note() { /* notes non-empty */ }
#[test] fn base_env_keeps_user_ca_bundle() { /* GIT_SSL_CAINFO=/mine stays */ }
#[cfg(windows)] #[test] fn env_lookup_ignores_case() { /* "Path" found as "PATH" */ }
#[test] fn plain_path_strips_extended_prefix_only_for_drives() {
    assert_eq!(plain(r"\\?\E:\r"), PathBuf::from(r"E:\r"));
    assert_eq!(plain(r"\\?\UNC\s\x"), PathBuf::from(r"\\?\UNC\s\x"));
}
```

- [ ] **Step 2:** `cargo test git::env`. Expected: FAIL (functions missing).
- [ ] **Step 3:** Implement `env.rs` from the spike’s `login_shell_env`, `system_ca_bundle`, `plain_path`, with errors reported as notes instead of dropped.
- [ ] **Step 4:** `cargo test git::env`. Expected: PASS.
- [ ] **Step 5: Failing integration test** `tests/runner.rs` (bundled Git from `RAMOSO_GIT_ROOT`, default `../.cache/git-linux`; the test fails with that path in the message if missing):

```rust
#[test] fn runs_in_path_with_spaces_and_accents() {
    let repo = temp_repo("répo with space");          // git init through git::run
    std::fs::write(repo.join("é.txt"), "x").unwrap();
    let out = git::run(&RepoPath::new(&repo), &["status", "--porcelain"], opts_read()).unwrap();
    assert_eq!(out.stdout, "?? é.txt\n");            // quotepath off
}
```

- [ ] **Step 6:** Expected FAIL, then implement the minimal `git::run` in `mod.rs` (fixed flags, `env_clear` + base env, `current_dir`, `CREATE_NO_WINDOW` on Windows, stdout and stderr kept separate) and `git::init(setup, on_entry)`. Expected PASS.
- [ ] **Step 7: Commit** `feat(git): runner environment and bundled Git lookup`.

### Task 5: Secret masking and known errors

**Files:** Create `src-tauri/src/git/mask.rs`, `src-tauri/src/git/errors.rs`, `src-tauri/src/error.rs`.

**Interfaces:**
- Produces:
  - `pub fn mask(text: &str) -> Cow<str>`: replaces the userinfo password in `scheme://user:secret@` and the token formats `ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_`, `github_pat_`, `glpat-` with `••••`.
  - `pub enum GitError { Spawn(String), Cancelled, NotARepository, AuthFailed, IndexLocked { path: String }, PushRejected, Diverged, Unknown { code: Option<i32>, stderr: String } }`, `pub fn classify(code: Option<i32>, stderr: &str) -> GitError`.
  - `pub struct AppError { kind: String, message: String, git_stderr: Option<String> }`, serde camelCase, `From<GitError>`.

- [ ] **Step 1: Failing tests:**

```rust
#[test] fn masks_url_password() {
    assert_eq!(mask("https://bob:ghp_abc123@github.com/x.git"), "https://bob:••••@github.com/x.git");
}
#[test] fn masks_bare_tokens() {
    assert_eq!(mask("token glpat-AbC_12-x end"), "token •••• end");
    assert_eq!(mask("github_pat_11AB_cd"), "••••");
}
#[test] fn leaves_plain_urls_alone() { assert_eq!(mask("https://github.com/x.git"), "https://github.com/x.git"); }
#[test] fn classifies_known_errors() {
    assert!(matches!(classify(Some(128), "fatal: not a git repository (or any of the parent directories): .git"), GitError::NotARepository));
    assert!(matches!(classify(Some(128), "fatal: Authentication failed for 'https://…'"), GitError::AuthFailed));
    assert!(matches!(classify(Some(128), "fatal: Unable to create '/r/.git/index.lock': File exists."), GitError::IndexLocked { .. }));
    assert!(matches!(classify(Some(1), " ! [rejected]        main -> main (fetch first)"), GitError::PushRejected));
    assert!(matches!(classify(Some(128), "fatal: Need to specify how to reconcile divergent branches."), GitError::Diverged));
    assert!(matches!(classify(Some(2), "something new"), GitError::Unknown { .. }));
}
```

- [ ] **Step 2:** Expected FAIL. **Step 3:** Implement with plain string matching (no regex crate unless masking needs it). **Step 4:** Expected PASS.
- [ ] **Step 5: Commit** `feat(git): secret masking and known-error table`.

### Task 6: Queue, cancel, stream, Console log

**Files:** Create `src-tauri/src/git/queue.rs`, `src-tauri/src/git/console.rs`. Modify `src-tauri/src/git/mod.rs`, `src-tauri/tests/runner.rs`.

**Interfaces:**
- Consumes: Task 4 `run`, `init`; Task 5 `mask`, `classify`.
- Produces:
  - `pub enum Source { Button(String), Palette, Console, Background, App }` (serialised as `{ kind, label? }`).
  - `pub struct RunOpts { pub source: Source, pub stdin: Option<Vec<u8>>, pub stream: Option<tauri::ipc::Channel<String>>, pub cancel: Option<CancelToken>, pub write: bool }`; `CancelToken::new()`, `.cancel()`.
  - `pub struct Output { pub stdout: String, pub stderr: String, pub code: i32 }`.
  - `pub struct ConsoleEntry { id: u64, started_at_ms: i64, command: String, source: Source, background: bool, duration_ms: u64, exit_code: Option<i32>, stdout: String, stderr: String }` (camelCase); `command` is `git <args>` after masking.
  - `git::console_entries() -> Vec<ConsoleEntry>` (last 1,000), `git::console_count() -> u64` (all calls).
  - `init(setup, on_entry: Box<dyn Fn(&ConsoleEntry) + Send + Sync>)`.

- [ ] **Step 1: Failing integration tests:**

```rust
#[test] fn every_call_becomes_a_masked_entry() {
    let out = git::run(&repo, &["ls-remote", "https://u:ghp_x@127.0.0.1:1/x.git"], opts_read());
    assert!(out.is_err());
    let e = git::console_entries().last().unwrap().clone();
    assert!(e.command.contains("u:••••@") && !e.command.contains("ghp_x"));
    assert!(!e.stderr.contains("ghp_x"));
}
#[test] fn writes_to_one_repo_run_in_order() { /* 2 threads, write: true, each appends a commit; both succeed, no index.lock error */ }
#[test] fn large_output_on_both_pipes_does_not_deadlock() { /* commit a 5 MB file, `show HEAD:big`, stdout.len() == 5 MB */ }
#[test] fn cancel_kills_the_child() { /* stdin left open on `hash-object --stdin`… cancel after 100 ms → Err(Cancelled) within 1 s */ }
#[test] fn console_keeps_last_1000_but_counts_all() { /* 1,005 `--version` calls → entries().len()==1000, count()==1005 */ }
#[test] fn stream_receives_stderr_lines() { /* channel built with Channel::new(callback) */ }
#[test] fn stdin_reaches_git() { /* hash-object --stdin with b"x" → known SHA */ }
```

- [ ] **Step 2:** Expected FAIL. **Step 3:** Implement: write lock per canonical repo path (`Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>`), reads skip it and set `GIT_OPTIONAL_LOCKS=0`; stdout and stderr read on separate threads; `try_wait` loop checking the cancel token; stderr lines forwarded to `stream`; `VecDeque` capped at 1,000. **Step 4:** Expected PASS.
- [ ] **Step 5: Commit** `feat(git): write queue, cancel, streaming and Console log`.

### Task 7: Commands, ipc.ts and events

**Files:** Create `src-tauri/src/commands.rs`, `src/lib/ipc.ts`, `src/lib/queries.ts`, `src/lib/ipc.test.ts`. Modify `src-tauri/src/lib.rs`, `e2e/mock.ts`.

**Interfaces:**
- Consumes: Task 6.
- Produces:
  - Rust commands (each in `spawn_blocking`): `git_version() -> Result<String, AppError>` (runs `git --version` with `Source::App`, cwd = the app data folder), `console_entries() -> Vec<ConsoleEntry>`.
  - Events: `console-entry` (payload `ConsoleEntry`), `repo-changed` (payload `{ repo: string, scope: 'worktree'|'index'|'refs'|'operation' }`, type and listener only; emitted from milestone 3).
  - `lib.rs`: resolves `bundled_root`, captures the login shell in a background thread before `git::init`, emits each entry with `app.emit("console-entry", entry)`; startup notes become Console entries with `Source::App`.
  - `ipc.ts`: `gitVersion(): Promise<string>`, `consoleEntries(): Promise<ConsoleEntry[]>`, `onConsoleEntry(cb): Promise<UnlistenFn>`, `onRepoChanged(cb)`, `windowControls { minimize, toggleMaximize, close, startDragging }`, `AppError` type and `isAppError()`.
  - `queries.ts`: `useGitVersion()`, `useConsoleEntries()` (seeded from `consoleEntries`, appended by `onConsoleEntry`).
  - `e2e/mock.ts`: default handlers for `git_version` and `console_entries`; `emitConsoleEntry(page, entry)` helper using `emit` from `@tauri-apps/api/event` inside the page.

- [ ] **Step 1: Failing Vitest** `ipc.test.ts` with `mockIPC(..., { shouldMockEvents: true })`: `gitVersion()` resolves the mocked string; `onConsoleEntry` receives an emitted payload; a rejected invoke with `{ kind: 'IndexLocked', message: 'm' }` satisfies `isAppError`.
- [ ] **Step 2:** Expected FAIL. **Step 3:** Implement. **Step 4:** `pnpm test` and `cargo test`. Expected PASS.
- [ ] **Step 5: Commit** `feat: IPC layer with git_version and Console events`.

### Task 8: Theme, fonts, shortcuts, Settings → Theme

**Files:** Create `src/app/theme.ts`, `src/lib/keys.ts`, `src/lib/keys.test.ts`, `src/lib/format.ts`, `src/lib/format.test.ts`, `src/stores/ui.ts`, `src/screens/settings/Settings.tsx`, `src/assets/fonts/*`, `src/assets/brand/*`. Modify `src/index.css`, `src-tauri/Cargo.toml` (`tauri-plugin-store`), `src-tauri/src/lib.rs`, capability (`store:default`), `src/lib/ipc.ts` (`loadTheme`, `saveTheme`).

**Interfaces:**
- Produces:
  - `type ThemeChoice = 'dark' | 'light' | 'system'`, default `'dark'`. `applyTheme(choice)` sets `class="dark"` on `<html>`; `'system'` follows `matchMedia('(prefers-color-scheme: dark)')` live.
  - `ipc.loadTheme(): Promise<ThemeChoice>`, `ipc.saveTheme(c)`: store file `settings.json`, key `theme`.
  - `keys.ts`: `type Shortcut = { id: string; key: string; mod?: boolean; shift?: boolean; ctrl?: boolean }`, `SHORTCUTS` with `console.toggle` (⌃\`) and `settings.open` (⌘,), `label(s, os): string`, `useShortcuts(handlers)`.
  - `format.ts`: `formatTime(ms: number): string` → “17 Nov, 14:02” (local time).
  - `stores/ui.ts`: `screen: 'home' | 'settings'`, `consoleOpen`, `theme`, and setters.

- [ ] **Step 1: Failing Vitest:**

```ts
test('mod shortcut labels per OS', () => {
  expect(label(byId('settings.open'), 'macos')).toBe('⌘,');
  expect(label(byId('settings.open'), 'windows')).toBe('Ctrl+,');
  expect(label(byId('settings.open'), 'linux')).toBe('Ctrl+,');
});
test('console shortcut is Ctrl+` on every OS', () => {
  expect(label(byId('console.toggle'), 'macos')).toBe('⌃`');
  expect(label(byId('console.toggle'), 'linux')).toBe('Ctrl+`');
});
test('shortcut ids are unique', () => { /* new Set(ids).size === ids.length */ });
test('formats dates as "17 Nov, 14:02"', () => {
  expect(formatTime(new Date(2026, 10, 17, 14, 2).getTime())).toBe('17 Nov, 14:02');
});
```

- [ ] **Step 2:** Expected FAIL. **Step 3:** Implement.
- [ ] **Step 4: Tokens:** open `design/Git client concept.html` in a browser and read the zinc neutrals, semantic colours (green, red, violet, blue, amber), and the sprig logo (dark and light palettes). Write them as shadcn CSS variables in `index.css` (`:root` light, `.dark` dark). Light accents: added `#16a34a`, removed `#dc2626`. Check 4.5:1 contrast for text tokens in both themes. Export the logo as two SVGs and generate app icons with `pnpm tauri icon`.
- [ ] **Step 5: Fonts:** Geist and Geist Mono woff2 from the Vercel Geist release, `@font-face` in `index.css`, `OFL.txt` beside them. No network font loading.
- [ ] **Step 6: Settings screen:** Appearance → Theme as three preview cards (Light, Dark, Match system), a radio group with labels; change saves immediately.
- [ ] **Step 7: Playwright** in `e2e/frame.spec.ts`: choosing Light in Settings removes `dark` from `<html>`; reload with mocked store returning `'light'` starts light. Expected PASS.
- [ ] **Step 8: Commit** `feat: themes, fonts, shortcut table and Settings theme`.

### Task 9: App frame and Console

**Files:** Create `src/app/Frame.tsx`, `TitleBar.tsx`, `Rail.tsx`, `Sidebar.tsx`, `ConsoleBar.tsx`, `ConsolePanel.tsx`. Modify `src/App.tsx`, `e2e/frame.spec.ts`.

**Interfaces:**
- Consumes: Task 7 `windowControls`, `useGitVersion`, `useConsoleEntries`; Task 8 `useShortcuts`, `label`, `formatTime`, `stores/ui`.

- [ ] **Step 1: Failing Playwright tests:**

```ts
test('frame regions are present', async ({ page }) => {
  await expect(page.getByRole('banner')).toBeVisible();                       // title bar
  await expect(page.getByRole('button', { name: 'Minimize' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Maximize' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Close' })).toBeVisible();
  await expect(page.getByText('No repository open')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Changes' })).toBeDisabled();
});
test('console entry appears in bar and panel', async ({ page }) => {
  await emitConsoleEntry(page, entry({ command: 'git --version', exitCode: 0, stdout: 'git version 2.56.1' }));
  await expect(page.getByTestId('console-bar')).toContainText('git --version');
  await expect(page.getByTestId('console-bar')).toContainText('1');
  await page.keyboard.press('Control+`');
  await expect(page.getByRole('region', { name: 'Console' })).toContainText('git version 2.56.1');
});
test('failed entry shows ✕ and red stderr with a word, not only colour', async ({ page }) => { /* exitCode 128 → "✕" and "exit 128" */ });
test('background entry is dimmed', async ({ page }) => { /* source background → has data-background */ });
test('frame renders in both themes', async ({ page }) => { /* screenshot-free: html.dark toggles; no axe violations via @axe-core/playwright */ });
```

- [ ] **Step 2:** Expected FAIL.
- [ ] **Step 3: Implement** against the design (history and diff screen, frame only):
  - Title bar 48 px: logo + wordmark (drag area, `startDragging` on mousedown, `toggleMaximize` on double-click), “No repository open”, window buttons right on Windows/Linux and left on macOS. Fetch, Pull, Push and Search are not shown until their milestones.
  - Rail 52 px collapsed: no tiles yet; gear opens Settings, plus disabled with tooltip “Comes with Open and Clone”.
  - Sidebar 232 px: sections Changes, History, Branches, Pull requests, Activity, disabled, with the hint “Open a repository to see its changes”.
  - Console bar 30 px: last command, green ✓ / red ✕ dot with a word, count, “Console” button with its shortcut label.
  - Console panel 300 px, resizable (shadcn Resizable): entries newest last, time (`formatTime`), ✓/✕, `$ git …` in Geist Mono, source chip, duration and exit code, stdout and stderr (stderr red when exit ≠ 0), background entries dimmed. Toolbar: Close only (filters arrive in milestone 10).
  - On start the app calls `gitVersion()` once, so `git --version` is the first Console entry.
- [ ] **Step 4:** `pnpm e2e`. Expected PASS.
- [ ] **Step 5: Commit** `feat: app frame, Console bar and panel`.

### Task 10: AppImage and Windows installer

**Files:** Create `build/app-linux/Dockerfile`, `build/app-linux/build-appimage.sh`, `build/app-linux/ramoso.desktop`, `build/app-windows/Dockerfile`, `build/build-linux.sh`, `build/build-windows.sh`, `src-tauri/tauri.windows.conf.json`. Modify `src-tauri/tauri.conf.json` (resources `../.cache/git-linux/` → `git/`), `package.json` scripts (`build:linux`, `build:windows`).

**Interfaces:**
- Consumes: Task 2 `docker-run.sh`, `fetch-tools.sh`; Task 3 `.cache/git-linux`.
- Produces: `dist-app/Ramoso-x86_64.AppImage`, `dist-app/Ramoso-setup.exe`.

- [ ] **Step 1:** Copy the spike Dockerfiles; image names `ramoso-linux`, `ramoso-windows`. Shared Rust/pnpm setup kept identical in both files (one comment pointing at the other).
- [ ] **Step 2: Debloat:** switch the Arch image to quick-sharun’s debloated packages (no LLVM). Check the quick-sharun commit’s docs for the exact package names before editing.
- [ ] **Step 3:** `build-appimage.sh` from the spike: names `ramoso`, copies `.cache/git-linux` to `$APPDIR/git`, adds `WEBKIT_DISABLE_DMABUF_RENDERER=1`.
- [ ] **Step 4:** `build-windows.sh`: `fetch-tools.sh dugite-windows`, unpack to `.cache/git-windows/`, build with `cargo-xwin` and NSIS; `tauri.windows.conf.json` maps `../.cache/git-windows/` → `git/`.
- [ ] **Step 5: Verify:** `pnpm build:linux` and `pnpm build:windows` succeed from a clean `.cache/` (except caches). AppImage size noted (spike: 165 MB; expect lower). `rg '/usr/(lib|share)' src-tauri/src src` returns nothing.
- [ ] **Step 6: Commit** `build: AppImage and Windows installer in Docker`.

### Task 11: README and Done-when checks

**Files:** Create `README.md`. Modify `docs/plans/roadmap.md`.

- [ ] **Step 1: README:** what Ramoso is, requirements (Docker, pnpm, Rust), `pnpm tauri dev` (needs `build/build-git-linux.sh` once), tests, `pnpm build:linux`, `pnpm build:windows`, installing the AppImage and the unsigned installer (SmartScreen: More info → Run anyway), licenses (Ramoso MIT OR Apache-2.0; Git GPLv2 with source link; Geist OFL).
- [ ] **Step 2: Manual check, Linux (KDE):** AppImage starts; frame in dark and light; first Console entry is `git --version` → `git version 2.56.1`; window drag, double-click maximise, minimise, close work.
- [ ] **Step 3: Manual check, Windows:** same checks with the installer; no console window flashes.
- [ ] **Step 4:** Tick milestone 1 scope and “Done when” in `roadmap.md`, status ✅ with the date.
- [ ] **Step 5: Commit** `docs: README and milestone 1 done`; push the branch and open its pull request into `main`.
