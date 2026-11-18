# Ramoso roadmap

Nov 18, 2026 · @Larisa

This roadmap orders the work that builds everything in [interface-spec.md](../interface-spec.md) on the design in [architecture.md](../architecture.md). Each milestone gets its own step-level plan (`docs/plans/NN-name.md`), written when the milestone starts, so it can use what earlier milestones and the spike taught.

## How milestones work

- **One plan per milestone.** Written and approved before any code for that milestone. Milestone 0 is a throwaway spike and has no plan; its results go into architecture.md §11.
- **Each milestone ships a working app.** Main is always buildable: the Linux AppImage and the Windows installer build locally in Docker, and all tests pass. CI is postponed (architecture §9).
- **Screens are built against the design.** The design screens named for each milestone are the visual reference (in the local `design/` folder, not versioned). They are rebuilt with Tailwind and shadcn/ui, not copied.
- **Data first, then screen.** Within a milestone, the Rust module and its tests land before the screen that uses it.
- **Settings grow with features.** Each milestone adds its own options to Settings; milestone 13 adds settings search and the remaining groups.

## Rules for every milestone

These apply to every task in every plan:

- Git runs only through `git::run` with the bundled Git. No other code starts a Git process.
- Every Git command appears in the Console, including background ones (shown dimmed).
- Destructive actions (discard, hard reset, checkout with clashes, stash drop, force push, branch delete) show a preview first and save a snapshot or backup before running.
- Secrets never reach a file, log, Console entry or Git config in plain text, never appear in command-line arguments, and never reach the webview.
- Errors are reported, never silently dropped; when something is deliberately ignored, a comment says why.
- No template leftovers or unused dependencies; the Content Security Policy is strict.
- Network and sign-in problems pause sync only; local work keeps working.
- The safe option is pre-selected and labelled; destructive options are red and never pre-selected.
- Plain names first, then the Git term in small monospace (“Keep the changes, unstaged · --mixed”).
- Buttons name the action (“Delete branch”), never “OK” or “Yes”. No exclamation marks.
- Dates as “17 Nov, 14:02”; relative times (“2 h”) in lists; full author names; SHAs shortened to 7 characters.
- Colour is never the only signal: every state also has a word, icon or +/− marker.
- Every control is a real button, link or input with a label; focus rings visible; text contrast 4.5:1 in both themes.
- Buttons at least 32 px tall in Comfortable density and 28 px in Compact.
- Shortcut labels: Ctrl on Windows and Linux, ⌘ on macOS. Every shortcut is registered in `src/lib/keys.ts`.
- Dark and light themes both work for every new screen.

## Milestones

| # | Milestone | Spec | Scenarios |
| --- | --- | --- | --- |
| 0 | Spike | architecture §11 | none |
| 1 | Foundation | §2, parts of §10 and §11 | none |
| 2 | Getting a repository | §3, parts of §11 | UC-14 |
| 3 | Changes and committing | §4 | UC-2 |
| 4 | History and the diff viewer | §5 | UC-3 |
| 5 | Activity and undo | §9 | UC-7 |
| 6 | Branches, tags and remotes | §6 (part) | none |
| 7 | Sync | §7 (part), §11 lock file | UC-4, UC-8 |
| 8 | Credentials and hosts | §3 accounts, §7 sign-in | UC-1, UC-9 |
| 9 | Conflicts and rewriting history | §8 | UC-5, UC-6, UC-10, UC-12 |
| 10 | Command palette and Console | §10 | none |
| 11 | Worktrees and submodules | §6 (part) | UC-11 |
| 12 | Pull requests | §10 | UC-13 |
| 13 | Finish | §2 sizes, §11, §13 | all 14, re-run |
| 14 | Browser sign-in | §3 accounts, §7 sign-in | UC-1 (browser part) |

### 0. Spike (throwaway)

Answers the seven questions in architecture §11 before anything is built on them. Code lives on the branch `feat/platform-risks-spike`, kept as a reference; milestone 1 copies its build setups.

