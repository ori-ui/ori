{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.pkg-config

    pkgs.gtk4
    pkgs.gtk4-layer-shell
    pkgs.libadwaita
  ];
}
