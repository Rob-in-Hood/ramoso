# Windows installer build image: cross-compiles the app on Linux (NSIS only), per Tauri's Windows guide.
# Build: DOCKER_BUILDKIT=0 docker build -t ramoso-spike-windows spike/app-windows

FROM node:22.22.0-bookworm

# Base build tools.
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    file \
    pkg-config \
    libssl-dev \
    unzip \
  && rm -rf /var/lib/apt/lists/*

# pnpm, matching the version that wrote the lockfile.
RUN npm install -g pnpm@9.1.0

# Rust outside $HOME, so the build can run as the host user.
ENV RUSTUP_HOME=/usr/local/rustup \
  PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | CARGO_HOME=/usr/local/cargo sh -s -- -y --profile minimal --default-toolchain 1.99.0 \
  && chmod -R a+rwX /usr/local/rustup /usr/local/cargo

# Compilers for the Windows target; clang acts as clang-cl when started under that name.
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    scons \
    zlib1g-dev \
    lld \
    llvm \
    clang \
  && rm -rf /var/lib/apt/lists/* \
  && ln -s "$(readlink -f /usr/bin/clang)" /usr/local/bin/clang-cl

# NSIS 3.11 (Debian 12 has 3.08; Tauri needs 3.09+): build makensis into Tauri's NSIS 3.11 package.
ARG NSIS_ZIP_SHA256=c7d27f780ddb6cffb4730138cd1591e841f4b7edb155856901cdf5f214394fa1
ARG NSIS_SRC_SHA256=6ba0e34576f95282e4c7ab4347c016eaaa929fb5f20c90c6a28fb84ca1e2724e
ENV PATH=/opt/nsis-3.11/Bin:$PATH
RUN curl -sSfLo /tmp/nsis.zip https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.11/nsis-3.11.zip \
  && echo "$NSIS_ZIP_SHA256  /tmp/nsis.zip" | sha256sum -c - \
  && unzip -q /tmp/nsis.zip -d /opt \
  && curl -sSfLo /tmp/nsis-src.tar.gz https://github.com/kichik/nsis/archive/refs/tags/v311.tar.gz \
  && echo "$NSIS_SRC_SHA256  /tmp/nsis-src.tar.gz" | sha256sum -c - \
  && tar -xzf /tmp/nsis-src.tar.gz -C /tmp \
  && cd /tmp/nsis-311 \
  && scons VERSION=3.11 SKIPSTUBS=all SKIPPLUGINS=all SKIPUTILS=all SKIPMISC=all \
    NSIS_CONFIG_CONST_DATA_PATH=no PREFIX=/opt/nsis-3.11/Bin install-compiler \
  && cd / \
  && rm -rf /tmp/nsis.zip /tmp/nsis-src.tar.gz /tmp/nsis-311 \
  && makensis -VERSION

# Windows target and cargo-xwin (downloads the Windows SDK on first build).
RUN rustup target add x86_64-pc-windows-msvc \
  && CARGO_HOME=/usr/local/cargo cargo install --locked cargo-xwin@0.23.1 \
  && chmod -R a+rwX /usr/local/rustup /usr/local/cargo
