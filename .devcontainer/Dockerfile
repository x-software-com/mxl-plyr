FROM mcr.microsoft.com/devcontainers/base:ubuntu24.04

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    ca-certificates lsb-release build-essential clang git tar curl wget zip unzip \
    linux-tools-common linux-tools-generic \
    xvfb libgtk-4-dev libadwaita-1-dev \
    libgstreamer-plugins-base1.0-dev libgstreamer-plugins-good1.0-dev libgstreamer-plugins-bad1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
    libgstreamer1.0-dev libunwind-dev
