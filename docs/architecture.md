# Ramoso architecture

Nov 18, 2026 · @Larisa · updated Nov 19, 2026 after the spike (§11)

This document describes how Ramoso is built. What it does, screen by screen, is in [interface-spec.md](interface-spec.md). Section numbers like “spec §4” point there.

## 1. Goals and constraints

**Why Ramoso exists.** A Git client for daily work that makes every action visible, previewable and undoable (spec §1). It is open source.

**Goals**

- Every feature in the interface spec, built step by step in one app.
- Linux and Windows fully supported and tested; macOS built and covered by automated tests only.
- One UI on every OS: the custom frame from the design, with OS-specific shortcut labels and names.
- Git hooks run exactly as they do in the user’s terminal.
- Credentials are typed once and stored safely; restarts never ask again.

**Non-goals**

- AI-suggested conflict resolutions.
- Mobile builds.
- Using the system Git.

**Anti-goals** (failures even if everything else works)

- A Git command runs without appearing in the Console.
- A destructive action runs without a snapshot or a preview.
- A secret is written to a file, log or Git config in plain text.
- Local work stops because of a network or sign-in problem.

## 2. Stack

| Layer | Choice |
| --- | --- |
| Shell | Tauri 2 (Rust) |
| UI | React, TypeScript, Vite |
| Styling and components | Tailwind CSS, shadcn/ui, Geist and Geist Mono |
| Data from Rust | TanStack Query |
| UI state | Zustand |
| Forms | React Hook Form and Zod, only where a screen has a real form (Clone, Create, Settings, Create tag, …) |
| Tests | Vitest, Playwright, Rust `cargo test`; WebdriverIO smoke tests later |
| Git | Always bundled: our own static build on Linux (§9); dugite-native builds on Windows and macOS |
| Credentials | OS password store via `keyring-core` and the platform store crates; Ramoso is Git's credential helper |
| Linux AppImage | quick-sharun, built in an Arch container (bundles glibc) |
| Windows installer | NSIS, cross-compiled on Linux in a Debian container (`cargo-xwin`) |

Not used: Next.js (Tauri serves a static bundle; nothing uses server rendering) and TanStack Router (no URLs or back/forward in the spec; see §9).

## 3. Approaches considered

| Decision | Chosen | Rejected, and why |
| --- | --- | --- |
| How Ramoso talks to Git | Run the bundled `git` program for every read and write | A Rust Git library (gitoxide) for fast reads: those reads would be invisible in the Console (spec goal 3), and it means two Git implementations to trust and parse |
| Which Git | Always the bundled one | System Git when new enough: version drift, untested output formats |
| Git on Linux | Static Git built from source (musl, our own static curl) | dugite's Linux build: needs glibc 2.34, so it fails on older distros such as openSUSE Leap 15.4 (glibc 2.31) |
| Word diff and moved lines | Get the plain patch from Git, compute word diff and moves in Rust | Parse `--color-moved` output: means reading colour codes back out of terminal output |
| Screen navigation | A screen value in Zustand | TanStack Router: typed parameters, but the spec has no URLs or history |
| Data from Rust | TanStack Query, invalidated by file-watch events | Zustand only: every screen would hand-write loading, caching and refresh |
| App data | JSON files plus a per-repo append-only Activity log | SQLite: nothing needs real queries yet |
| Sign-in | Ramoso as Git's credential helper, tokens in the OS store via `keyring-core`; pasted tokens first, browser sign-in later (roadmap milestone 14) | Git Credential Manager: 79 MB of .NET on Linux, and a second sign-in system next to the one the provider layer needs anyway |
| Linux packaging | AppImage via quick-sharun (bundles every library, glibc included) | Tauri's own AppImage step (linuxdeploy): misses WebKit's helper programs on non-Debian layouts, so the app crashes on start |
| CI | Postponed; builds run locally in Docker | GitHub Actions now: not needed yet; Windows is tested by hand |
| Tests | Vitest + Playwright (mocked Rust) + Rust tests on real repos | Playwright alone: never touches Rust or real Git |
| License | MIT OR Apache-2.0 | GPL-3.0 |