- **Done when:** each of the seven items is marked confirmed or replaced with a working alternative in architecture.md §11, tested on Linux (KDE) and on Windows by hand.
- **Done (Nov 19, 2026).** Six confirmed; item 5 (ask-pass) is partial: Linux only, Windows untested. Results, design changes and known limits are in architecture §11. Windows was tested by hand instead of in CI.

### 1. Foundation

- **Scope:**
  - Project scaffold: Vite, React, TypeScript, Tailwind, shadcn/ui, Zustand, TanStack Query, Vitest, Playwright, Tauri 2.
  - Build setups from the spike (`build/`): static Linux Git, Linux AppImage (quick-sharun), Windows installer (cargo-xwin, NSIS); one shared `docker run` script; the tool manifest (Git, curl, Git LFS, dugite builds, with checksums).
  - Pruned Linux Git (about 37 MB) and the AppImage’s debloated packages.
  - `git::run` with the bundled environment, login-shell environment, write queue, secret masking, known-error table and Console entries, following the runner rules in architecture §5.1 (no inherited ask-pass, cleared credential helpers, Windows `CREATE_NO_WINDOW`, case-insensitive environment, plain paths, separate stdout/stderr, no hardcoded `/usr` paths).
  - `ipc.ts`, the `repo-changed` and `console-entry` events, and `mockIPC` setup for Playwright.
  - Theme tokens (dark, light, match system), Geist fonts bundled, the shortcut table.
  - App frame: custom title bar with window controls and dragging, collapsed repo rail, sidebar, Console bar, and a basic Console panel listing entries.
  - License files; README with build and install instructions.
- **Design screens:** brand mark (logo and app icon), history and diff (frame only), narrow window (frame only).
- **Done when:** the AppImage and the Windows installer build in Docker; the app starts on Linux and Windows, shows the frame in both themes, and running `git --version` through the runner shows up in the Console with the bundled version.

### 2. Getting a repository

- **Scope:**
  - Repo list store (pinned, recent, order, colours).
  - Rail: expanded state, tiles with state dots, drag to reorder, right-click menu.
  - Repository switcher (⌘⇧O).
  - Start screen (recent, filter, drop zone), Open a folder (including the scan for repositories two levels deep), Create (without publishing), Clone by URL (`ls-remote` preview, progress view, cancel, continue in background).
  - First-launch setup steps 1 (identity, `includeIf`) and 3 (appearance); step 2 (accounts) shows “Connect later” until milestone 8.
  - Repository moved or deleted.
- **Design screens:** start, first launch, clone a repository, open a folder, create a repository, repository not found.
- **Done when:** UC-14 passes; a public repo clones by URL with progress; a moved repo shows its empty state with Locate folder.

### 3. Changes and committing

- **Scope:**
  - Folder watcher with polling fallback.
  - Status, file list with Unstaged and Staged groups, the diff view for working-copy changes (unified only; the full viewer is milestone 4).
  - Staging by file, hunk and line; floating selection bar.
  - Commit composer: summary counter, description wrap, amend, co-author chips, ticket prefix, split button, hook output inline.
  - Snapshots (`refs/ramoso/snapshots/`) and the Activity log file, recording from this milestone on.
  - Discard with confirmation and undo toast.
  - Nothing to commit.
  - Stashes screen (apply, pop, drop, branch from stash); conflicting applies stop with a message until milestone 9.
- **Design screens:** working copy, nothing to commit, discard changes, stashes.
- **Done when:** UC-2 passes; a failing pre-commit hook keeps the message and shows its output; discarding a file and restoring its snapshot round-trips in a Rust integration test.

### 4. History and the diff viewer

- **Scope:**
  - Log pages of 200, graph lanes, ref badges, multi-select, right-click menu (actions that need later milestones are disabled with a tooltip).
  - Commit details.
  - Diff engine: word diff, moved lines, ignore whitespace, folded regions, J/K navigation, split view with synced scrolling.
  - Special files: image modes, binary, renamed, large, generated.
  - Full-width log with filters.
  - Blame and file history (follow moves, skip formatting commits).
  - Detached HEAD.
  - Loading a large repository (commit-graph write).
