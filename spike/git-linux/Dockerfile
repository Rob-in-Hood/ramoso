# Static, relocatable Git for x86_64 Linux (musl). Runs on any distro.
# Build: DOCKER_BUILDKIT=0 docker build -t ramoso-git-spike spike/git-linux
# Copy out: id=$(docker create ramoso-git-spike /x) && docker cp "$id":/ spike/git-linux/out && docker rm "$id"

FROM alpine:3.22@sha256:49340b346798875b5592e5d420241f2f9cfba5f47e62ebb1446a1b34734e898f AS build

ARG CURL_VERSION=8.23.0
ARG CURL_SHA256=e3de77cfa6367083c776a26647fc9f633b538c06c3baa73d931a252ac8f72b49
ARG GIT_VERSION=2.56.1
ARG GIT_SHA256=f0b0bf89264b9bd2e1c640363e7529643989da76281600473c50745cd34a607f

# Build tools and the static libraries curl and Git link against.
RUN apk add --no-cache build-base linux-headers pkgconf perl xz curl cargo \
      openssl-dev openssl-libs-static zlib-dev zlib-static \
      nghttp2-dev nghttp2-static expat-dev expat-static

# Our own static curl: Alpine's links libidn2, whose error() clashes with Git's.
# No default CA path; Ramoso always passes GIT_SSL_CAINFO.
WORKDIR /src
RUN curl -fsSLO https://curl.se/download/curl-${CURL_VERSION}.tar.gz \
 && echo "${CURL_SHA256}  curl-${CURL_VERSION}.tar.gz" | sha256sum -c - \
 && tar -xzf curl-${CURL_VERSION}.tar.gz \
 && cd curl-${CURL_VERSION} \
 && ./configure --prefix=/opt/curl --disable-shared --enable-static \
      --with-openssl --with-nghttp2 --with-zlib \
      --without-libidn2 --without-libpsl --without-brotli --without-zstd --without-libssh2 \
      --without-nghttp3 --without-ngtcp2 \
      --disable-ldap --disable-ldaps --disable-rtsp --disable-dict --disable-telnet --disable-tftp \
      --disable-pop3 --disable-imap --disable-smtp --disable-gopher --disable-mqtt \
      --disable-manual --disable-docs \
      --without-ca-bundle --without-ca-path \
 && make -j"$(nproc)" && make install

# Git source, checked against kernel.org's checksum.
RUN curl -fsSLO https://mirrors.edge.kernel.org/pub/software/scm/git/git-${GIT_VERSION}.tar.xz \
 && echo "${GIT_SHA256}  git-${GIT_VERSION}.tar.xz" | sha256sum -c - \
 && tar -xJf git-${GIT_VERSION}.tar.xz

# Flags: relocatable (RUNTIME_PREFIX), musl-safe regex, no Perl/Python/Tk/translations,
# no hard-linked copies, and our curl from /opt/curl.
WORKDIR /src/git-${GIT_VERSION}
RUN make -j"$(nproc)" prefix=/opt/git \
      RUNTIME_PREFIX=YesPlease PROCFS_EXECUTABLE_PATH=/proc/self/exe \
      NO_REGEX=YesPlease NO_GETTEXT=YesPlease NO_PERL=YesPlease NO_PYTHON=YesPlease NO_TCLTK=YesPlease \
      SKIP_DASHED_BUILT_INS=YesPlease INSTALL_SYMLINKS=YesPlease \
      CURLDIR=/opt/curl CURL_CONFIG=/opt/curl/bin/curl-config \
      CURL_LDFLAGS="$(PKG_CONFIG_PATH=/opt/curl/lib/pkgconfig pkg-config --static --libs libcurl)" \
      CFLAGS="-O2" LDFLAGS="-static" \
      install \
 && (find /opt/git -type f -perm -u+x -exec strip --strip-unneeded {} + 2>/dev/null || true) \
 && rm -rf /opt/git/share/man /opt/git/share/perl5 /opt/git/share/gitweb /opt/git/share/git-gui /opt/git/share/gitk

# Fail if anything is dynamically linked.
RUN for f in /opt/git/bin/git /opt/git/libexec/git-core/git-remote-http; do \
      if readelf -d "$f" | grep -q NEEDED; then echo "dynamic: $f"; exit 1; fi; done \
 && /opt/git/bin/git --version

# Only the finished Git goes into the output.
FROM scratch
COPY --from=build /opt/git /