## 4. Overview

```
┌──────────────────────────── React (webview) ────────────────────────────┐
│ Screens (spec §2–§11)   shadcn/ui components   Zustand (UI state)       │
│        │                                                                │
│ TanStack Query  ◄── invalidate on "repo-changed" events                 │
│        │                                                                │
│ ipc.ts: the only module that calls invoke() / listens to events         │
└────────┼────────────────────────────────────────────────────────────────┘
         │ Tauri commands, events, channels (JSON)
┌────────┼──────────────────────────── Rust ──────────────────────────────┐
│ commands.rs  thin layer: argument checks, call a module, map errors     │
│                                                                         │
│ repo     reads: status, log + graph, diff, blame, branches, stashes …   │
│ ops      writes: preview → snapshot → run → record in Activity          │
│ git      runner: bundled git, environment, queue, Console log, masking  │
│ watch    folder watcher → "repo-changed" events                         │
│ creds    OS password store (keyring) + Git credential helper            │
│ hosts    provider seam: GitHub, GitLab adapters                         │
│ store    settings, repo list, push queue (JSON)                         │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │ child processes
                          bundled git (+ LFS; sh on Windows)
```

The rule: React never knows how Git works. It asks Rust for typed data (“the status of this repo”, “page 3 of the log”) and for actions (“preview a reset of main to 98ad0c3”, “run it”). Rust never renders anything.

## 5. Rust modules

Each module is meant to be deep: a small interface, with most of the work hidden behind it.

### 5.1 `git`: the runner

Everything that executes Git goes through one function:

```rust
git::run(repo: &RepoPath, args: &[&str], opts: RunOpts) -> Result<Output, GitError>
// RunOpts: source (button, palette, background…), stdin, stream: Option<Channel>, cancel token, write: bool
```

Behind it:

