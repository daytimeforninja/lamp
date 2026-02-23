{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    pkg-config
    just
  ];

  buildInputs = with pkgs; [
    libxkbcommon
    wayland
    vulkan-loader
    libinput
    udev
    mesa
    expat
    fontconfig
    freetype
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
    libxkbcommon
    wayland
    vulkan-loader
  ]);
}
