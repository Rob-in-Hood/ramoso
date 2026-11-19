# Spike (milestone 0): reference only

**Don’t merge this branch into `main`.** This is throwaway code that answered seven risky questions before Ramoso was built on them. Milestone 1 copies the build setups from here into `build/` on `main`, cleaned up; the rest was written to test, not to ship.

The results, the design changes they caused, and the known limits are in `docs/architecture.md` §11 on `main`. The docs on this branch are older and out of date.

## What’s here

| Folder | What it is | Reused in milestone 1 |
| --- | --- | --- |
| `git-linux/` | Dockerfile for a static, relocatable Git 2.56.1 (musl, our own static curl) | Yes |
| `app-linux/` | Linux AppImage build: Arch container, quick-sharun, build script, desktop entry | Yes |
| `app-windows/` | Windows NSIS installer, cross-compiled on Linux with cargo-xwin | Yes |
| `tauri-app/` | Test app with one button that checks the bundled Git, hooks, PATH and HTTPS | No |
| `cred-helper/` | Test Git credential helper on the OS password store | No |
| `diff-bench/` | Benchmark of the `similar` diff library on a 20,000-line file | No |

Build output and downloaded binaries (`target/`, `git-linux/out/`, `git-windows/`, `app/`) are ignored and not committed.

## Rebuilding

From the repository root, on Linux with Docker (classic builder, no buildx needed):

```sh
# Static Linux Git
DOCKER_BUILDKIT=0 docker build -t ramoso-git-spike spike/git-linux
id=$(docker create ramoso-git-spike /x) && docker cp "$id":/ spike/git-linux/out && docker rm "$id"

# Build images, then the AppImage and the Windows installer (written to spike/app/)
cd spike/tauri-app
pnpm run docker:image:linux && pnpm run linux:appimage
pnpm run docker:image:windows && pnpm run windows:setup
```

The Windows build also needs the dugite-native Windows Git unpacked into `spike/git-windows/` (dugite-native release v2.56.1, `windows-x64`, checked against its `.sha256`).

## Known issues

These were found in review and are fixed in milestone 1, not here: unused template leftovers, a `null` Content Security Policy, a few swallowed errors, duplicated `docker run` flags, and Git LFS copied into `git-linux/out/` by hand instead of fetched with a checksum.
