#!/usr/bin/env -S just --justfile

# To change profile or release call just with:
buildtype := "release"
profile := "development"
package := "mxl_plyr"
binary := "mxl_plyr"

builddir := "builddir"
pkgdir := "pkgdir"
pkgdir_path := builddir / pkgdir
appdir := "AppDir"
install_prefix := / "usr"
result_dir := "result"

vcpkg_version := "2025.08.27"
vcpkg_cache_dir := package + "-archive"

# Default Rust toolchain
rust-toolchain := "stable"

alias c := config
alias b := build
alias i := install

# default recipe to display help information
default:
    @just --list

#
# Setup the environment:
#

setup-mxl-env: install-vcpkg
    ./scripts/mxl-env.py --print-env --no-export-print-env > .mxl-env
    @echo "Created '.mxl-env' file"

setup-git:
    git config pull.rebase true
    git config branch.autoSetupRebase always

setup-cargo-tools:
    cargo install typos-cli
    cargo install cargo-bundle-licenses
    cargo install cargo-version-util
    cargo install cargo-hack
    cargo install cargo-audit

setup-cargo-dev-tools:
    cargo install cargo-edit
    cargo install cargo-upgrades
    cargo install cargo-machete

setup-cocogitto:
    cargo install cocogitto
    cog install-hook --overwrite commit-msg

setup: setup-git setup-cargo-dev-tools setup-cargo-tools setup-cocogitto self-update remove-mxl-env
    @echo "Done"

setup-vcpkg: setup-git setup-cargo-dev-tools setup-cargo-tools setup-cocogitto self-update setup-mxl-env
    @echo "Done"

setup-ci: setup-cargo-tools setup-mxl-env
    git config --global --add safe.directory $(pwd)

#
# Recipes for packaging:
#

config:
    #!/usr/bin/env sh
    set -e
    if [ -d {{builddir}} ]; then
        meson setup --reconfigure --buildtype {{buildtype}} -Dpkgconfig.relocatable=true -Ddefault_library=static -Dprofile={{profile}} \
            -Dprefix={{install_prefix}} {{builddir}}
        echo "Successfully reconfigured the project!"
    else
        meson setup --buildtype {{buildtype}} -Dpkgconfig.relocatable=true -Ddefault_library=static -Dprofile={{profile}} \
            -Dprefix={{install_prefix}} {{builddir}}
        echo "Successfully configured the project!"
    fi

build: config
    @ninja -C {{builddir}}
    @echo "Successfully built the project!"

install: build
    @rm -rf {{pkgdir_path}}
    DESTDIR={{pkgdir}} ninja -C {{builddir}} install
    @echo "Successfully installed the project!"

linuxdeployimg: install
    ./scripts/build-linuxdeployimg.sh "{{package}}" "{{buildtype}}" "{{binary}}" "{{builddir}}" "{{pkgdir}}" "{{result_dir}}"

appimage-from-linuxdeployimg:
    ./scripts/build-appimage-from-linuxdeployimg.sh "{{package}}" "{{buildtype}}" "{{result_dir}}"

makeself-from-appimage:
    ./scripts/build-makeself-from-appimage.sh "{{package}}" "{{result_dir}}"

#
# Recipes for cargo:
#

test rust-toolchain=rust-toolchain:
    cargo +{{rust-toolchain}} test --no-fail-fast --workspace --locked --all-features --all-targets

hack rust-toolchain=rust-toolchain: setup-cargo-tools
    cargo +{{rust-toolchain}} hack --feature-powerset --no-dev-deps check

clippy rust-toolchain=rust-toolchain:
    cargo +{{rust-toolchain}} clippy --quiet --release --all-targets --all-features

audit: setup-cargo-tools
    cargo audit

machete: setup-cargo-dev-tools
    cargo machete --with-metadata

upgrades: setup-cargo-dev-tools
    cargo upgrades

cargo-fmt:
    cargo fmt --all

cargo-fmt-check:
    cargo fmt --check

#
# Misc recipes:
#

install-vcpkg:
    ./scripts/install-vcpkg.py --project-name={{package}} --vcpkg-version={{vcpkg_version}}

self-update:
    cargo install just

mxl-env:
    ./scripts/mxl-env.py

remove-mxl-env:
    rm -f .mxl-env
    @echo "Removed '.mxl-env' file"

clean: _clean-vcpkg
    cargo clean
    rm -rf vcpkg_installed vcpkg {{builddir}}

clean-build: clean
    rm -rf .mxl-env build
    find . -name "config.rs" -delete
    find . -name "*_third_party_licenses.json" -delete
    find . | grep -E "(/__pycache__$|\.pyc$|\.pyo$)" | xargs rm -rf

clean-cargo-cache:
    rm -rf .cargo-cache

_clean-vcpkg:
    rm -rf vcpkg_installed vcpkg

clean-vcpkg-cache: _clean-vcpkg
    rm -rf ~/.cache/vcpkg/{{vcpkg_cache_dir}}

clean-cache: clean-build clean-cargo-cache clean-vcpkg-cache
    @echo "Cleaned all cache directories"

#
# Docker image for local testing:
#

docker-tag := "mxl-plyr-test"

docker-build-no-cache:
    docker build --no-cache -t {{docker-tag}} -f docker/Dockerfile docker

docker-build:
    docker build -t {{docker-tag}} -f docker/Dockerfile docker

docker-run: docker-build
    #!/usr/bin/env bash
    set -eo pipefail
    # Get parent directory as the mountpoint for the volume.
    MOUNT_DIR="$(dirname "$(pwd)")"
    docker run --privileged=true -it --rm \
        -v ${HOME}/.ssh:/root/.ssh \
        -v /var/cache/vcpkg:/root/.cache/vcpkg \
        -v ${MOUNT_DIR}:${MOUNT_DIR} \
        --workdir $(pwd) {{docker-tag}} bash

docker-run-clean:
    #!/usr/bin/env bash
    set -eo pipefail
    # Get parent directory as the mountpoint for the volume.
    MOUNT_DIR="$(dirname "$(pwd)")"
    docker run --privileged=true -it --rm \
        -v ${HOME}/.ssh:/root/.ssh \
        -v /var/cache/vcpkg:/root/.cache/vcpkg \
        -v ${MOUNT_DIR}:${MOUNT_DIR} \
        --workdir $(pwd) {{docker-tag}} bash -lc 'just clean-build'

#
# Commands to test build in Docker image:
#

docker-build-appimage: setup-ci
    #!/usr/bin/env bash
    set -eo pipefail
    mkdir -p {{result_dir}}
    if [ "{{buildtype}}" != "release" ]; then
        eval "$(./scripts/mxl-env.py --vcpkg-debug --print-env)"
    else
        eval "$(./scripts/mxl-env.py --print-env)"
    fi
    just --justfile {{justfile()}} buildtype={{buildtype}} profile={{profile}} linuxdeployimg 2>&1 | tee {{result_dir}}/build-linuxdeployimg.log
    just --justfile {{justfile()}} buildtype={{buildtype}} appimage-from-linuxdeployimg 2>&1 | tee {{result_dir}}/build-appimage.log

docker-build-makeself:
    #!/usr/bin/env bash
    set -eo pipefail
    just --justfile {{justfile()}} buildtype={{buildtype}} makeself-from-appimage 2>&1 | tee {{result_dir}}/build-makeself.log
