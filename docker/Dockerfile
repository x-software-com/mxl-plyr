FROM almalinux:8

LABEL org.opencontainers.image.source=https://github.com/x-software-com/mxl-docker
LABEL org.opencontainers.image.description="Build image for MXL Plyr"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"

RUN sed -i -re "s|enabled=0|enabled=1|g" /etc/yum.repos.d/almalinux-powertools.repo

# Add epel repository
RUN dnf -y install epel-release \
  && dnf clean all \
  && rm -rf /var/cache/dnf

# Required base packages
RUN dnf -y install xz bzip2 less python3.12-pip \
  && dnf clean all \
  && rm -rf /var/cache/dnf

# Install zstd for actions/cache see: https://github.com/actions/cache/issues/1455#issuecomment-2328358604
RUN dnf -y install zstd \
  && dnf clean all \
  && rm -rf /var/cache/dnf

RUN pip3 install meson

ARG DEVTOOLSET=gcc-toolset-12
# Required for building vcpkg packages
RUN dnf -y install ${DEVTOOLSET} ${DEVTOOLSET}-libatomic-devel libtool make cmake ninja-build git elfutils autoconf automake zlib-devel curl wget zip unzip tar bison flex \
  && dnf clean all \
  && rm -rf /var/cache/dnf \
  && echo -e "source /opt/rh/${DEVTOOLSET}/enable" >> /etc/profile.d/compiler.sh

RUN dnf -y install diffutils pkgconfig nasm yasm rsync desktop-file-utils \
  && dnf clean all \
  && rm -rf /var/cache/dnf

RUN dnf -y install \
  pulseaudio-libs-devel vulkan-loader-devel \
  libva-devel \
  mesa-libGL-devel libX11-devel libXft-devel libXext-devel libXrandr-devel libXi-devel libXcursor-devel libXdamage-devel libXinerama-devel libxkbcommon-devel libxkbcommon-x11-devel \
  perl-IPC-Cmd gtk-update-icon-cache \
  && dnf clean all \
  && rm -rf /var/cache/dnf

# Requirements for 'cargo test' implementing integration tests using X11
RUN dnf -y install \
  xorg-x11-server-Xvfb mesa-libEGL mesa-dri-drivers \
  && dnf clean all \
  && rm -rf /var/cache/dnf

# Reinstall some packages to get access to deleted license texts:
RUN dnf -y reinstall libblkid lz4-libs \
  && dnf clean all \
  && rm -rf /var/cache/dnf

COPY pkg-config /usr/local/bin

RUN echo -e "export PKG_CONFIG_PATH=/usr/local/lib64/pkgconfig:/usr/local/lib/pkgconfig:/usr/local/share/pkgconfig\${PKG_CONFIG_PATH:+:\${PKG_CONFIG_PATH}}" >> /etc/profile.d/pkg-config-path.sh
RUN echo -e "export PKG_CONFIG=/usr/local/bin/pkg-config" >> /etc/profile.d/pkg-config-path.sh
RUN echo -e "export PATH=/usr/local/bin\${PATH:+:\${PATH}}" >> /etc/profile.d/usr-local-path.sh

# Install latest wayland version for GTK/GStreamer
RUN dnf -y install \
  libxml2-devel \
  && dnf clean all \
  && rm -rf /var/cache/dnf

ARG PACKAGE=wayland
ARG VERSION=1.23.1
RUN pushd /tmp \
  && curl -LO https://gitlab.freedesktop.org/wayland/${PACKAGE}/-/releases/${VERSION}/downloads/${PACKAGE}-${VERSION}.tar.xz \
  && tar -xf ${PACKAGE}-${VERSION}.tar.xz \
  && cd ${PACKAGE}-${VERSION} \
  && source /etc/profile.d/compiler.sh \
  && meson -Ddocumentation=false build/ \
  && ninja -C build/ install \
  && popd \
  && rm -rf /tmp/${PACKAGE}*

ARG PACKAGE=wayland-protocols
ARG VERSION=1.38
RUN pushd /tmp \
  && curl -LO https://gitlab.freedesktop.org/wayland/${PACKAGE}/-/releases/${VERSION}/downloads/${PACKAGE}-${VERSION}.tar.xz \
  && tar -xf ${PACKAGE}-${VERSION}.tar.xz \
  && cd ${PACKAGE}-${VERSION} \
  && source /etc/profile.d/compiler.sh \
  && meson build/ \
  && ninja -C build/ install \
  && popd \
  && rm -rf /tmp/${PACKAGE}*

# Install rust and some cargo tools
ARG RUST_VERSION=1.83.0
RUN export RUSTUP_HOME=/opt/rust \
  && export CARGO_HOME=/opt/rust \
  && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain ${RUST_VERSION} \
  && unset RUSTUP_HOME CARGO_HOME \
  && printf '#!/bin/sh\nsource /opt/rust/env\ncase ":${PATH}:" in\n    *:"$HOME/.cargo/bin":*)\n        ;;\n    *)\n        export PATH="$HOME/.cargo/bin:$PATH"\n        ;;\nesac\n' > /etc/profile.d/rust.sh \
  && source /etc/profile.d/rust.sh \
  && rustup default stable \
  && cargo install cocogitto \
  && cargo install just \
  && cargo install cargo-bundle-licenses \
  && cargo install cargo-audit \
  && cargo install typos-cli \
  && cargo install cargo-version-util \
  && cargo install cargo-hack

# Requirements for 'cargo test' implementing integration tests using X11
RUN yum -y install \
  xorg-x11-server-Xvfb mesa-libEGL mesa-dri-drivers \
  && yum clean all \
  && rm -rf /var/cache/yum

# Appimage installation:
COPY appimage-setup/setup-rhel.sh /tmp
COPY appimage-setup/linuxdeploy.sh /tmp
COPY appimage-setup/linuxdeploy-plugin-gtk.patch /tmp
RUN pushd /tmp \
  && ./setup-rhel.sh \
  && rm setup-rhel.sh

# Makeself installation:
RUN dnf -y install unzip chrpath \
  && dnf clean all \
  && rm -rf /var/cache/dnf

ARG VERSION=2.5.0
RUN cd /tmp \
  && curl -LO https://github.com/megastep/makeself/releases/download/release-${VERSION}/makeself-${VERSION}.run \
  && chmod +x makeself-${VERSION}.run \
  && mkdir -p /opt \
  && cd /opt \
  && /tmp/makeself-${VERSION}.run \
  && mv makeself-${VERSION} makeself \
  && rm /tmp/makeself-${VERSION}.run
