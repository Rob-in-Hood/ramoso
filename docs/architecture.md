# Ramoso architecture

Nov 18, 2026 · @Larisa

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
| Git | Bundled portable Git (dugite-native builds), always |

Not used: Next.js (Tauri serves a static bundle; nothing uses server rendering) and TanStack Router (no URLs or back/forward in the spec; see §9).

## 3. Approaches considered

| Decision | Chosen | Rejected, and why |
| --- | --- | --- |
| How Ramoso talks to Git | Run the bundled `git` program for every read and write | A Rust Git library (gitoxide) for fast reads: those reads would be invisible in the Console (spec goal 3), and it means two Git implementations to trust and parse |
| Which Git | Always the bundled one | System Git when new enough: version drift, untested output formats |
| Word diff and moved lines | Get the plain patch from Git, compute word diff and moves in Rust | Parse `--color-moved` output: means reading colour codes back out of terminal output |
| Screen navigation | A screen value in Zustand | TanStack Router: typed parameters, but the spec has no URLs or history |
| Data from Rust | TanStack Query, invalidated by file-watch events | Zustand only: every screen would hand-write loading, caching and refresh |
| App data | JSON files plus a per-repo append-only Activity log | SQLite: nothing needs real queries yet |
| Sign-in | Git Credential Manager (bundled) for HTTPS, `keyring` for pasted tokens | A browser sign-in flow per host: much more code per host |
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
│ creds    password store (GCM + keyring)                                 │
│ hosts    provider seam: GitHub, GitLab adapters                         │
│ store    settings, repo list, push queue (JSON)                         │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │ child processes
                          bundled git (+ LFS, GCM, sh on Windows)
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

- **Binary and environment.** Resolves the bundled Git from `BaseDirectory::Resource` and sets the variables it needs (as GitHub Desktop does):
  - all OSes: `GIT_EXEC_PATH`
  - Linux, macOS: `GIT_CONFIG_SYSTEM`, `GIT_TEMPLATE_DIR`
  - Linux: `PREFIX`, `GIT_SSL_CAINFO` (bundled CA file, unless the user set one)
  - Windows: Git’s `bin` and `usr\bin` folders first in `PATH`
- **User environment for hooks.** On Linux and macOS the app may start from a desktop menu with a bare `PATH`. At startup Ramoso runs the login shell once (`$SHELL -l -i -c 'env -0'`, with a timeout) and uses that environment as the base for every Git process, so hooks find node, asdf and nvm shims. If it fails or times out, the app’s own environment is used and the Console says so. Windows apps already get the user’s full environment.
- **Working folder.** Always the repository (or worktree) root. Hooks run there, as in the terminal.
- **Fixed flags.** `--no-pager`, `-c core.quotepath=off`, `-c color.ui=false`; `GIT_OPTIONAL_LOCKS=0` for background reads so a status refresh never takes `index.lock`.
- **Prompts.** `GIT_TERMINAL_PROMPT=0`. Credential and SSH passphrase prompts go through Ramoso’s own ask-pass helper (§5.6) so they appear as dialogs, never hang.
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

The diff library is chosen in the spike (candidate: the `similar` crate). This is the most algorithm-heavy part of Ramoso and gets the most unit tests.

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

Secrets live only in the OS password store: Secret Service on Linux (KWallet, GNOME Keyring), Credential Manager on Windows, Keychain on macOS. The OS encrypts them and unlocks them at login, so restarts don’t ask again.

| Secret | Stored by |
| --- | --- |
| GitHub / GitLab sign-in (HTTPS) | Git Credential Manager (bundled), browser sign-in, saved to the password store |
| Token for API calls (PR review) | The same entry, read back with `git credential fill` |
| Login and password or token for other hosts | Asked once in a Ramoso dialog with “Remember” ticked; saved through Git’s credential helper |
| Pasted tokens (Accounts) | `keyring` crate |
| SSH key passphrase | ssh-agent if running; otherwise optionally remembered in the password store and supplied through `SSH_ASKPASS` |
| GPG passphrase | gpg-agent, as in the terminal |

- **Ask-pass helper:** the Ramoso executable itself, started by Git with a special argument (`GIT_ASKPASS`, `SSH_ASKPASS`). It forwards the prompt to the running app, which shows a dialog and returns the answer.
- **No password store on Linux:** `keyring` falls back to the kernel keyring (kept until logout or reboot), and Ramoso explains that installing KWallet or GNOME Keyring makes sign-in permanent. Plain-text files are never used.
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

## 9. Build, CI and releases

- **Repository layout:** `src/` (React), `src-tauri/` (Rust), `docs/`, `e2e/` (Playwright).
- **Bundled Git:** a script downloads the pinned dugite-native release for the target platform, checks its SHA-256, and unpacks it into `src-tauri/resources/git/`. It is listed in `bundle.resources` and never committed. The pinned version lives in one file; updating Git is a one-line change plus a CI run.
- **CI (GitHub Actions):** a matrix of `ubuntu-latest`, `windows-latest` and `macos-latest` runs lint, type checks and Rust tests; Vitest and Playwright run on Linux; every platform builds an installer.
- **Releases:** Linux AppImage and `.deb`, Windows installer, macOS `.dmg` (unsigned until there’s a reason to sign).

## 10. Licensing

- Ramoso: MIT OR Apache-2.0.
- Bundled Git: GPLv2. Ramoso runs it as a separate program, which doesn’t affect Ramoso’s license; releases include Git’s license text and a link to the exact source.
- Geist fonts: SIL Open Font License; license file shipped with the fonts.

## 11. To verify in a spike, before building on them

1. The bundled Git runs from the Tauri resource folder on Linux and Windows: executable bits survive bundling (AppImage, `.deb`), and HTTPS clone works with the bundled CA file.
2. The Windows build includes `sh`, so hooks run; a husky / lint-staged hook passes in a test repo.
3. The login-shell environment capture works on KDE with asdf: a hook that calls `node` succeeds when Ramoso starts from the app menu.
4. Git Credential Manager on Linux saves to KWallet through Secret Service (and what it needs configured), and `git credential fill` returns the token for API calls.
5. The ask-pass helper: Git and SSH prompts reach a Ramoso dialog on Linux and Windows.
6. A diff library for word diff and moved lines (`similar` or another) handles a 20,000-line file fast enough.
7. Bundled Git version versus the user’s system Git: a repo created by a newer system Git (for example with reftable) opens, or fails with a clear message.

## 12. Still open

- Activity snapshot retention and disk budget (spec §13).
- Whether to generate TypeScript types from Rust (`tauri-specta`) instead of writing them by hand.
