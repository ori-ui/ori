{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    pkgs.pkg-config

    pkgs.gtk4
    pkgs.gtk4-layer-shell
    pkgs.libadwaita

    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.freetype
    pkgs.fontconfig
    pkgs.vulkan-loader
    pkgs.vulkan-validation-layers
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
}
