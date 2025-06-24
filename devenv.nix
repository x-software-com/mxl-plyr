{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  pkgConfigPath = "$PKG_CONFIG_PATH:$PKG_CONFIG_PATH_x86_64_unknown_linux_gnu:${pkgs.xorg.libX11.dev}/lib/pkgconfig:${pkgs.xorg.xorgproto.out}/share/pkgconfig:${pkgs.xorg.libXext.dev}/lib/pkgconfig:${pkgs.pulseaudio.dev}/lib/pkgconfig:${pkgs.xorg.libXrender.dev}/lib/pkgconfig:${pkgs.kdePackages.wayland.dev}/lib/pkgconfig:${pkgs.kdePackages.wayland-protocols.out}/share/pkgconfig:${pkgs.libdrm.dev}/lib/pkgconfig:${pkgs.libxkbcommon.dev}/lib/pkgconfig:${pkgs.xorg.libXrandr.dev}/lib/pkgconfig:${pkgs.xorg.libXi.dev}/lib/pkgconfig:${pkgs.xorg.libXcursor.dev}/lib/pkgconfig:${pkgs.xorg.libXdamage.dev}/lib/pkgconfig:${pkgs.xorg.libXfixes.dev}/lib/pkgconfig:${pkgs.xorg.libXinerama.dev}/lib/pkgconfig:${pkgs.libva.dev}/lib/pkgconfig:${pkgs.xorg.libxcb.dev}/lib/pkgconfig";
  pkgConfigWrapper = pkgs.writeShellScriptBin "pkg-config" ''
    PKG_CONFIG_PATH_x86_64_unknown_linux_gnu=${pkgConfigPath} ${pkgs.pkg-config}/bin/pkg-config $@
  '';
in
{
  options = {
    use-vcpkg = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "When enabled system GTK/GStreamer/FFmpeg will not be available";
    };
  };

  config = {
    # https://devenv.sh/basics/
    env.GREET = "mxl-player devenv";

    # https://devenv.sh/packages/
    packages =
      with pkgs;
      [
        pkgConfigWrapper
        fish
        vim
        wget
        curl
        htop
        eza
        ripgrep
        bat
        dust
        fd
        ouch
        zip
        perl
        gitFull
        delta
        stdenv
        gcc
        libgcc.lib
        nasm
        yasm
        flex
        gnumake
        bison
        rsync
        valgrind
        libGL
        linuxPackages.nvidia_x11
        cudatoolkit
        cudaPackages.cuda_nvrtc
        autoAddDriverRunpath
        python3Full
        pipx
        hugo
        just
        zlib
        expat
        libgbm
        autoconf
        automake
        libtool
        cmake
        meson
        gdb
        gettext
        gperf
        libtool
        glib
        libxkbcommon.dev
        m4
        ninja
        pkg-config
        graphene.dev
        zip
        zstd.dev
        vulkan-loader.dev
        gtk4
        gsettings-desktop-schemas
        freetds

        libva.dev
        libdrm.dev
        libglvnd.dev
        mesa
        mesa_glu.dev
        pulseaudio.dev
        wayland.dev
        kdePackages.wayland.dev
        kdePackages.wayland-protocols
        wayland-scanner
        xorg.libxcb.dev
        xorg.libxcb
        xorg.xcbutilimage.dev
        xorg.xcbutilwm.dev
        xorg.xorgserver.dev
        xorg.libpthreadstubs
        xorg.libX11.dev
        xorg.libXext.dev
        xorg.libXi.dev
        xorg.xcbproto
        xorg.xcbutil.dev
        xorg.xcbutilcursor.dev
        xorg.xcbutilerrors
        xorg.xcbutilkeysyms.dev
        xorg.xcbutilrenderutil.dev
        xorg.xcbutilwm.dev
        xorg.libXrender.dev
        xorg.libXfixes.dev
        xorg.libXrandr.dev
        xorg.libXcursor.dev
        xorg.libXdamage.dev
        xorg.libXinerama.dev
        xorg.xorgproto
      ]
      ++ lib.optionals (config.use-vcpkg == false) [
        gtk4.dev
        gdk-pixbuf
        librsvg
        libadwaita.dev
        gst_all_1.gstreamer.dev
        gst_all_1.gst-vaapi.dev
        gst_all_1.gst-libav.dev
        gst_all_1.gst-plugins-base.dev
        gst_all_1.gst-plugins-good.dev
        gst_all_1.gst-plugins-bad.dev
        gst_all_1.gst-plugins-ugly.dev
        gst_all_1.gst-plugins-rs.dev
        gst_all_1.gst-editing-services.dev
        gst_all_1.gst-devtools
        ffmpeg_6-full.dev
        nvidia-vaapi-driver
        libepoxy
        glide-media-player
      ];

    # https://devenv.sh/languages/
    # languages.rust.enable = true;
    # languages.c.enable = true;
    # languages.cplusplus.enable = true;
    # languages.shell.enable = true;

    # https://devenv.sh/processes/
    # processes.cargo-watch.exec = "cargo-watch";

    # https://devenv.sh/services/
    # services.postgres.enable = true;

    # https://devenv.sh/scripts/
    scripts.hello.exec = ''
      echo hello from $GREET
    '';

    enterShell = with pkgs; ''
      # By default the GDK backend is set to Wayland on NixOS.
      # This fixes an issue with NVIDIA/GTK4/GStreamer (gtk4paintablesink) under Wayland, where the playback is very slow and choppy.
      # Check in the future, if this issue still exists, so we can remove this workaround.
      export GDK_BACKEND=x11

      export CUDA_PATH=${pkgs.cudatoolkit}
      export EXTRA_LDFLAGS="-L/lib -L${pkgs.linuxPackages.nvidia_x11}/lib"
      export EXTRA_CCFLAGS="-I/usr/include"
      export PKG_CONFIG_PATH="${pkgConfigPath}"
      export PKG_CONFIG_EXECUTABLE="$(which pkg-config)"
      export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
        pkgs.lib.makeLibraryPath [
          libgcc.lib
          wayland
          libva
          cudaPackages.cuda_nvrtc
          xorg.libXinerama
          xorg.libXrandr
          xorg.libXfixes
          xorg.libXcursor
          libxkbcommon
          xorg.libXi
          xorg.libXdamage
          xorg.libXext
          xorg.libxcb
          xorg.libX11
          xorg.libXrender
          expat
          libgbm
          libGL
          libdrm
          linuxPackages.nvidia_x11
          openssl
          libpulseaudio
          libjack2
        ]
      }:/usr/lib:usr/lib64"
      exec fish
    '';

    # https://devenv.sh/tasks/
    # tasks = {
    #   "myproj:setup".exec = "mytool build";
    #   "devenv:enterShell".after = [ "myproj:setup" ];
    # };

    # https://devenv.sh/tests/
    enterTest = ''
      echo "Running tests"
      git --version | grep --color=auto "${pkgs.git.version}"
    '';

    # https://devenv.sh/pre-commit-hooks/
    # pre-commit.hooks.shellcheck.enable = true;

    # See full reference at https://devenv.sh/reference/options/
  };
}
