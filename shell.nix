{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation rec {
  name = "ori";
  
  buildInputs = [
    pkgs.libGL

    pkgs.libxkbcommon
    pkgs.xorg.libxcb
    pkgs.wayland
    pkgs.fontconfig
    pkgs.freetype
  ];

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
}
