[![CI checks](https://github.com/x-software-com/mxl-plyr/actions/workflows/check.yml/badge.svg)](https://github.com/x-software-com/mxl-plyr/actions/workflows/check.yml)
[![CI packaging](https://github.com/x-software-com/mxl-plyr/actions/workflows/package.yml/badge.svg)](https://github.com/x-software-com/mxl-plyr/actions/workflows/package.yml)
[![dependency status](https://deps.rs/repo/github/x-software-com/mxl-plyr/status.svg)](https://deps.rs/repo/github/x-software-com/mxl-plyr)
[![License](https://img.shields.io/github/license/x-software-com/mxl-plyr)](https://github.com/x-software-com/mxl-plyr/blob/main/LICENSE)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
![Downloads of latest release](https://badgen.net/github/assets-dl/x-software-com/mxl-plyr)
![Latest stable release](https://badgen.net/github/release/x-software-com/mxl-plyr/stable)

# MXL Plyr: Overview

MXL Plyr is an open source media player focused on the use with the [MXL Recorder](https://www.x-software.com/download/).

This tool is constantly evolving, and we always appreciate contributions!

Packages are available for [download](overview:releases).

If you've never used MXL Plyr before, or if you're trying to figure out how to use it, check out our [Getting Started](#getting-started) section.

For short description of available commands, once you've installed MXL Plyr, you can run `mxl-plyr --help`.

# Table of Contents

- [MXL Plyr: Overview](#mxl-plyr-overview)
- [Table of Contents](#table-of-contents)
- [Getting Started](#getting-started)
  - [Quick Start: Linux](#quick-start-linux)
  - [Installing Linux Developer Tools](#installing-linux-developer-tools)
- [Contributing](#contributing)
- [Privacy](#privacy)
- [License](#license)

# Getting Started

First, follow the quick start guide for [Linux](#quick-start-unix), depending on what you're using.

If you need a feature or encounter a problem, you can [open an issue on the GitHub repo][contributing:submit-issue] where the MXL team and community can see it.

## Quick Start: Linux

Prerequisites:

- [Git][getting-started:git]
- [Rust][getting-started:rust]
- [Python][getting-started:python]
- GCC **or** clang

### Install just and download the repository

```sh
cargo install just
git clone https://github.com/x-software-com/mxl-plyr
cd mxl-plyr
```

### Setup using the system libraries

```sh
just setup
```

### Setup using the vcpkg libraries

```sh
just setup-vcpkg
just mxl-env
```

### Development

To build and run the MXL Plyr:

```sh
cargo r --all-features
```

### Packaging

Setup and build the project to verify everything is setup properly:

```sh
just config
just build
```

To create an AppImage execute:

```sh
just linuxdeployimg
just appimage-from-linuxdeployimg
```

## Installing Linux Developer Tools

Across the different distributions of Linux, there are different packages you'll need to install:

### AlmaLinux, Rocky Linux, CentOS and other RedHat-based distributions

```sh
$ sudo sed -i -re "s|enabled=0|enabled=1|g" /etc/yum.repos.d/almalinux-powertools.repo
$ sudo dnf install epel-release
$ sudo dnf install xz bzip2 python39-pip gcc-toolset-12 make cmake ninja-build git \
    elfutils autoconf automake libtool zlib-devel curl zip unzip tar bison flex \
    pkgconfig nasm yasm clang rsync desktop-file-utils pulseaudio-libs-devel vulkan-loader-devel \
    wayland-devel wayland-protocols-devel mesa-libGL-devel libX11-devel libXft-devel \
    libXext-devel libXrandr-devel libXi-devel libXcursor-devel libXdamage-devel libXinerama-devel \
    libxkbcommon-devel libxkbcommon-x11-devel perl-IPC-Cmd
$ pip3 install meson
```

### Debian, Ubuntu, popOS, and other Debian-based distributions

```sh
$ sudo apt-get update
$ sudo apt-get install build-essential git tar curl zip unzip rsync clang nasm autoconf libtool \
    bison meson flex gettext patchelf libx11-dev libxft-dev libxext-dev  libx11-dev \
    xserver-xorg-dev xorg-dev libxkbcommon-dev libxkbcommon-x11-dev libx11-xcb-dev libva-dev \
    libgles2-mesa-dev  libglu1-mesa-dev freeglut3-dev mesa-common-dev libegl1-mesa-dev \
    libvulkan-dev libwayland-dev wayland-protocols libpulse-dev linux-tools-common \
    linux-tools-generic desktop-file-utils
```

### ArchLinux, Manjaro Linux, EndeavourOS, and other ArchLinux-based distributions

To install rust follow these steps: <https://wiki.archlinux.org/title/Rust>

#### Building with vcpkg

```sh
$ sudo pacman -Sy zip nasm python-pipx python-pip meson git curl unzip tar cmake ninja clang \
    python-distutils-extra libtool flex libva libxdamage rsync pkg-config autoconf automake make \
    bison desktop-file-utils libx11 libxext libxrender libpulse wayland wayland-protocols \
    wayland-utils egl-wayland libxkbcommon libxkbcommon-x11 libxrandr libxi libxcursor libxinerama
```

#### Building with system components

```sh
$ sudo pacman -Sy git clang pkg-config gtk4 libadwaita gst-libav gst-plugin-pipewire gst-plugin-va \
    gst-plugins-bad gstreamer gst-plugins-bad-libs gst-plugins-base gst-plugins-base-libs \
    gst-plugins-good gst-plugins-ugly gstreamer-vaapi
```

For any other distributions, make sure you're installing at least gcc and g++. If you want to add instructions for your specific distribution, [please open a PR][contributing:submit-pr]!

[getting-started:git]: https://git-scm.com/downloads
[getting-started:rust]: https://rustup.rs/
[getting-started:python]: https://www.python.org/downloads/

# Contributing

MXL Plyr is an open source project, and is thus built with your contributions. Here are some ways you can contribute:

- [Submit Issues][contributing:submit-issue]
- [Submit Fixes and New Features][contributing:submit-pr]

Please refer to our [Contributing Guide](CONTRIBUTING.md) for more details.

[contributing:submit-issue]: https://github.com/x-software-com/mxl-plyr/issues/new/choose
[contributing:submit-pr]: https://github.com/x-software-com/mxl-plyr/pulls

# Privacy

We believe that privacy is a human right, period.

MXL Plyr does respect your privacy, we collect no data and do not send any telemetry or usage data.

# License

The code in this repository is licensed under either of [Apache-2.0 License](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
