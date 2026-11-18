# Linux AppImage build image. quick-sharun bundles glibc with the app, so the AppImage runs on any distro.
# Build: DOCKER_BUILDKIT=0 docker build -t ramoso-spike-linux spike/app-linux

FROM archlinux:base-devel-20261011.0.609700

# Packages from the Arch archive snapshot of the image date, so builds are reproducible.
# No download timeout: slow connections stall long enough for pacman to give up.
ARG ARCHIVE_DATE=2026/10/11
RUN echo "Server=https://archive.archlinux.org/repos/$ARCHIVE_DATE/\$repo/os/\$arch" > /etc/pacman.d/mirrorlist \
  && pacman -Syyuu --noconfirm --needed --disable-download-timeout \
    webkit2gtk-4.1 \
    librsvg \
    nodejs-lts-jod \
    npm \
    curl \
    wget \
    file \
    openssl \
    patchelf \
    strace \
    xorg-server-xvfb \
  && pacman -Scc --noconfirm

# pnpm, matching the version that wrote the lockfile.
RUN npm install -g pnpm@9.1.0

# Rust outside $HOME, so the build can run as the host user.
ENV RUSTUP_HOME=/usr/local/rustup \
  PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | CARGO_HOME=/usr/local/cargo sh -s -- -y --profile minimal --default-toolchain 1.99.0 \
  && chmod -R a+rwX /usr/local/rustup /usr/local/cargo

# quick-sharun (pkgforge-dev/Anylinux-AppImages), pinned by commit and checksum.
ARG QUICK_SHARUN_COMMIT=786f526e7c195b1ebe73ead56eccd10b849156bc
ARG QUICK_SHARUN_SHA256=b38ac233f0b56a03bb10565f6b074a43ce016939faee44ae8bb23d6e66c2d553
RUN curl -sSfo /usr/local/bin/quick-sharun \
    https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/$QUICK_SHARUN_COMMIT/useful-tools/quick-sharun.sh \
  && echo "$QUICK_SHARUN_SHA256  /usr/local/bin/quick-sharun" | sha256sum -c - \
  && chmod +x /usr/local/bin/quick-sharun
