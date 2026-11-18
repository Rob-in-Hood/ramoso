# Ramoso interface specification

Nov 17, 2026 · @Larisa · updated Nov 18, 2026

## 1. Overview

Ramoso is an open-source desktop Git client that combines a precise diff viewer with a guided conflict resolver, and makes every Git action visible, previewable and undoable. This spec describes every screen and element of the design canvas, how each behaves, and the scenarios it must support. The technical design is in [architecture.md](architecture.md).

### Platforms and scope

- **Linux and Windows:** fully supported and tested.
- **macOS:** built and covered by automated tests only.
- **One UI on every OS:** the same custom frame and layout everywhere; only shortcut labels and OS names (file manager, password store) change.
- **Git:** Ramoso always uses its own bundled Git, a fixed version tested with the app and updated with each release. The system Git is never used.
- **Hosts:** plain Git (clone, fetch, pull, push over HTTPS or SSH) works with any host. Sign-in, repo browsing and pull request review go through a provider layer: GitHub and GitLab first, more hosts (Gitea, Forgejo, Bitbucket) later.

### Product goals

1. Reading changes is fast and exact: word-level diffs, moved-line detection, blame that follows moves and renames.
2. Risky actions are safe: each one shows its effect before it runs and can be undone from Activity for 30 days.
3. Git is never hidden: every command the app runs is listed in the Console with its output.
4. One frame everywhere: any screen can be reached from any state in one or two clicks.

### Design principles

- **Preview, then act.** Rebase, pull, reset, force push and discard show what will happen (commits, files, conflicts) before the button is enabled.
- **Plain names over Git jargon.** “Upstream” and “Your commit” instead of ours/theirs; “Throw everything away” instead of `--hard`. The Git term stays visible in small monospace text for experts.
- **The safe option is the default.** Recommended choices are pre-selected and labelled; destructive ones are red and never pre-selected.
- **Never block local work.** Network or sign-in problems pause sync only; commits, branches and history keep working.
- **Keyboard first, mouse friendly.** Every action is in the command palette (⌘K) and most have a direct shortcut.

### Terminology

| Term | Meaning in Ramoso |
| --- | --- |
| Repository (repo) | A Git project folder known to Ramoso, shown as a tile in the rail |
| Working copy | Files on disk with uncommitted changes, shown in Changes |
| Upstream | The branch you are rebasing or merging onto |
| Operation | A multi-step Git process that can pause: merge, rebase, cherry-pick, revert, bisect |
| Activity | Ramoso’s readable log of everything you did, built on the reflog plus its own snapshots |
| Snapshot | A hidden commit Ramoso makes before discard, hard reset or force push, so they can be undone |

### Visual language

- Components: shadcn/ui (Button, Badge, Tabs, Command, Dialog, Sonner toast, Switch, Checkbox, Table, Tooltip, Resizable).
- Type: Geist for UI, Geist Mono for code, hashes, paths and commands.
- Colour: zinc neutrals; dark theme by default, light theme and “Match system” available.
- Semantic colours: green = added, red = removed or destructive, violet = moved lines and feature-branch lanes, blue = the main branch and “upstream”, amber = in progress, attention or “your commit”. Colour is never the only signal: every state also has a word, icon or +/− marker.
- Brand: the ramose sprig mark (four alternating leaves on a tapering stem, lime bud) beside the “Ramoso” wordmark.
- Shortcut labels follow the OS: Ctrl on Windows and Linux, ⌘ on macOS. This spec writes the macOS form (⌘K); read it as Ctrl+K elsewhere.

## 2. App shell

Every screen sits inside the same frame: title bar on top, repository rail and sidebar on the left, Console bar at the bottom. Only the content area changes; dialogs open over the full frame.

### Title bar (48 px)

Ramoso draws its own title bar on every OS. Empty areas of the bar drag the window; double-clicking them maximises or restores it.

| Element | Behaviour |
| --- | --- |
| Logo + wordmark | Static. Part of the drag area. |
| Window controls | Minimise, maximise/restore, close. Right end of the bar on Windows and Linux; left end (traffic lights) on macOS. |
| Repository switcher (`acme/gateway ▾`) | Opens a searchable list of known repos, pinned first. Selecting one switches the whole window. Shortcut ⌘⇧O. Shows “No repository open” when none is. |
| Branch switcher (`main ↑ 0 ↓ 2 ▾`) | Opens a branch picker: type to filter, ↵ switches, ⌘↵ creates a branch with the typed name. Ahead/behind counts are versus the tracked remote branch. During an operation it shows the state instead (merging, rebasing, cherry-picking, bisecting, detached). |
| Search (⌘K) | Opens the command palette (section 10). |
| Fetch | Runs `git fetch --prune` for all remotes. Spins while running; result appears in the Console bar. |
| Pull (badge = commits behind) | Pulls with the repo’s remembered strategy. If branches have diverged and no strategy is remembered, opens the Diverged dialog. |
| Push (primary; small count = commits ahead) | Pushes the current branch; sets upstream on first push. Rejection opens the Push rejected dialog. Offline: queues the push and shows “1 queued”. |

### Repository rail (52 px, collapsible)

- Collapsed (default): one 34 px tile per repo with a two-letter monogram on a per-repo colour.
  - The current repo has a white ring.
  - An amber dot marks uncommitted changes.
  - A red dot marks a repo that can’t be found.
- The top button expands the rail into the full list (272 px): search, Pinned and Recent groups, each row showing owner/name, current branch and state (“4 changes · ↓ 2”), plus Clone…, Open… and New… at the bottom.
- Gear (bottom): opens Settings. Plus (bottom): opens the start screen’s Clone / Open / Create.
- Drag tiles to reorder; right-click a tile for Pin, Show in file manager (labelled Finder, Explorer or Files per OS), Open in terminal, Remove from list.
- The expanded state is remembered per window.