- **Where the bundled Git is.** Inside an AppImage (`APPIMAGE` and `APPDIR` set): `$APPDIR/git`. Everywhere else: Tauri’s `BaseDirectory::Resource`. Tauri’s own AppImage lookup can’t be used: quick-sharun rewrites the `/usr/lib` text inside it (see “No hardcoded system paths” below).
- **Binary and environment, Linux.** Our static Git is built with `RUNTIME_PREFIX`, so it finds its helpers next to itself; `GIT_EXEC_PATH`, `PREFIX` and `GIT_TEMPLATE_DIR` are not needed. Our curl has no built-in certificate path, so Ramoso always sets `GIT_SSL_CAINFO` to the first system bundle found (Debian/Ubuntu/Arch, Fedora/RHEL and openSUSE locations), unless the user set one. Using the system bundle keeps company CAs working.
- **Binary and environment, Windows.** Run `cmd\git.exe`, Git for Windows’ launcher, which sets up its own paths (including `usr\bin\sh.exe` for hooks). TLS uses Windows’ certificate store (`sslBackend = schannel` in the bundle’s config). Every Git process is started with `CREATE_NO_WINDOW` (0x08000000), or each call flashes a console window; the hooks Git starts stay hidden too (tested). Environment variable names are compared without case (Windows spells it `Path`). Paths from Tauri are turned from `\\?\E:\…` back into `E:\…`, because some tools can’t read the long form.
- **Binary and environment, macOS.** dugite-native’s variables (`GIT_EXEC_PATH`, `GIT_CONFIG_SYSTEM`, `GIT_TEMPLATE_DIR`); not yet tested.
- **User environment for hooks.** On Linux and macOS the app may start from a desktop menu with a bare `PATH`. At startup Ramoso runs the login shell once (`$SHELL -l -i -c 'env -0'`, with a timeout) and uses that environment as the base for every Git process, so hooks find node, asdf and nvm shims. If it fails or times out, the app’s own environment is used and the Console says so. Windows apps already get the user’s full environment.
- **Working folder.** Always the repository (or worktree) root. Hooks run there, as in the terminal.
- **Fixed flags.** `--no-pager`, `-c core.quotepath=off`, `-c color.ui=false`; `GIT_OPTIONAL_LOCKS=0` for background reads so a status refresh never takes `index.lock`.
- **Prompts.** `GIT_TERMINAL_PROMPT=0`. Credential and SSH passphrase prompts go through Ramoso’s own ask-pass helper (§5.6) so they appear as dialogs, never hang.
- **Nothing outside Ramoso answers prompts.** `GIT_ASKPASS` and `SSH_ASKPASS` are never inherited; Ramoso always sets its own. Every call starts with `-c credential.helper=` (clears helpers from the user’s config and the bundle’s system config, which sets `manager` on Windows) before adding Ramoso’s helper. During the spike an inherited `SSH_ASKPASS` (KDE’s `ksshaskpass`) answered silently with a real token from KWallet.
- **No secrets in arguments.** Tokens and passwords never appear on a command line (they would show in `ps` and shell history); they go through stdin or the credential protocol.
- **Output.** stdout and stderr are kept separate (the Console shows both; the spike glued them together and mixed Git’s output with the hook’s).
- **No hardcoded system paths.** quick-sharun rewrites every `/usr/lib/…` and `/usr/share/…` string in the AppImage’s program to a random `/tmp/…` path. Ramoso’s code never contains such strings.
- **Queue.** Per repository, writes run one at a time in order; reads run in parallel. Background fetch counts as a write.
- **Console log.** Every call becomes a Console entry: time, command, source, duration, exit code, stdout and stderr, background flag. Entries stream to the UI over an event; the last 1,000 per session are kept in memory.
- **Secret masking.** Tokens in URLs (`https://user:token@…`) and known token formats are replaced with `••••` before anything reaches the log or the UI.
- **Known errors.** stderr is matched against a table of known failures (rejected push, auth failure, `index.lock`, diverged branches, …) and returned as a typed `GitError` variant that the UI maps to the dialogs in spec §7 and §11. Unknown errors keep the raw message.
- **Cancel.** Kills the child process (Cancel clone, Stop bisect).

Hooks need no special code: Git runs them. “Commit without hooks” adds `--no-verify`. A failed hook is a failed commit whose output is already in the Console entry; the composer shows it inline.

### 5.2 `repo`: reads

Typed read functions, each one or a few Git calls plus a parser:

| Data | Git command (machine-readable form) |
| --- | --- |
| Status | `status --porcelain=v2 -z --branch` |
| Log page (200) | `log -z --format=<NUL-separated fields> --parents --skip/--max-count` |
| Graph lanes | Computed in Rust from parent lists, per page, stable lane colour per branch |
| Diff | `diff --no-color --no-ext-diff -M` (patch text), then §5.3 |
| Blame | `blame --porcelain -M -C --ignore-revs-file .git-blame-ignore-revs` |
| File history | `log --follow --name-status -z` |
| Branches / tags / remotes | `for-each-ref --format=…`, `remote -v` |
| Ahead / behind | `rev-list --left-right --count` |
| Conflict prediction, dry runs | `merge-tree --write-tree` (needs no working-tree changes) |

After the first full read of a large repo: `commit-graph write --reachable` (spec §5, Loading a large repository).

### 5.3 Diff engine

Input: Git’s patch text. Output: a typed model the diff viewer renders.

- Parses files, hunks and lines; detects binary, image, renamed, generated (`linguist-generated`, lock files) and large files (thresholds from Settings).
- **Word diff:** pairs removed and added lines inside a hunk and diffs them by word.
- **Moved lines:** matches blocks of removed lines with blocks of added lines across all files in the diff; matched blocks become “moved to N” / “moved from N”.
- **Line staging:** builds a patch from the selected lines (a partial pair becomes a deletion plus an addition, as with `git add -p`) and applies it with `git apply --cached` (or `--reverse` to unstage or discard).

