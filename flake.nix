{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      mkCraneLib = system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default;
        in
        {
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          inherit pkgs rustToolchain;
        };
    in
    {
      packages = forAllSystems (system:
        let
          inherit (mkCraneLib system) craneLib pkgs;
          lib = pkgs.lib;

          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (craneLib.filterCargoSources path type)
              || (lib.hasInfix "/i18n/" path)
              || (lib.hasInfix "/res/" path)
              || (baseNameOf path == "i18n.toml");
          };

          commonArgs = {
            inherit src;
            strictDeps = true;

            nativeBuildInputs = with pkgs; [
              pkg-config
              makeWrapper
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
              openssl
              systemd
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          lamp = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            postInstall = ''
              install -Dm0644 res/dev.lamp.app.desktop $out/share/applications/dev.lamp.app.desktop
              install -Dm0644 res/dev.lamp.app.metainfo.xml $out/share/metainfo/dev.lamp.app.metainfo.xml
              install -Dm0644 res/icons/hicolor/scalable/apps/dev.lamp.app.svg $out/share/icons/hicolor/scalable/apps/dev.lamp.app.svg
              wrapProgram $out/bin/lamp \
                --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath [
                  pkgs.libxkbcommon
                  pkgs.wayland
                  pkgs.vulkan-loader
                ]}
            '';
          });
        in
        {
          default = lamp;
          inherit lamp;
        }
      );

      devShells = forAllSystems (system:
        let
          inherit (mkCraneLib system) pkgs rustToolchain;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustToolchain
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
              openssl
              systemd
            ];

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
              libxkbcommon
              wayland
              vulkan-loader
            ]);
          };
        }
      );
    };
}
