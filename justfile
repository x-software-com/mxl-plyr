#!/usr/bin/env -S just --justfile

# To change profile or release call just with:
build-type := "release"
profile := "development"
package := "mxl_plyr"
binary := "mxl_plyr"

builddir := "builddir"
pkgdir := "pkgdir"
pkgdir_path := builddir / pkgdir
appdir := "AppDir"
install_prefix := / "usr"
result_dir := "result"

vcpkg_version := "2024.10.21"
vcpkg_cache_dir := package + "-archive"

# To speed up build times use 'cargo binstall --no-confirm'.
# On AlmaLinux 8 binstall cannot be used, due to GLIBC issues.
cargo-install-cmd := "cargo install --locked"

alias c := config
alias b := build
alias i := install

#
# Setup the environment:
#

setup-cargo-binstall:
    cargo install --locked cargo-binstall

setup-cargo-hack: setup-cargo-binstall
    {{cargo-install-cmd}} cargo-hack

setup-cargo-audit: setup-cargo-binstall
    {{cargo-install-cmd}} cargo-audit

setup-mxl-env: install-vcpkg
    ./scripts/mxl-env.py --print-env --no-export-print-env > .mxl-env
    @echo "Created '.mxl-env' file"

setup-git:
    git config pull.rebase true
    git config branch.autoSetupRebase always

setup-cargo-tools: setup-cargo-binstall
    {{cargo-install-cmd}} typos-cli
    {{cargo-install-cmd}} cargo-bundle-licenses
    {{cargo-install-cmd}} cargo-version-util

setup-cocogitto: setup-cargo-binstall
    {{cargo-install-cmd}} cocogitto
    cog install-hook --overwrite commit-msg

setup: setup-git setup-cargo-hack setup-cargo-audit setup-cargo-tools setup-cocogitto self-update remove-mxl-env
    @echo "Done"

setup-vcpkg: setup-git setup-cargo-hack setup-cargo-audit setup-cargo-tools setup-cocogitto self-update setup-mxl-env
    @echo "Done"

setup-ci: setup-cargo-hack setup-cargo-audit setup-cargo-tools setup-mxl-env
    git config --global --add safe.directory $(pwd)

#
# Recipes for packaging:
#

config:
    #!/usr/bin/env sh
    set -e
    if [ -d {{builddir}} ]; then
        meson setup --reconfigure --buildtype {{build-type}} -Dpkgconfig.relocatable=true -Ddefault_library=static -Dprofile={{profile}} \
            -Dprefix={{install_prefix}} {{builddir}}
        echo "Successfully reconfigured the project!"
    else
        meson setup --buildtype {{build-type}} -Dpkgconfig.relocatable=true -Ddefault_library=static -Dprofile={{profile}} \
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
    ./scripts/build-linuxdeployimg.sh "{{package}}" "{{build-type}}" "{{binary}}" "{{builddir}}" "{{pkgdir}}" "{{result_dir}}"

appimage-from-linuxdeployimg:
    ./scripts/build-appimage-from-linuxdeployimg.sh "{{package}}" "{{build-type}}" "{{result_dir}}"

makeself-from-appimage:
    ./scripts/build-makeself-from-appimage.sh "{{package}}" "{{result_dir}}"

#
# Recipes for cargo:
#

hack: setup-cargo-hack
    cargo hack --feature-powerset --no-dev-deps check

clippy:
    cargo clippy --quiet --release --all-targets --all-features

audit: setup-cargo-audit
    cargo audit

cargo-fmt:
    cargo fmt --all

cargo-fmt-check:
    cargo fmt --check

#
# Misc recipes:
#

install-vcpkg:
    ./scripts/install-vcpkg.py --project-name={{package}} --vcpkg-version={{vcpkg_version}}

self-update: setup-cargo-binstall
    {{cargo-install-cmd}} just

mxl-env:
    ./scripts/mxl-env.py

remove-mxl-env:
    rm -f .mxl-env
    @echo "Removed '.mxl-env' file"

clean:
    cargo clean
    rm -rf vcpkg_installed vcpkg {{builddir}}

clean-cache: clean
    rm -rf ~/.cache/vcpkg/{{vcpkg_cache_dir}}

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
    set -e
    # Get parent directory as the mountpoint for the volume.
    MOUNT_DIR="$(dirname "$(pwd)")"
    docker run --privileged=true -it --rm \
        -v ${HOME}/.ssh:/root/.ssh \
        -v /var/cache/vcpkg:/root/.cache/vcpkg \
        -v ${MOUNT_DIR}:${MOUNT_DIR} \
        --workdir $(pwd) {{docker-tag}} bash

#
# Commands to test build in Docker image:
#

docker-build-appimage: setup-ci
    #!/usr/bin/env bash
    set -e
    mkdir -p {{result_dir}}
    if [ "{{build-type}}" != "release" ]; then
        eval "$(./scripts/mxl-env.py --vcpkg-debug --print-env)"
    else
        eval "$(./scripts/mxl-env.py --print-env)"
    fi
    (
        # set -o pipefail exits the script if a command piped with tee exits with an error
        set -o pipefail
        just --justfile {{justfile()}} profile={{build-type}} linuxdeployimg 2>&1 | tee {{result_dir}}/build-linuxdeployimg.log
        just --justfile {{justfile()}} appimage-from-linuxdeployimg 2>&1 | tee {{result_dir}}/build-appimage.log
    )

docker-build-makeself:
    #!/usr/bin/env bash
    set -e
    (
        # set -o pipefail exits the script if a command piped with tee exits with an error
        set -o pipefail
        just --justfile {{justfile()}} makeself-from-appimage 2>&1 | tee {{result_dir}}/build-makeself.log
    )