The diff library is `similar` 3.x. In the spike, a 20,000-line file (1.6 MB) took under 10 ms for the line diff plus word highlights, and 3.3 ms for the worst case (two unrelated files). `similar` has no moved-line detection; that is Ramoso’s own block matching on top, still untested for speed. Limits (roadmap milestone 4): ≤ 50 ms for line diff, word highlights and moved-line matching on a 20,000-line file; ≤ 100 ms from click to the diff on screen. Diffs run with a timeout (2 s), after which `similar` returns an approximate result. This is the most algorithm-heavy part of Ramoso and gets the most unit tests.

### 5.4 `ops`: every action that changes the repository

One interface for all of them:

```rust
ops::preview(repo, action: Action) -> Result<Preview, OpError>   // what will happen
ops::run(repo, action: Action)     -> Result<Outcome, OpError>   // do it
ops::undo(repo, activity_id)       -> Result<Outcome, OpError>
```

`Action` is one enum covering commit, discard, reset, checkout, branch create/delete/rename, merge, rebase (plan), cherry-pick, revert, stash apply/pop/drop, pull, push, force push, tag, bisect steps, worktree and submodule changes.

Behind it:

- **Preview** returns what the spec’s dialogs show: commits that move, files affected, predicted conflicts (`merge-tree`), whether a force push is needed. The UI enables the button only after a preview (spec principle “Preview, then act”).
- **Snapshot** before anything that destroys uncommitted work (discard, hard reset, checkout with clashes, stash drop): the working tree and index, untracked files included, are written as a hidden commit using a temporary index (`GIT_INDEX_FILE`, `add -A`, `write-tree`, `commit-tree`) and stored at `refs/ramoso/snapshots/<timestamp>`.
- **Run** calls the Git commands through `git::run`.
- **Record** appends an Activity entry (§5.8) with the before/after refs and the snapshot id.
- **Undo** reverses an entry: branch moves from the recorded old SHA (or reflog), lost work from its snapshot. Undoing an entry that later entries depend on undoes those too; `preview` of an undo lists them first.
- **Operations that pause** (merge, rebase, cherry-pick, revert, bisect) are detected from `.git` state files (`MERGE_HEAD`, `rebase-merge/`, `CHERRY_PICK_HEAD`, `BISECT_LOG`, …) and reported in status, which drives the sidebar card.
- **Pushed changes** are recorded as “Pushed · shared”; their undo is a revert, never a rewrite.

### 5.5 `watch`: refresh within 200 ms

- `notify` with its debouncer (~100 ms) watches the working tree recursively, plus `HEAD`, `index`, `refs/` and the operation state files inside `.git`; other `.git` noise is ignored.
- Each batch emits one `repo-changed { repo, scope }` event (`worktree`, `index`, `refs`, `operation`); the UI invalidates the matching queries.
- Linux limits inotify watches. If a watch fails for that reason, that repo falls back to polling `status` every 2 s and Settings shows why.

### 5.6 `creds`: credentials

Secrets live only in the OS password store. The OS encrypts them and unlocks them at login, so restarts don’t ask again. Only OS stores; no separate password managers (1Password, Bitwarden, `pass`).

How it works:

