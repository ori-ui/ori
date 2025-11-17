{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    pkgs.gtk4
    pkgs.gtk4-layer-shell
    pkgs.pkg-config
    pkgs.libadwaita

    pkgs.wayland
    pkgs.libGL
    pkgs.libxkbcommon
    pkgs.freetype
    pkgs.fontconfig
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
