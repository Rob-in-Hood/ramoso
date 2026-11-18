#!/bin/sh
# Builds spike/app/ramoso-spike.AppImage inside the ramoso-spike-linux image. Run from spike/tauri-app.
set -eu

work=.cache/appimage
out=../app

# Start clean; replace the old shortcut or AppImage.
rm -rf "$work"
mkdir -p "$work" "$out"
rm -f "$out/ramoso-spike.AppImage"

# Frontend packages from the lockfile.
pnpm install --frozen-lockfile --store-dir .cache/pnpm-store

# Compile only; quick-sharun does the packaging.
pnpm tauri build --ci --no-bundle

# The icon file must match the desktop entry's Icon= name.
cp src-tauri/icons/icon.png "$work/ramoso-spike.png"

export APPDIR="$work/AppDir"
export ICON="$work/ramoso-spike.png"
export DESKTOP=../app-linux/ramoso-spike.desktop
export OUTPATH="$out"
export OUTNAME=ramoso-spike.AppImage

# Bundle the app with every library it needs, glibc included.
quick-sharun "$CARGO_TARGET_DIR/release/tauri-app"

# Our static Git at $APPDIR/git, where the app looks in an AppImage.
cp -a ../git-linux/out "$APPDIR/git"

# Bundled graphics libraries make WebKitGTK's DMA-BUF renderer draw thick text on some systems.
echo "WEBKIT_DISABLE_DMABUF_RENDERER=1" >> "$APPDIR/.env"

quick-sharun --make-appimage