- **`keyring-core` with the platform store crates, tried in order:** Linux: Secret Service (`zbus-secret-service-keyring-store`), then the kernel keyring (`linux-keyutils-keyring-store`, kept until logout or reboot). Windows: Credential Manager. macOS: the login keychain (the “protected” store needs a signed app). A store that can’t start is left out and logged.
- **Saving reports whether the secret survives a restart.** With no working store the secret stays in memory only, and Ramoso tells the user they will have to sign in again after restarting. Plain-text files are never used.
- **Secrets are stored as bytes, not passwords:** Windows keeps passwords as UTF-16 capped at 2,560 bytes, which long tokens can exceed.
- **Store calls run off the async threads** (they block on D-Bus or OS APIs). The Linux store uses `async-io`, keeping its D-Bus calls independent of Tauri’s tokio runtime.
- **Secrets never reach the webview.** Rust keeps them; React sees only account names and whether sign-in is remembered.
- **Tests** use a mock store and an always-unavailable store, so the fallback and memory-only paths are covered.

| Secret | Stored by |
| --- | --- |
| Token for GitHub, GitLab or any host (HTTPS and API) | Pasted once in Accounts, saved by Ramoso in the OS store. Browser sign-in for GitHub and GitLab.com comes later (roadmap milestone 14) |
| Login and password for other hosts | Asked once in a Ramoso dialog with “Remember” ticked; Ramoso saves it itself |
| SSH key passphrase | ssh-agent if running; otherwise optionally remembered in the OS store and supplied through `SSH_ASKPASS` |
| GPG passphrase | gpg-agent, as in the terminal |

- **Ramoso is Git’s credential helper.** The Ramoso executable, started by Git with the credential protocol (`get`, `store`, `erase` on stdin). `get` answers from the OS store with the account’s real username and the token. With nothing stored it answers `quit=1`, so Git asks no other source. Git’s `store` and `erase` don’t save anything: Ramoso saves credentials itself, from the Accounts screen or the sign-in dialog.
- **Token refresh is done by the running app, one at a time.** The helper asks the app instead of refreshing itself, because some hosts replace both tokens on every refresh (GitLab), so two Git commands refreshing at once would lock each other out.
- **Ask-pass helper:** the same executable, started by Git through `GIT_ASKPASS` and `SSH_ASKPASS`. It forwards the prompt to the running app, which shows a dialog and returns the answer. Tested: HTTPS login prompts and SSH passphrase prompts reach it.
- **Which store answers on Linux depends on the desktop.** Secret Service is provided by whichever program owns `org.freedesktop.secrets` on the session bus: GNOME Keyring, KWallet, KeePassXC… On the test machine (KDE Plasma, KWallet 5.90) it was GNOME Keyring, so tokens did not appear in KWalletManager. Settings → Accounts shows where secrets are stored (“Stored in: GNOME Keyring”), from that bus owner.
- Sign out deletes the entry. An expired credential opens “Sign-in failed” (spec §7).

### 5.7 `hosts`: the provider seam

Plain Git works with any host. Host-specific features go through one trait:

```rust
trait Provider {
    fn account(&self) -> Result<Account>;
    fn repos(&self, owner: &str, query: &str) -> Result<Vec<RemoteRepo>>;
    fn create_repo(&self, spec: NewRepo) -> Result<RemoteRepo>;
    fn pull_requests(&self, repo: &RemoteId, filter: PrFilter) -> Result<Vec<PrSummary>>;
    fn pull_request(&self, repo: &RemoteId, number: u64) -> Result<PrDetail>; // files, threads, reviewers, checks
    fn submit_review(&self, repo: &RemoteId, number: u64, review: Review) -> Result<()>;
    fn delete_branch(&self, repo: &RemoteId, branch: &str) -> Result<()>;
    fn ssh_key_registered(&self, public_key: &str) -> Result<Option<bool>>;
}
```

- Adapters: `github` (REST/GraphQL) and `gitlab` (REST; merge requests, `!N` numbering, pipelines as checks). Two adapters from day one, so the seam is real. Gitea, Forgejo and Bitbucket are later adapters.
- The adapter is picked from the remote URL’s host; self-hosted GitLab instances are added in Accounts.
- HTTP via `reqwest`; token from `creds`.
- Network failures never block local work: they pause sync for that repo (spec principle “Never block local work”).

### 5.8 `store`: app data

