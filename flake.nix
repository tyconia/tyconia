{
  description = "Tyconia dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    blender-bin.url = "https://flakehub.com/f/edolstra/blender-bin/*";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      blender-bin,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Use the specific version package provided by blender-bin
        blenderPkg = blender-bin.packages.${system}.blender_5_0;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        nativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
          clang
          llvm
          lld
        ];

        buildInputs = with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          libxkbcommon
          wayland
          libx11
          libxcursor
          libxi
          libxrandr
          libxinerama
          libxext
        ];

        runtimeInputs = buildInputs ++ (with pkgs; [
          libGL
          mesa
        ]);
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;
          buildInputs = runtimeInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeInputs;
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" buildInputs;
        };

        devShells.blender = pkgs.mkShell {
          inputsFrom = [ self.devShells.${system}.default ];
          buildInputs = [ blenderPkg ];
        };
      }
    );
}