- **Design screens:** history and diff, split diff, image and special file diffs, full-width log, blame and file history, detached HEAD, loading a large repository.
- **Done when:** UC-3 passes; on the spike’s 20,000-line test file, line diff + word highlights + moved-line matching take ≤ 50 ms, and clicking the file to the diff on screen takes ≤ 100 ms; the graph matches `git log --graph` on a test repo with merges.

### 5. Activity and undo

- **Scope:**
  - `ops::undo` for every action recorded so far, including dependent steps.
  - Activity screen (filters, timeline, detail panel).
  - ⌘Z / ⌘⇧Z on any screen.
  - Undo toasts (8 s, hover pauses, stack of 3).
  - Retention pruning.
- **Design screens:** activity and undo, command palette and undo (toast parts).
- **Done when:** UC-7 passes, restoring both the commits and the uncommitted files; every action from milestones 3–4 has a tested undo.

### 6. Branches, tags and remotes

- **Scope:**
  - Branch switcher (⌘B, ⌘↵ creates a branch).
  - Branches screen (local, remote, tags tabs; cleanup callout; behind/ahead bars; row actions).
  - Checkout with uncommitted changes (carry or offer stash).
  - Delete a branch, Reset a branch (three modes).
  - Tags and remotes screen (add, edit with `ls-remote` test, prune, create and push tags).
- **Design screens:** branches, delete branch, reset branch, tags and remotes.
- **Done when:** deleting an unmerged branch and resetting with “Throw everything away” are both undoable from Activity.

### 7. Sync

- **Scope:**
  - Fetch button and background fetch (interval, focus, prune).
  - Pull with all five situations from spec §7, including the Diverged dialog with conflict prediction and remembered strategy.
  - Push (first push sets upstream), Push rejected, Force push with typed confirmation, lease and backup branch.
  - Offline banner, push queue with backoff.
  - Leftover lock file.
- **Design screens:** pull with diverged branches, push rejected, confirm force push, offline, lock file found.
- **Done when:** UC-4 and UC-8 pass against a local bare repository used as the remote; the push queue survives an app restart.

### 8. Credentials and hosts

- **Scope:**
  - `creds` as in architecture §5.6: `keyring-core` with the ordered platform stores, memory-only fallback that tells the user, and Ramoso as Git’s credential helper (`quit=1` when nothing is stored). Token sign-in only; browser sign-in is milestone 14.
  - Ask-pass dialogs for credentials and SSH passphrases.
  - Settings → Accounts shows where secrets are stored (for example “Stored in: GNOME Keyring”).
  - `Provider` trait with the GitHub and GitLab adapters (account, repos, create repo, delete branch, SSH key check).
  - Setup step 2 and Settings → Accounts.
  - Clone host tabs, Publish to a host on Create, “Also delete on \<remote\>”.
  - Sign-in failed.
- **Design screens:** sign-in needed, first launch (accounts), clone a repository (host tabs), create a repository (publish), settings (commits and signing: key picker).
- **Done when:** UC-1 (with a pasted token instead of the browser) and UC-9 pass (adapters tested against recorded API responses); on KDE, a restart doesn’t ask for credentials again.

### 9. Conflicts and rewriting history

- **Scope:**
  - Operation detection and the sidebar card.
  - Merge conflict resolver (three panes, result editor, decision bars, Combine, auto-resolved chunks, “why it conflicts”, shortcuts).
  - Interactive rebase plan with dry run.
  - Rebase paused.
  - Cherry-pick and revert, paused screen.
  - Bisect, including “let a command decide”.
  - Stash apply with conflicts now continues into the resolver.