| Data | Where |
| --- | --- |
| Settings, repo list (pinned, recent, order, colours), window state | JSON in the app data folder (Tauri store plugin) |
| Push queue (offline) | JSON in the app data folder |
| Activity log | `.git/ramoso/activity.jsonl` in each repository (in the common `.git` folder, so worktrees share it); append-only |
| Snapshots | Git refs: `refs/ramoso/snapshots/*` |
| Settings that are Git config (identity, pull strategy, signing) | Git config, global or repo, through `git config` |

Snapshots and Activity entries older than the retention period (default 30 days) are pruned when a repo opens.

### 5.9 Background work

- **Fetch:** per open repo, every 5 minutes (Settings), on window focus and on demand; `fetch --prune`. Marked as background in the Console.
- **Offline:** a failed fetch or push with a network error marks the app offline; retries back off from 30 s to 5 min; queued pushes run in order when the connection returns.
- **Windows:** each Tauri window has its own repo. “Open in new window” (worktrees) opens another window of the same app.

## 6. IPC between React and Rust

- **Commands** for reads and actions, one per `repo` function and one per `ops` entry point. Every command returns `Result<T, AppError>`; `AppError` is serialised as `{ kind, message, gitStderr? }` so the UI can pick the right dialog.
- **Channels** for streams tied to one call: clone progress, bisect “run automatically”, long hook output.
- **Events** for app-wide signals: `repo-changed`, `console-entry`, `sync-state`, `offline`.
- **`src/lib/ipc.ts`** is the only file that calls `invoke` or `listen`. Its types mirror the Rust types by hand. Generating them (for example with `tauri-specta`) is a later option if drift becomes a problem.
- **Permissions:** the capability file grants only the window controls, dialog, store and opener permissions the app uses.
- **Content Security Policy:** strict, never `null` (the Tauri template’s default). Only the app’s own scripts and styles; no remote code.

## 7. Frontend

### 7.1 Structure

```
src/
  app/            shell: title bar, rail, sidebar, console bar, theme, shortcuts
  screens/        one folder per spec area: changes, history, branches, sync, conflicts, rebase,
                  activity, palette, console, pull-requests, settings, start, setup, errors
  components/ui/  shadcn/ui components (generated, then owned)
  components/     shared Ramoso components: diff viewer, graph, file list, toast content
  lib/ipc.ts      typed Rust calls and events
  lib/queries.ts  TanStack Query keys and hooks
  lib/keys.ts     shortcut table
  stores/         Zustand: navigation, selection, dialogs, console panel, layout
```

### 7.2 State

- **TanStack Query** holds everything that comes from Rust, keyed by `[repoId, kind, …args]`. `repo-changed` events invalidate by scope; nothing polls.
- **Zustand** holds what only the UI knows: current repo and screen, selected commit/file/lines, open dialog, console panel size, rail expanded.
- **Navigation** is the `screen` value in Zustand. Sections (⌘1–⌘5), the operation card and the palette all set it. Dialogs are state, not screens.

### 7.3 Design system

- Colours from the design (zinc neutrals, semantic green/red/violet/blue/amber) as CSS variables in the shadcn theme, dark by default, light and “Match system” from the OS setting.
- Geist and Geist Mono bundled with the app (no network font loading).
- The 43 design screens are reference mockups with inline styles; they are rebuilt with Tailwind and shadcn, not copied.

### 7.4 Title bar and window

- `decorations: false` on every OS. The title bar calls `startDragging()` on mouse down and `toggleMaximize()` on double-click.
- Window buttons are drawn by Ramoso: right side on Windows and Linux, left on macOS.
- Capability permissions: `core:window:allow-close`, `allow-minimize`, `allow-toggle-maximize`, `allow-start-dragging`.

### 7.5 Shortcuts

One table in `lib/keys.ts`: action id, key, label. The command palette, menus and key handlers all read from it. Labels render as Ctrl on Windows and Linux and ⌘ on macOS.

