{
  description = "Minebak flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        name = "minebak";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
        nativeDeps = with pkgs; [
          libgcc.libgcc
          libxkbcommon
          dbus
          wayland
          libGL
          vulkan-loader
          openssl
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

          RUST_BACKTRACE = 1;

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath nativeDeps}:$LD_LIBRARY_PATH"
          '';
        };

        # 构建应用 (nix build)
        packages.default = (pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        }).buildRustPackage rec {
          pname = "${name}";
          version = "0.2.0";

          src = ./.;

          cargoLock.lockFile = src + /Cargo.lock;
          
          cargoSha256 = nixpkgs.lib.fakeSha256;
          nativeBuildInputs = with pkgs;[
            pkg-config

            makeWrapper
          ];
          buildInputs = nativeDeps;
          RUSTFLAGS = "--cfg=web_sys_unstable_apis";

          postInstall = ''
            wrapProgram "$out/bin/${name}" --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath nativeDeps}"
          '';
        };

        # 应用运行器 (nix run)
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/${name}";
        };
      });
}
