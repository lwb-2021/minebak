{
  description = "Minebak flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    let
      name = "minebak";
      version = "1.0.0-alpha";
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustNightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" ];
        });

        npmDeps = pkgs.fetchNpmDeps {
          name = "${name}-${version}-npm-deps";
          src = ./.;
          hash = nixpkgs.lib.fakeSha256;
        };

      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
          inherit name version;

          src = ./.;

          cargoHash = nixpkgs.lib.fakeSha256;

          nativeBuildInputs = with pkgs;[
            cargo-tauri.hook
            nodejs
            npmHooks.npmConfigHook
            pkg-config
            gobject-introspection
          ] ++ (with pkgs; lib.optionals stdenv.hostPlatform.isLinux [ wrapGAppsHook4 ]);

          buildInputs = (with pkgs; [
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            openssl
          ]) ++ (with pkgs; lib.optionals stdenv.hostPlatform.isLinux [
            glib-networking
          ]);

          cargoRoot = "src-tauri";
          buildAndTestSubdir = finalAttrs.cargoRoot;

          inherit npmDeps;
        });

        devShells.default = pkgs.mkShell {
          inherit name;

          nativeBuildInputs = with pkgs; [
            pkg-config
            gobject-introspection
            rustNightly
            cargo-tauri
            nodejs
          ];

          buildInputs = with pkgs; [
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            openssl
          ];

          shellHook = ''
            export RUST_BACKTRACE=1
          '';
        };
      }
    );
}