### 7.6 Responsive layout

The three widths from spec §2 (≥ 1280, 1000–1279, < 1000) are Tailwind breakpoints on the app container.

## 8. Testing

| Layer | Tool | What it covers | Where it runs |
| --- | --- | --- | --- |
| Rust units | `cargo test` | Parsers, diff engine, graph lanes, masking, error matching | Linux, Windows, macOS |
| Rust integration | `cargo test` with the bundled Git | Real Git in temporary repos: every `ops` action, its undo, snapshots, hooks (a failing pre-commit hook), conflicts | Linux, Windows, macOS |
| Frontend units | Vitest | Stores, formatting (dates, SHAs, counts), shortcut labels | Linux |
| UI scenarios | Playwright against the Vite build, with Rust calls replaced by `mockIPC` | The 14 scenarios in spec §12, keyboard flows, both themes, accessibility checks | Linux |
| App smoke tests (later) | WebdriverIO + `@wdio/tauri-service` | The real app starts, opens a repo, commits | Linux, Windows |

Playwright can’t drive the Tauri window itself; that’s why the scenario tests run against the browser build with Rust mocked, and the Rust side is tested on real repos.

## 9. Build and releases

All builds run in Docker containers, as the current user, with caches in the project’s ignored `.cache/` folder. The setups were proven in the spike (branch `feat/platform-risks-spike`) and are copied from there in milestone 1. CI is postponed.

- **Repository layout:** `src/` (React), `src-tauri/` (Rust), `docs/`, `e2e/` (Playwright), `build/` (the Docker setups below).
- **Tool manifest.** One file lists every bundled tool with version, download URL and SHA-256: Git (source), curl (source), Git LFS, and the dugite-native Windows and macOS builds. Every build reads it; nothing is downloaded or copied by hand. Updating Git is one line.
- **Linux Git:** built from source in an Alpine container: static (musl), our own static curl (Alpine’s links libidn2, whose `error()` clashes with Git’s in a static link), `RUNTIME_PREFIX`, Rust part enabled. The build fails if anything links dynamically. About 37 MB after pruning to `git`, `git-remote-http`, `git-http-fetch`, Git LFS and the helper scripts. `strip` runs only on real binaries and its errors are not hidden.
- **Linux app: AppImage.** In an Arch container (packages from a dated archive snapshot): `tauri build --no-bundle`, then quick-sharun (pinned by commit and checksum) packs the app with every library, glibc included, so it runs on old and new distros. The build adds `WEBKIT_DISABLE_DMABUF_RENDERER=1` (WebKitGTK draws thick text with bundled graphics libraries on openSUSE Leap 15.4) and copies Git to `AppDir/git`. Size is about 165 MB; switching to quick-sharun’s debloated packages (no LLVM) should shrink it.
- **Windows app: NSIS installer.** In a Debian container: Rust with the `x86_64-pc-windows-msvc` target, `cargo-xwin` (downloads the Windows SDK), LLD, `clang-cl`, and NSIS 3.11 (`makensis` built from source into Tauri’s NSIS package; Debian ships 3.08). About 48 MB. Installs per user, no admin rights; unsigned, so SmartScreen asks once (More info → Run anyway).
- **macOS app:** built on a Mac (`tauri build --bundles dmg`).
- **Shared Docker run settings:** one script holds the common `docker run` flags (user, `HOME`, cache folders, mount); the Dockerfiles share the Rust and pnpm setup instead of repeating it.
- **Local pitfalls:** Homebrew on Linux puts its own `pkg-config` first in `PATH`, which can’t see system libraries, so host builds set `PKG_CONFIG=/usr/bin/pkg-config`. Docker without buildx needs `DOCKER_BUILDKIT=0` and `docker create` + `docker cp` to copy results out.
- **Releases:** Linux AppImage, Windows installer, macOS `.dmg` (unsigned until there’s a reason to sign). A `.deb`/`.rpm` would need its own pipeline (the AppImage path uses `--no-bundle`).

