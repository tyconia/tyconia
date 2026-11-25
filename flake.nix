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
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true; # needed for aseprite
        };

        # Use the specific version package provided by blender-bin
        blenderPkg = blender-bin.packages.${system}.blender_5_0;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Newer nightlies break bevy 0.15.3 type inference before codegen even starts.
        craneliftToolchain = pkgs.rust-bin.nightly."2025-03-01".default.override {
          extensions = [ "rust-src" "rustc-codegen-cranelift-preview" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # nixpkgs' trunk derivation doesn't provide libzstd, which zstd-sys probes via pkg-config
        trunk = pkgs.trunk.overrideAttrs (old: {
          buildInputs = old.buildInputs ++ [ pkgs.zstd ];
        });

        commonNativeBuildInputs = with pkgs; [
          pkg-config
          trunk
          clang
          llvm
          lld
        ];

        nativeBuildInputs = commonNativeBuildInputs ++ [ rustToolchain ];
        craneliftNativeBuildInputs = commonNativeBuildInputs ++ [ craneliftToolchain ];

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
        packages.aseprite = pkgs.aseprite;

        apps.aseprite = {
          type = "app";
          program = "${pkgs.aseprite}/bin/aseprite";
        };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;
          buildInputs = runtimeInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeInputs;
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" buildInputs;
        };

        # Fast debug builds: ~58s vs ~2m45s for a clean LLVM build with the new profiles.
        # Release/trunk builds should still use the default shell (LLVM).
        devShells.cranelift = pkgs.mkShell {
          nativeBuildInputs = craneliftNativeBuildInputs;
          buildInputs = runtimeInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeInputs;
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" buildInputs;
          shellHook = ''
            export RUSTFLAGS="-Zcodegen-backend=cranelift $RUSTFLAGS"
          '';
        };

        devShells.blender = pkgs.mkShell {
          inputsFrom = [ self.devShells.${system}.default ];
          buildInputs = [ blenderPkg ];
        };
      }
    );
}
