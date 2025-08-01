{
  description = "Rust egui application flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        name = "";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        nativeDeps = with pkgs; [
          libxkbcommon
          dbus
          wayland
          libGL
          vulkan-loader
        ] ++ (with pkgs.xorg;[
            libX11
            libXcursor
            libXrandr
            libXi
          ]);

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config

          ] ++ nativeDeps;

          shellHook = ''
            export RUST_BACKTRACE=1
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath nativeDeps}:$LD_LIBRARY_PATH"
          '';
        };

        # 构建应用 (nix build)
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "${name}";
          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          buildInputs = nativeDeps;

          # egui 特定构建参数
          RUSTFLAGS = "--cfg=web_sys_unstable_apis";

          # 测试时需要的变量
          checkPhase = ''
            export XDG_RUNTIME_DIR=$(mktemp -d)
          '';
        };

        # 应用运行器 (nix run)
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/${name}";
        };
      });
}