## 10. Licensing

- Ramoso: MIT OR Apache-2.0.
- Bundled Git: GPLv2. Ramoso runs it as a separate program, which doesn’t affect Ramoso’s license; releases include Git’s license text and a link to the exact source.
- Geist fonts: SIL Open Font License; license file shipped with the fonts.

## 11. Spike results (Nov 2026)

Tested on openSUSE Leap 15.4 with KDE Plasma (glibc 2.31) and on Windows (by hand). The spike code is on branch `feat/platform-risks-spike`.

| # | Question | Result | How it was tested |
| --- | --- | --- | --- |
| 1 | The bundled Git runs from the packaged app; executable bits survive; HTTPS works | **Yes**, Linux (AppImage) and Windows (installer) | A test app ran `git --version`, a commit and an HTTPS `ls-remote` from inside each package. Linux uses the system’s CA bundle instead of a bundled one (keeps company CAs working); Windows uses its own certificate store |
| 2 | Hooks run on Windows | **Yes**: `usr\bin\sh.exe` is in the bundle and ran a pre-commit hook calling `node` | Test app on Windows |
| 3 | Hooks see the user’s PATH when started from the desktop | **Yes** | On KDE, apps already receive the session’s PATH (asdf found); the login-shell capture also worked as a safety net. On Windows, `node` from nvm for Windows was found |
| 4 | Credentials from the OS store reach Git | **Yes**, with a changed design: Git Credential Manager dropped, Ramoso is the helper (§5.6) | A test helper saved a token through `keyring` and Git read it back with every other source switched off. The token was found over D-Bus in GNOME Keyring’s `login` collection, not in KWallet |
| 5 | Login and passphrase prompts reach Ramoso | **Partial:** Linux yes; Windows untested (SSH there goes through the bundled `usr\bin\ssh.exe`) | A stand-in ask-pass script received Git’s “Username/Password for https://github.com” prompts and an `ssh-add` passphrase prompt (`SSH_ASKPASS_REQUIRE=force`, OpenSSH 8.4) |
| 6 | The diff library is fast enough on 20,000 lines | **Yes**: under 10 ms (§5.3) | Benchmark with 1,000 edited lines and a moved 200-line block, Myers and Patience |
| 7 | A repo made by a newer system Git (reftable) works | **Yes** | Repo created by system Git 2.56 with `--ref-format=reftable`; the bundled Git ran status, log and branch in it |

**Changed by the spike:** Linux Git built from source (dugite’s needs glibc 2.34); Git Credential Manager dropped; token sign-in first, browser sign-in later; AppImages through quick-sharun, not Tauri’s linuxdeploy step; CI postponed; Windows-specific runner rules (§5.1).

**Known limits, for milestone 1 and later:**

- A real husky / lint-staged setup was not tested, only a plain `sh` hook calling `node`.
- The current Linux build makes only an AppImage; `.deb`/`.rpm` need their own pipeline.
- Ask-pass and credentials were tested on Linux only; macOS was not tested at all.
- The OS store was proven through GNOME Keyring only. Not tested: KDE without GNOME Keyring, where KWallet itself must provide Secret Service; milestone 8 tests on such a setup.
- The AppImage was tested on one machine only (openSUSE Leap 15.4, KDE). No other distros yet.
- Moved-line detection (Ramoso’s own block matching) has no speed test yet.
- The spike’s test helper returned a fixed username and ignored Git’s `store`; the real helper returns the account’s username, and Ramoso saves credentials itself (§5.6).

## 12. Still open

- Activity snapshot retention and disk budget (spec §13).
- Whether to generate TypeScript types from Rust (`tauri-specta`) instead of writing them by hand.
- Registering Ramoso’s OAuth apps on GitHub and GitLab.com, for browser sign-in (roadmap milestone 14).
- When to set up CI.