- **Design screens:** resolve merge conflicts, interactive rebase, rebase paused on a conflict, cherry-pick paused, find a bad commit.
- **Done when:** UC-5, UC-6, UC-10 and UC-12 pass; aborting each operation restores the exact starting state in a Rust integration test.

### 10. Command palette and Console

- **Scope:**
  - Command palette (fuzzy search over actions, branches, commits, files, repos; prefixes; context chip; preview panel for risky actions; ⇥ target picking).
  - Full Console panel (filters, errors only, copy log, clear, known-error explanations, `$ git` command line with the same confirmations as buttons).
- **Design screens:** command palette and undo.
- **Done when:** every action from milestones 2–9 is reachable from the palette; a destructive command typed in the Console asks for confirmation.

### 11. Worktrees and submodules

- **Scope:** Worktrees screen (new, open in new window, remove with stash or discard); Submodules screen (statuses, update, update all, add); a second Tauri window per worktree.
- **Design screens:** worktrees, submodules.
- **Done when:** UC-11 passes; feature-branch work is untouched after the hotfix flow.

### 12. Pull requests

- **Scope:**
  - Pull requests in the sidebar (yours, review requested, all).
  - Review screen (conversation, commits, files with inline threads, pending review, approve or request changes, checks).
  - Check out locally.
  - Merge request wording and numbering on GitLab.
- **Design screens:** pull request review.
- **Done when:** UC-13 passes on GitHub and the same flow passes on GitLab (both against recorded API responses).

### 13. Finish

- **Scope:**
  - Settings search and remaining groups.
  - Narrow window layout (< 1000 px) and the 1000–1279 px fallbacks.
  - Light theme review of all screens.
  - Reduced motion.
  - Accessibility audit.
  - WebdriverIO smoke tests on Linux and Windows.
  - Release packaging (AppImage, Windows installer, `.dmg` built on a Mac) with Git’s license and source link.
- **Design screens:** settings (both), history and diff (light theme), narrow window.
- **Done when:** all 14 scenarios pass in one run; the AppImage and Windows installer build in Docker, and the `.dmg` builds on a Mac.

### 14. Browser sign-in

Starts once the core (milestones 1–13) is done.

- **Scope:**
  - Register Ramoso’s OAuth apps on GitHub and GitLab.com (public client IDs; no secrets in the app).
  - Device sign-in for GitHub and GitLab.com: show the code, open the browser, poll for the token.
  - Token refresh in the running app, one refresh at a time (GitLab replaces both tokens on every refresh).
  - “Sign in with \<host\>” in setup step 2, Accounts and Sign-in failed.
- **Design screens:** sign-in needed, first launch (accounts).
- **Done when:** UC-1 passes with browser sign-in on GitHub, and the same flow works on GitLab.com.

## Spec coverage

| Spec section | Milestones |
| --- | --- |
| §2 App shell | 1 (frame, Console bar), 2 (rail, repo switcher), 6 (branch switcher), 7 (Fetch, Pull, Push), 13 (window sizes) |
| §3 Getting a repository | 2; accounts parts in 8 (tokens) and 14 (browser sign-in) |
| §4 Changes and committing | 3; stash conflicts in 9 |
| §5 History and the diff viewer | 4 |
| §6 Branches, tags, remotes, worktrees, submodules | 6, 11 |
| §7 Sync | 7; sign-in failed in 8, browser fix in 14 |
| §8 Conflicts and rewriting history | 9 |
| §9 Activity and undo | 3 (recording), 5 |
| §10 Command palette, Console, pull requests | 1 (basic Console), 10, 12 |
| §11 Settings, themes, system errors | 1 (themes), 2 (repo moved), 7 (lock file), each milestone (its settings), 13 |
| §12 User scenarios | as listed per milestone; all re-run in 13 |
| §13 Shortcuts, accessibility, copy | rules above, every milestone; audit in 13 |

## Open

- Activity snapshot retention and disk budget (interface spec §13); needed by milestone 5.
- When to set up CI (postponed; builds and tests run locally until then).