### Sidebar (232 px)

From top to bottom:

1. **Operation card** (only while one is in progress): amber card such as “Merging feature/sso · 2 conflicts left”. Clicking it returns to the operation from any screen.
2. **Sections:** Changes (count of changed files), History, Branches (count), Pull requests (open count), Activity. The current section is highlighted.
3. **Branches:** the 5 most recent local branches; current branch in bold; colour dot = its lane colour in the graph; “All” opens Branches.
4. **Remotes, Stashes, Tags:** short lists linking to Tags & remotes and Stashes.
5. **Tools:** Worktrees, Submodules, Find a bad commit (bisect).

With no repository open, the section items are disabled and a one-line hint explains why.

### Console bar (30 px) and panel

- The bar always shows the last Git command, a green or red status dot and the command count.
- Clicking **Console** (or ⌃\`) opens the panel (300 px, resizable). Section 10 describes it.

### Window sizes

| Width | Layout |
| --- | --- |
| ≥ 1280 px | Full layout as designed |
| 1000–1279 px | Split diff falls back to unified; history list narrows to 320 px |
| < 1000 px | Rail hides (repo switcher in a tile at the top of the sidebar); sidebar becomes 56 px icons with tooltips; changed files move into a dropdown |

## 3. Getting a repository

A new user reaches a working repository in under a minute: a three-step setup, then Clone, Open or Create from the start screen. Git needs no setup: Ramoso ships its own.

### First launch (setup)

Three steps in a card with a step list on the left. Back, Skip for now and Continue sit at the bottom.

1. **Your identity.** Name and email are pre-filled from `~/.gitconfig`.
   - Optional rule “Use a different email for work repositories”: a folder plus an email. It is saved as a Git `includeIf`, so the terminal uses it too.
   - Validation: the email must contain @. An empty name blocks Continue.
2. **Accounts.** Connect GitHub and/or GitLab through a browser sign-in, or any other host with a pasted personal access token. Ramoso detects existing SSH keys and says, for providers that support it, whether each is on the account.
3. **Appearance.** Light, Dark or Match system, with live previews.

Skipped steps reappear as a dismissible banner on the start screen.

> Design note: the “Git not found” screen in the design canvas is no longer used, and the first-launch screen’s Git step is removed.

### Start screen (no repository open)

- Greeting, then three action cards: Clone (⌘⇧C), Open (⌘O), Create (⌘N).
- **Recent:** each row shows the repo tile, owner/name, path, current branch, state (“4 changes”, “14 to pull”, Clean) and when it was last opened.
  - Star toggles Pinned.
  - Click opens the repo.
  - The filter box searches names and paths.
- **Accounts:** connection status per provider, with Connect buttons.
- **Drop zone:** dropping a folder behaves like Open.

### Clone

A dialog with source tabs: URL, plus one tab per connected host (GitHub, GitLab, …).

- **URL:** paste an HTTPS or SSH URL from any host. After 300 ms Ramoso runs `git ls-remote` and shows a preview card: host, owner/name, private or public, default branch, approximate size, and whether you have write access. Invalid or inaccessible URLs show the Git error in plain words, with Sign in if access was denied.
- **Host tabs (GitHub, GitLab, …):** pick an organisation (a group on GitLab), then search its repos. Repos already cloned show “Cloned” and open instead of cloning again. An unconnected provider shows a Connect prompt.
- **Clone into:** filled in as `<default folder>/<owner>/<name>`, editable, with Browse. A warning appears if the folder exists and isn’t empty.
- **Options:** branch to check out, shallow clone, include submodules (detected from `.gitmodules`), download Git LFS files (detected from `.gitattributes`).
- **Clone (⌘↵)** switches the dialog to a progress view:
  - Steps: Connecting, Receiving objects (progress bar with objects, MB, speed and time left), Resolving deltas, Checking out, Submodules and LFS.
  - The raw Git output streams below.
  - **Continue in background** closes the dialog and shows progress on the repo’s rail tile.
  - **Cancel clone** stops it and removes the partial folder.

### Open a folder

- **The folder is a repository:** it opens immediately.
- **It isn’t, but contains repositories** (searched 2 levels deep): a list with checkboxes.
  - Repos already in Ramoso are greyed out.
  - The primary button reads “Add N repositories”.
  - Turning the folder itself into a repository is tucked into a collapsed section, with a warning about nesting.
- **It isn’t a repository and contains none:** offers “Make this folder a repository”, which pre-fills Create with that location.

### Create

- **Fields:** name, location (with a full-path preview), .gitignore template, license, initial branch (default `main`), Add README, Make the first commit.
- **Publish to a host** (switch, on when an account is connected): host (if several are connected), owner, Private or Public (Private pre-selected), description.
- **Commands Ramoso will run:** a collapsible list of the exact commands.
- The primary button reads “Create and publish”, or “Create” when publishing is off. Name conflicts at the location or on the host are flagged inline before submitting.

## 4. Changes and committing

Changes is where work becomes commits: pick exactly which files, hunks or lines to stage, write the message, commit. Ramoso watches the folder and refreshes within 200 ms of a save.

### File list (left, 360 px)

- Two groups, **Unstaged** and **Staged**, each with a count and a Stage all / Unstage all button.
- **Each row:**
  - Checkbox: ticking stages the whole file; unticking unstages it.
  - Status badge: M modified, A added, D deleted, R renamed, ? untracked, ! conflicted.
  - File name with its folder in grey.
  - Partial-staging note, such as “1 of 2 hunks”.
- **Click** selects a file and shows its diff. **Space** toggles staging. **⌫** asks to discard.
- **Right-click:** Discard changes, Stash this file, Ignore (adds a line to .gitignore), Open in editor, Show in file manager, Copy path.
- **Stash…** in the header opens the stash dialog. Options: include untracked files, only what’s staged.

### Diff with line staging (right)

- **Header:** file path, a “Partly staged” badge when it applies, and a switch for Unstaged / Staged / Both.
- **Hunk card:** each hunk is a card. Its header shows the line range and the enclosing function, plus Stage hunk and Discard hunk.
- **Selecting lines:**
  - Changed lines have a checkbox in the gutter.
  - Click to pick one line, Shift-click for a range, or drag across the gutter.
  - Selected lines get a white left edge and a stronger green or red background.
  - A floating bar appears: “N lines selected · Discard (⌫) · Stash lines · Stage lines (S)”. Esc clears the selection.
- Context lines can’t be selected. Picking only part of a modified pair stages it as a deletion plus an addition, exactly like `git add -p` editing.

### Commit composer (bottom of the file list)

- **Summary:**
  - Required. The counter shows “38 / 72” and turns amber past the length set in Settings (default 72).
  - If the branch name contains a ticket ID and the setting is on, the ticket prefix is suggested.
- **Description:** optional. Wraps at 72 characters on commit.
- **Amend:** a switch. On: the summary and description are pre-filled from the last commit, and the button reads “Amend commit”. If that commit was already pushed, a warning says a force push will be needed.
- **Co-author chips:**
  - Suggested from people who touched the same files this week.
  - Each chip adds a `Co-authored-by` trailer, and × removes it.
- **Commit split button:**
  - The main part reads “Commit N files” (⌘↵). It is disabled when nothing is staged or the summary is empty.
  - The menu offers Commit and push, Commit without hooks (if allowed in Settings) and Commit to a new branch….
- **Hooks:** pre-commit hook output streams into the Console. A failed hook stops the commit, keeps the message, and shows the hook’s output inline above the button.
  - Every Git hook (pre-commit, commit-msg, pre-push, post-checkout, …) runs as it would in the terminal: from the repository folder, with the user’s login-shell environment on Linux and macOS, so tools such as node, asdf or nvm shims resolve the same way. On Windows, the user’s normal environment is used.

### Nothing to commit

- A centred empty state reads “Nothing to commit”, explains that the working tree matches the last commit, and says how many commits are unpushed.
- **Next steps:** Push N commits (primary), Create pull request, Open in editor.
- **Cards:** the last commit (opens it in History) and stashes on this branch (opens Stashes).

### Discard changes (confirmation)

- Lists every affected file with its line counts.
- Staged changes are kept. Untracked files go to the system Trash.
- A safety line says a snapshot is saved first and can be restored from Activity.
- **Buttons:** Discard changes (red) and Cancel. Optional “Don’t ask again for single files”.
- Discarding one hunk or selected lines doesn’t ask for confirmation. It shows an undo toast instead.

### Stashes

- **List (left):** message, `stash@{n}`, original branch, age, file count, and whether untracked files are included.
- **Preview (right):** the stash’s files as tabs with a diff. Above it, a note on whether it applies cleanly to the current branch, checked by a dry run.
- **Actions:**
  - Apply keeps the stash.
  - Pop applies it, then drops it.
  - New branch from stash creates a branch at the stash’s base and applies it there.
  - Drop removes it: red, with an undo toast, and recoverable from Activity.
- **Conflicts:** if applying would conflict, the button reads “Apply (1 conflict)” and continues into the conflict resolver.

## 5. History and the diff viewer

History pairs the commit graph with the commit’s diff. The diff viewer is shared by History, Changes, Stashes and pull requests, so every rule below applies everywhere.

### Commit list and graph (460 px)

- **Toolbar:**
  - The expand icon opens the Full-width log.
  - Scope tabs switch between All branches and Current.
  - The Filter box searches message, SHA, author and ticket ID as you type.
- **Row (40 px):**
  - Graph lanes: blue for main, a stable colour per branch. Merges curve down from the merge commit into the second parent’s lane; branches curve up from the fork point. No lane line runs above the newest commit.
  - Ref badges: local branches, remote branches and tags; HEAD is marked.
  - The message.
  - The author’s avatar and full name. The name is cut off with an ellipsis only when it doesn’t fit, and the full name shows on hover.
  - Relative time.
- **Selection:**
  - Click or ↑/↓ selects a commit.
  - ⌘-click selects several, which shows their combined diff and enables Cherry-pick N and Squash N.
  - Double-click checks out the commit, which opens the Detached HEAD state.
- **Right-click:** Copy SHA, Checkout, Create branch here, Create tag here, Cherry-pick to…, Revert, Reset current branch to here…, Rebase onto here…, Start bisect from here.
- **Loading:** commits load in pages of 200 as you scroll. Very large repos use the Loading state (below).

### Commit details

- **Header:** summary as the title, description below.
- **Meta line:** author and date, parent SHAs (clickable), file count and +/− totals.
- **SHA button:** copies the SHA. The ⋯ menu repeats the right-click actions.
- **File tabs:** one per changed file, with its status badge and counts. A file’s full history and blame are one click away (Blame, below).

### Diff viewer

| Control | Behaviour |
| --- | --- |
| Unified / Split | Split shows before and after side by side with scrolling kept in sync; it falls back to unified below 1000 px |
| Word diff (on) | Highlights changed words inside a modified line pair |
| Detect moves (on) | Moved lines are violet with “moved to N” / “moved from N” labels instead of red and green |
| Ignore whitespace | Hides whitespace-only changes and shows how many were hidden |
| Blame | Adds an author and age gutter to the diff |
| Change N of M + J/K | Jumps between changes across all files |
| Folded regions | “Show 47 unchanged lines”; click expands 20 at a time, Shift-click expands all |

Line numbers for old and new are both shown in unified view. Selecting text copies code without +/− markers.

### Special files

| File kind | What is shown |
| --- | --- |
| Image (png, jpg, gif, webp, svg) | Before and after on a checkerboard, with dimensions, size and % change. Modes: Side by side, Swipe, Onion skin, Difference |
| Binary | “Binary file changed”, old to new size, Open both, Use tool… (external diff from .gitattributes) |
| Renamed / moved | One entry “old → new” with a similarity %, showing only the changed lines |
| Large (over 5 MB or 20,000 lines) | Not rendered; Load anyway or Open in editor |
| Generated (linguist-generated, lock files) | Collapsed with its +/− counts; Expand |

### Full-width log

- Opened from the History toolbar. The collapse icon returns to the split view.
- **Table columns:** Graph, Message (with refs), Author (full name with avatar), Date (“17 Nov, 14:02”), Commit, Changes (+/−).
- **Filters:**
  - Text search.
  - Author chooser.
  - Path filter (glob).
  - Date range.
  - First parent only.
  - Hide merges.
  - The header shows the commit count for the current filter.
- The header stays fixed while you scroll. Enter on a row opens that commit in the split view.

### Blame and file history

- **Left: file history.** Every commit that touched the file, including across renames (the rename is shown on its row). Clicking one shows the file as of that commit.
- **Right: blame.**
  - Lines are grouped by commit. The first line of each group shows the author’s full name, the commit message and the age.
  - A 3 px bar shows the age: lighter is newer. The age is also written as text.
- **Follow moved lines (on):** credits moved code to its original author and marks it “moved”.
- **Skip formatting commits (on):** uses `.git-blame-ignore-revs`.
- **Hovering a group:** shows a card with author, date, SHA and message, plus Open commit and Blame before this. Blame before this re-blames from that commit’s parent.

### Detached HEAD

- Shown when a commit or tag is checked out instead of a branch. The branch switcher reads “v2.8.0 · detached”.
- **Amber banner:** explains that new commits here won’t belong to any branch. Actions: Create branch here… (primary) and Back to main.
- The graph marks HEAD on the commit in amber.
- Committing here first asks for a branch name. This can be turned off in Settings.

### Loading a large repository

- **Blue banner:** “Reading history of acme/monorepo” with commits read so far, a progress bar, and Load in background.
- The newest commits are usable immediately. Rows still loading show grey placeholder bars.
- After the first load, Ramoso writes a commit-graph file so the next opening takes under a second.

## 6. Branches, tags, remotes, worktrees, submodules

These screens manage the repository’s structure. Each list shows its state against the remote at a glance and offers cleanup.

### Branches

- **Header:**
  - Title, “Compared with main · fetched 2 min ago”.
  - Tabs: Local, Remote, Tags.
  - Filter, and New branch (primary).
- **Cleanup callout:** appears when some branches are fully merged and their remote is gone. It names them and offers Review or Delete N branches.
- **Table columns:**
  - Branch: name, a HEAD badge on the current one, and a sub-line with its tracking branch and any worktree.
  - Behind · Ahead versus main: a two-sided bar with numbers.
  - Last commit.
  - Author: full name, cut off only when it doesn’t fit.
  - Updated.
  - Pull request: status chip linking to the PR.
  - A ⋯ menu.
- **Row actions:**
  - Double-click checks out the branch. Uncommitted changes come along when they don’t clash; otherwise Ramoso offers to stash them.
  - The ⋯ menu: Merge into current, Rebase current onto this, Compare with current, Rename, Set upstream, Create pull request, Delete….

### Delete a branch (confirmation)

- **Merged branch:** deleted without a dialog; an undo toast appears.
- **Unmerged branch:** a dialog warns how many commits exist only on it and lists the first three.
- **“Also delete on \<remote\>”:** off by default. A note appears if an open pull request uses the branch.
- Safety line: restorable from Activity for 30 days. The button is red.

### Reset a branch (confirmation)

- **Shows the move:** “main c42e9aa → 98ad0c3 · 2 commits leave the branch”.
- **Three modes, each named by its outcome with the Git flag in small text:**
  - Keep the changes, staged (`--soft`).
  - Keep the changes, unstaged (`--mixed`, the default).
  - Throw everything away (`--hard`, red). Ramoso saves a snapshot first.
- Footer: “Undo from Activity for 30 days”.

### Tags and remotes

- **Remotes table:** name, URL, last fetched, Fetch and Edit per row.
  - Header actions: Prune deleted branches, Add remote….
  - Editing a URL tests it with `ls-remote` before saving.
- **Tags table:** tag, message (or “Lightweight tag”), commit and date, status, action.
  - Statuses: Signed · verified, Unsigned, Only on this computer.
  - Unpushed tags get Push tag. Release notes opens the provider’s release page.
  - Create tag… asks for name, message, sign (on when signing is set up) and push now.

### Worktrees

- Lets you work on several branches at once, each in its own folder, without stashing.
- **Table columns:** folder, branch, status (Clean or N changes), last used, Open in new window, Remove.
- The main folder is marked “this window” and can’t be removed.
- **New worktree:** branch (existing or new) and a folder suggested as `<repo>-<branch>`. It opens in a new window.
- **Remove:** if the worktree has uncommitted changes, Ramoso asks to stash or discard them first.

### Submodules

- **Table columns:** path and URL, pinned commit, status, action.
- **Statuses:**
  - N commits behind → Update.
  - Has unpushed commits (amber) → Open, Push.
  - Not initialised → Download.
  - Up to date.
- **Selecting a row:** shows the commits between the pinned commit and its remote branch.
- **Updating:** moves the pin and leaves the change staged in Changes, so it can be reviewed before committing.
- Header actions: Update all, Add submodule….

## 7. Sync: fetch, pull, push

Sync never loses work silently: every case where your work and the server’s disagree stops and explains the choice, with the safe option pre-selected.

### Background fetch

- Every 5 minutes, when the window gains focus, and on demand. The interval can be changed in Settings.
- Updates ahead/behind counts in the title bar, sidebar and rail.
- Background commands show at reduced opacity in the Console and can be hidden there.

### Pull

| Situation | What happens |
| --- | --- |
| Behind only | Fast-forward; toast “Pulled 2 commits” with Undo |
| Diverged, strategy remembered | Uses it (rebase or merge) without asking |
| Diverged, no strategy | Opens the Diverged dialog |
| Uncommitted changes that clash | Offers “Stash, pull, re-apply” (default) or Cancel |
| Pull causes conflicts | Opens the merge or rebase conflict flow (section 8) |

### Branches have diverged (dialog)

- **A small graph:** the common base, your N commits in blue, the server’s M commits in violet, with counts.
- **Choices:**
  - Put my commits on top (rebase): pre-selected and marked Recommended.
  - Merge.
  - Fast-forward only: disabled, with the reason.
- **Conflict prediction:** an amber line names files changed on both sides, such as “Both sides changed src/session/store.ts, so a conflict is likely”.
- **Remember my choice for this repository:** saves the strategy to the repo’s Git config.
- **Buttons:** Pull and rebase (or Pull and merge), Cancel. Footer: “Undo from Activity at any time”.

### Push

- **First push of a branch:** sets the upstream automatically and offers “Create pull request” in the success toast.
- **Push rejected (the server has new commits):**
  - The dialog reads “Your push was rejected” and lists the new commits on the server with author and age.
  - Choices: Pull, then push (recommended, pre-selected); Pull with a merge commit, then push; Force push… (red).
  - Git’s raw reason (“rejected (fetch first)”) is shown in small text.

### Force push (confirmation)

- **What moves:** “origin/feature/sso will move from 6be0f13 to c90a1d4”.
- **What the server loses:** “These commits will be removed from the server”, with authors.
- **Other people’s commits:** if any removed commit is by someone else, a red notice names them and suggests asking first.
- **Type to confirm:** the branch name must be typed before the button enables.
- **Defaults:**
  - Use `--force-with-lease` (on): stops if the server changed again since your last fetch.
  - Keep a backup of the old tip (on): saved as `backup/<branch>-<date>` and in Activity.

### Sign-in failed

- **Shown when:** a fetch, pull or push fails with an authentication error.
- **Summary:** remote URL, sign-in method, and the problem in plain words (“Token expired on 15 Nov”).
- **Fixes:**
  - Sign in with \<host\> in your browser (recommended, for hosts with a browser sign-in): stores the new token in the system password store (Secret Service on Linux, Credential Manager on Windows, Keychain on macOS).
  - Switch this repository to SSH: offered only when your key is already on the account.
  - Paste a new token (works for any host).
- **Effect:** sync for this repo pauses until fixed; local work is unaffected. Show Git output opens the Console entry.

### Offline

- **Banner:** “You’re offline. Everything local still works. 1 push is waiting.” Actions: Retry now, Show queue.
- **Queuing:** Push shows “N queued”. Queued pushes run automatically when the connection returns, in order, with a toast for each.
- **Retrying:** the Console shows the failing command and the next retry time (30 s, backing off to 5 min).

## 8. Conflicts and rewriting history

Merge, rebase, cherry-pick, revert and bisect are “operations”. They can pause, they show an amber card in the sidebar until finished, and they can always be aborted back to the exact starting point.

### Merge conflict resolver

- **Banner:**
  - “Merging feature/sso into main”, with how many files merged cleanly versus had conflicts.
  - A progress bar, “3 of 5 resolved”.
  - Abort merge, and Commit merge, which stays disabled with “· N left” until everything is resolved.
- **File list:** each file shows a check when resolved, an amber count when not, or “auto-resolved · review”.
- **Three panes (top):**
  - Ours (blue), Base (grey: the common ancestor) and Theirs (amber), each labelled with its branch and SHA.
  - Conflicting lines have checkboxes to pick them line by line. Take all copies a whole side.
- **Result (bottom, editable):**
  - Each conflict has a decision bar: Ours, Theirs, Both, Combine, Edit.
  - Combine appears when the two sides edited different lines. Ramoso pre-selects it and explains why.
  - Every result line is labelled with where it came from (ours or theirs). Unresolved spots are hatched.
  - Undo (⌘Z) steps back through choices. Open in editor hands the file to the external editor and watches for the save.
- **“Why it conflicts” panel (right):** the commits on each side that touched these lines, linking to History.
- **“N chunks resolved for you”:** whitespace-only and identical-on-both-sides changes are resolved automatically but must be reviewed (Review N chunks).
- **Shortcuts:** ⌥1 ours, ⌥2 theirs, ⌥3 both, N next conflict.

### Interactive rebase (plan)

- **Banner:** “feature/sso onto main · 5 commits, oldest first”. If commits are already pushed, an amber note says the push will use `--force-with-lease`.
- **Rows:**
  - A drag handle to reorder; ⌥↑ / ⌥↓ also move a row.
  - The action menu: Pick, Reword, Edit, Squash, Fixup, Drop, also set with the P R E S F D keys.
  - SHA, message, author, +/−.
- **How each action shows:**
  - Reword turns the message into an input.
  - Squash and Fixup indent the row under the commit above, with a chip “Folds into …”.
  - Drop strikes the row through.
  - `fixup!` commits are placed and set to Fixup automatically.
- **“After the rebase” panel:** the resulting commit list, such as “5 commits become 3”.
- **Dry run:** each step marked clean or “conflicts in file”, with Preview the conflict.
- **Buttons:** Start rebase (⌘↵), Cancel.

### Rebase paused on a conflict

- **Banner:** “Rebase paused at step 3 of 3”, naming the commit. Continue (disabled with “· N left” until resolved), Skip this commit, Abort rebase (red).
- **Step list:** each commit marked applied (old → new SHA), stopped here, or waiting.
- **Files with conflicts:** each with Resolve (opens the resolver) or a check.
- **“Which side is which” card:** Upstream (main, blue) versus Your commit (amber). The resolver uses these names during a rebase, because Git swaps ours and theirs.
- Aborting restores the branch exactly. Activity keeps the old tip for 30 days either way.

### Cherry-pick and revert

- **Starting:** select commits in History, then right-click → Cherry-pick to… (choose the target branch) or Revert.
- **Paused:** the same layout as the paused rebase. The banner reads “Cherry-pick paused at 2 of 3”, with the step list and conflict files.
- **Options:**
  - Add “cherry picked from commit …” (`-x`, on).
  - Keep original authors and dates (on).
- Revert uses the same screen. Each revert gets a default message “Revert “\<summary>””.

### Find a bad commit (bisect)

- **Starting:** from Tools, from the palette, or from History (right-click → Start bisect from here). It asks for a good commit and a bad one (defaults: the latest tag and HEAD).
- **Banner:** a plain explanation, about how many steps remain, and Stop and go back to the branch.
- **Range strip:** one dot per commit: green works, red broken, grey untested, ringed = checked out now. A legend sits underneath.
- **Current commit card:** message, author, SHA, an instruction, and three buttons:
  - Works (green).
  - Broken (red).
  - Can’t test this one (skip).
- **Let a command decide:** a command field (exit code 0 means it works) and Run automatically, which finishes on its own and streams to the Console.
- **Result:** the first bad commit, with Open commit, Revert it and Copy link, and the branch restored.

## 9. Activity and undo

Every action that changes the repository can be undone for 30 days, including discards and hard resets. Pushed changes are the only exception: they show as “Pushed · shared” and can only be reverted.

### How undo works

- Ramoso reads the reflog for branch moves and HEAD changes.
- Before anything that would destroy uncommitted work (discard, hard reset, checkout with clashes, stash drop), Ramoso saves a snapshot: a hidden commit under `refs/ramoso/snapshots/`.
- Snapshots older than 30 days are pruned. The retention period is configurable.
- **Undo (⌘Z)** reverses the most recent action, whatever screen is open. A toast confirms what was undone and offers Redo.
- **Dependent steps:** undoing a step that later steps depend on also undoes those steps. The detail panel says so first.

### Activity screen

- **Header:** title, the 30-day rule, and filter tabs: All, Commits, Branch moves, Stashes, Discarded.
- **Timeline:** grouped by day.
- **Each row:**
  - Icon, title (“Reset main to 98ad0c3”), one-line detail, time.
  - A contextual button: Undo, Undo commit, Restore lines, Restore branch, Apply, Switch back, or Abort rebase for a running operation.
- **Detail panel (selected row):**
  - Type and time.
  - Branch moved: from → to SHA.
  - The commits that left the branch.
  - What happened to the working copy.
  - Buttons: Undo this (primary), Save commits as branch, Cherry-pick….

### Undo toasts

- Shown after every repository-changing action, bottom right, for 8 seconds. Hovering pauses the timer.
- **Content:** what happened, a consequence line (“2 commits left the branch. They’re kept in Activity for 30 days.”) and Undo ⌘Z.
- Several toasts stack, up to 3. Older ones collapse into “+2 more in Activity”.

## 10. Command palette, Console, pull requests

### Command palette (⌘K)

- **Input:**
  - Fuzzy search across actions, branches, commits (by message or SHA), files and repositories.
  - A context chip (“on feature/sso”) shows what the actions apply to.
- **Result groups:** Actions, Branches, Commits, Files, Recent (including Undo of the last action). Each row has an icon, label, detail and shortcut.
- **Preview panel:** for risky actions (rebase, reset, force push, drop), a dry run shows what will happen before ↵:
  - Commits to replay.
  - Whether a force push is needed.
  - Predicted conflicts.
- **Keys:**
  - ↑↓ move.
  - ↵ runs.
  - ⇥ picks a target (such as the branch to rebase onto).
  - Esc closes.
- **Prefixes:** `>` actions only, `@` branches, `#` commits, `/` files.

### Git console

- **Bar:** always visible. It shows the last command, a status dot and the count; Console toggles the panel.
- **Each entry:**
  - Time and a ✓ or ✕ status.
  - The command in monospace, prefixed by `$`.
  - Source chip: Fetch button, Commit selected, background, and so on.
  - Duration and exit code.
- **Output:** stdout and stderr appear under each entry. Errors are highlighted red.
- **Known errors:** Ramoso adds a plain explanation and fixes, such as “The remote has 2 commits you don’t have yet” with Pull, then push and Force push….
- **Toolbar:**
  - Filter.
  - Show background commands (on).
  - Errors only.
  - Copy log.
  - Clear (session only).
  - Close.
- **Command line:** `$ git` runs any Git command in the current repository. Its output appears as a normal entry, and the app refreshes afterwards. Destructive commands typed here get the same confirmation as their buttons.
- **Secrets:** tokens and passwords are masked in the output.

### Pull request review

Works with every host in the provider layer (GitHub and GitLab first). On GitLab the UI says “merge request” and uses `!412` numbering; everywhere else it says “pull request”. Repos whose remote host has no connected provider show a Connect prompt instead of the list.

- **Sidebar:** Pull requests shows the open PRs for the repo: yours, review requested, and all.
- **Header:** title and number, status chip, `head → base`, author, commit count, checks summary (CI pipelines on GitLab), Open on \<host\>.
- **Tabs:** Conversation, Commits, Files.
- **Files tab:**
  - A file list with unresolved comment counts.
  - The diff viewer with inline threads under the commented line.
  - Hovering a line shows +, which starts a comment.
  - Reply adds to your pending review. Resolve conversation is available to the PR author and reviewers.
- **Review panel:**
  - Reviewers with their state (Approved, Commented, Changes requested).
  - Checks, each with status and duration.
  - “Your review · N pending comments”, with Comment, Request changes and Approve.
- **Check out locally:** fetches the PR branch, or reuses an existing worktree, so the code can be run.

## 11. Settings, themes, system errors

### Settings

Settings are opened from the gear in the rail or with ⌘,. A left menu holds General, Accounts, Commits & signing, Diff & editor, Appearance, Shortcuts and Advanced, plus a search box that filters every option. Changes save immediately, with no Save button. Options that write to Git config say so and show their scope (global or this repo).

| Group | Options |
| --- | --- |
| Identity | Name, email (global); per-folder email rules saved as `includeIf` |
| Signing | Sign commits and tags (SSH key by default, GPG supported); key picker with an “Added to \<host\>” check per connected host that supports it |
| Commit messages | Summary length guide (72); suggest co-authors; ticket prefix from branch name |
| Hooks | Run pre-commit hooks; allow “Commit without hooks” |
| Theme | Light / Dark / Match system, shown as preview cards |
| Layout | Density (Comfortable / Compact); code font and size (Geist Mono 12.5) |
| Diff | Default layout; highlight changed words; detect moved lines; ignore whitespace; context lines (3) |
| Tools | External editor (VS Code); merge tool (Ramoso built-in or external) |
| Sync | Background fetch interval (5 min); default pull strategy; prune on fetch |
| Accounts | Connected hosts (GitHub, GitLab, others by token); sign in, sign out, replace token |
| Advanced | Git: bundled version (read-only); Activity retention (30 days); large-file threshold (5 MB) |

### Light theme

- Same layout and components. Zinc neutrals are inverted, and accents darken so text stays readable on white (added text #16a34a, removed #dc2626).
- The logo switches to its light palette.
- “Match system” follows the OS setting live.

### Repository moved or deleted

- **Shown when:** a repo in the list can’t be found on disk. Its rail tile gets a red dot.
- **Content:** an empty state reading “acme/infra isn’t where it used to be”, with the old path.
- **Actions:** Locate folder…, Clone again (Clone dialog pre-filled), Remove from list.
- **Smart match:** if a repo with the same remote is found elsewhere, “Use this one”.

### Leftover lock file

- **Shown when:** a Git command fails because `.git/index.lock` exists.
- **Details:** the lock file path, how long ago it was created, and whether any Git process is running.
- **No Git process running:** Remove lock and retry is the primary action. If a process is running, the primary action is Wait and retry, and removal asks for confirmation.
- The failed command, such as the commit, is retried automatically once the lock is gone.

## 12. User scenarios

Fourteen scenarios cover the everyday and the risky paths. Each lists the steps and the expected outcome that acceptance testing should check.

### UC-1 First launch and first clone

Goal: a new user gets a working repository.

1. Opens Ramoso. Name and email are pre-filled from `~/.gitconfig`.
2. Adds a work email for `~/work/acme/` and connects GitHub in the browser.
3. On the start screen, chooses Clone → GitHub tab, searches “billing” and selects acme/billing.
4. Keeps the suggested folder and clicks Clone. Watches progress, then clicks Continue in background.

Expected: the repo opens in History when done. The work email applies to it. The Console lists the `git clone` command.

### UC-2 Commit only part of a file

1. In Changes, selects `saml.ts`.
2. Shift-selects three added lines and presses S.
3. Writes the summary and presses ⌘↵.

Expected: only those three lines are committed. The file stays in Unstaged with the remaining hunk. A toast offers Undo commit.

### UC-3 Review what a teammate changed

1. In History, filters by “refresh” and opens the commit.
2. Moved lines show in violet. Presses J to jump to the next change.
3. Opens Blame on the file. The moved lines keep their original author.

Expected: the reviewer can tell new code from moved code without reading the whole file.

### UC-4 Pull when branches have diverged

1. Clicks Pull and the Diverged dialog opens.
2. Sees the graph and the conflict warning, keeps “Put my commits on top”, ticks Remember my choice.
3. Clicks Pull and rebase.

Expected: the rebase runs. If it conflicts, the rebase-paused screen opens. Later pulls in this repo don’t ask again.

### UC-5 Resolve a merge conflict

1. Merges feature/sso into main. The resolver opens with 5 conflicts, 2 already auto-resolved.
2. For non-overlapping edits, accepts the pre-selected Combine.
3. For the TTL conflict, presses ⌥2 to take theirs, then reviews the 2 auto-resolved chunks.
4. Clicks Commit merge, now enabled.

Expected: the merge commit is created and the sidebar card disappears. Abort at any point restores main exactly.

### UC-6 Clean up a branch before review

1. Opens Interactive rebase from the palette (⌘K → “reb”).
2. Fixup is set automatically for `fixup!` commits. Drops a debug commit and rewords one.
3. Checks that the dry run is clean and starts the rebase.

Expected: “5 commits become 3”. Push uses `--force-with-lease` after confirmation. Activity can undo the whole rebase.

### UC-7 Recover from a mistaken hard reset

1. Resets main with “Throw everything away”.
2. Realises that 4 uncommitted files are gone and presses ⌘Z, or opens Activity → Undo this reset.

Expected: both the 2 commits and the 4 uncommitted files are restored from the snapshot.

### UC-8 Push is rejected

1. Clicks Push and the rejection dialog lists 2 new commits from teammates.
2. Keeps “Pull, then push”.

Expected: pull and push run in sequence. Nobody’s commits are lost. Force push needs the separate typed confirmation.

### UC-9 Expired token

1. A background fetch fails and the Sign-in failed dialog appears.
2. Chooses Sign in with GitHub in your browser.

Expected: the token is stored in the system password store, and sync resumes with the queued fetch. Local work was never blocked.

### UC-10 Find the commit that broke login

1. Tools → Find a bad commit, with good = v2.8.0 and bad = main.
2. Enters `npm test -- login` and clicks Run automatically.

Expected: after about 4 steps the first bad commit is shown, with Revert it. The original branch is restored.

### UC-11 Hotfix while mid-feature

1. With uncommitted work on feature/sso, opens Worktrees → New worktree for release/2.8.
2. Fixes the bug in the new window, commits, pushes and removes the worktree.

Expected: feature/sso was never stashed or touched.

### UC-12 Bring a fix to a release branch

1. In History, ⌘-selects 3 commits → Cherry-pick to… release/2.8.
2. The second commit conflicts. Resolves it from the paused screen and clicks Continue.

Expected: 3 new commits on release/2.8, each with a “cherry picked from” line.

### UC-13 Review a pull request

1. Opens Pull requests → #412 → Files.
2. Replies to Mira’s thread, adds a comment on another line, clicks Approve.

Expected: the comments and approval are posted together as one review on GitHub.

### UC-14 Open a folder of projects

1. Drags `~/work/acme` onto the start screen.
2. Ramoso finds 5 repositories, 2 already known. Clicks Add 3 repositories.

Expected: 3 new tiles appear in the rail. Nothing is initialised in the parent folder.

## 13. Shortcuts, accessibility, copy, open questions

### Keyboard shortcuts (macOS; Ctrl replaces ⌘ on Windows and Linux)

| Shortcut | Action |
| --- | --- |
| ⌘K | Command palette |
| ⌘Z / ⌘⇧Z | Undo / redo the last repository action |
| ⌘1 … ⌘5 | Changes, History, Branches, Pull requests, Activity |
| ⌘⇧O | Switch repository |
| ⌘B | Switch branch (⌘↵ in the picker creates one) |
| ⌘⇧C / ⌘O / ⌘N | Clone / Open / Create |
| ⌘⇧F / ⌘⇧L / ⌘⇧P | Fetch / Pull / Push |
| ⌘↵ | Commit; also confirms the primary button in dialogs |
| Space / S | Stage file / stage selected lines |
| ⌫ | Discard selection (asks first for whole files) |
| J / K | Next / previous change in a diff |
| N | Next conflict |
| ⌥1 / ⌥2 / ⌥3 | Take ours / theirs / both |
| P R E S F D | Rebase actions: pick, reword, edit, squash, fixup, drop |
| ⌥↑ / ⌥↓ | Move a rebase row |
| ⌃\` | Toggle the Console |
| ⌘, | Settings |
| Esc | Close dialog or palette, clear selection |

### Accessibility

- **Controls:** every control is a real button, link or input with a visible label or `aria-label`; icon-only buttons have tooltips too. Tab order follows the visual order, and focus rings are always visible.
- **Contrast:** text meets 4.5:1 in both themes, including 11–12 px grey labels.
- **Never colour alone:** diff lines carry +/−/↕ markers; status dots have text; graph lanes are paired with ref badges; conflict sides are named.
- **Colour pairs:** blue/amber is used for ours/theirs instead of red/green, so the two stay distinct for colour-blind users.
- **Screen readers:** progress bars expose `aria-valuenow`; dialogs trap focus and return it on close; toasts use `role=status`.
- **Reduced motion:** turns off graph and toast animations.
- **Sizes:** buttons are at least 28 px tall in Compact density and 32 px in Comfortable.

### Copy guidelines

- **Outcome first:** say what will happen, then the Git term (“Keep the changes, unstaged · --mixed”).
- **Buttons name the action:** “Delete branch”, “Pull, then push” — never “OK” or “Yes”.
- **Errors:** what happened, why, what to do, then Git’s raw message in small monospace.
- **Formats:** dates as “17 Nov, 14:02”, with relative times (“2 h”) in lists. Full author names; SHAs shortened to 7 characters.
- **Tone:** calm and specific. No exclamation marks, no blame (“Your push was rejected”, not “You forgot to pull”).

### Open questions

- [ ] Is 30 days the right default for Activity snapshots, and how much disk may they use?

### Decided

- [x] Git: Ramoso always uses its own bundled Git, never the system one.
- [x] Hosts: a provider layer; GitHub and GitLab first (including pull/merge request review), Gitea, Forgejo and Bitbucket later.
- [x] Title bar: custom on every OS.
- [x] AI-suggested conflict resolutions: out of scope.
- [x] Pricing: not applicable; Ramoso is open source.
