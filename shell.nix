{
  pkgs ? (
    import <nixpkgs> {
      config.allowUnfree = true;
    }
  ),
  useVcpkg ? true,
  userShell ? "fish",
}:

let
  pkgConfigPath = "$PKG_CONFIG_PATH:/usr/lib/pkgconfig:/usr/share/pkgconfig";
  pkgConfigWrapper = pkgs.writeShellScriptBin "pkg-config" ''
    PKG_CONFIG_PATH=${pkgConfigPath} ${pkgs.pkg-config}/bin/pkg-config $@
  '';
in
(pkgs.buildFHSEnv {
  name = "mxl-plyr";
  targetPkgs =
    pkgs:
    (
      with pkgs;
      [
        pkgConfigWrapper
        pkgs.${userShell}
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
        iconv.dev
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
        fribidi
        fontconfig
        freetype
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
      ++ pkgs.lib.optionals (useVcpkg == false) [
        gtk4.dev
        glib.dev
        pango.dev
        harfbuzz.dev
        cairo.dev
        gdk-pixbuf.dev
        librsvg.dev
        libadwaita.dev
        gst_all_1.gstreamer
        gst_all_1.gstreamer.dev
        gst_all_1.gst-vaapi
        gst_all_1.gst-vaapi.dev
        gst_all_1.gst-libav
        gst_all_1.gst-libav.dev
        gst_all_1.gst-plugins-base
        gst_all_1.gst-plugins-base.dev
        gst_all_1.gst-plugins-good
        gst_all_1.gst-plugins-good.dev
        gst_all_1.gst-plugins-bad
        gst_all_1.gst-plugins-bad.dev
        gst_all_1.gst-plugins-ugly
        gst_all_1.gst-plugins-ugly.dev
        gst_all_1.gst-plugins-rs
        gst_all_1.gst-plugins-rs.dev
        gst_all_1.gst-editing-services
        gst_all_1.gst-editing-services.dev
        gst_all_1.gst-devtools
        ffmpeg_6-full.dev
        nvidia-vaapi-driver
        libepoxy
        glide-media-player
      ]
    );

  runScript =
    with pkgs;
    pkgs.writeScript "init.sh" ''
      # By default the GDK backend is set to Wayland on NixOS.
      # This fixes an issue with NVIDIA/GTK4/GStreamer (gtk4paintablesink) under Wayland, where the playback is very slow and choppy.
      # Check in the future, if this issue still exists, so we can remove this workaround.
      export GDK_BACKEND=x11

      # Set the Cargo home directory to avoid conflicts with other projects and different compiler and library versions.
      export CARGO_HOME="${builtins.toString ./.}/.cargo"

      export CUDA_PATH=${pkgs.cudatoolkit}

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
          fribidi
          fontconfig
          freetype
          libGL
          libdrm
          linuxPackages.nvidia_x11
          openssl
          libpulseaudio
          libjack2
        ]
      }:/usr/lib:usr/lib64"

      export SHELL="/usr/bin/${userShell}"
      exec ${userShell}
    '';
}).env
