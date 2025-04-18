{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixpkgs-unstable;
    rust-overlay.url = "github:oxalica/rust-overlay";
    nur.url = github:polygon/nur.nix;
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, rust-overlay, nixpkgs, nur, naersk }:
    let
      systems = [
        "aarch64-linux"
        "i686-linux"
        "x86_64-linux"
      ];
      overlays = [ (import rust-overlay) ];
      program_name = "bevy_hilbert";
    in
    builtins.foldl'
      (outputs: system:

        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit overlays system;
          };

          rust-bin = pkgs.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default.override {
              targets = [ "wasm32-unknown-unknown" ];
              extensions = [ "rust-src" ];
            });
          naersk-lib = naersk.lib.${system}.override {
            cargo = rust-bin;
            rustc = rust-bin;
          };

          rust-dev-deps = with pkgs; [
            rust-analyzer
            rustfmt
            lldb
            cargo-geiger
            nur.packages.${system}.wasm-server-runner
            renderdoc
          ];
          build-deps = with pkgs; [
            pkg-config
            mold
            lld
            clang
            makeWrapper
          ];
          runtime-deps = with pkgs; [
            alsa-lib
            udev
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libxcb
            libGL
            libxkbcommon
            vulkan-loader
            vulkan-headers
          ];
        in
        {
          devShell.${system} =
            let
              all_deps = runtime-deps ++ build-deps ++ rust-dev-deps ++ [ rust-bin ];
            in
            pkgs.mkShell {
              buildInputs = all_deps;
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (all_deps);
              PROGRAM_NAME = program_name;
              shellHook = ''
                export CARGO_MANIFEST_DIR=$(pwd)
              '';
            };
          packages.${system} =
            {
              app = naersk-lib.buildPackage
                {
                  pname = program_name;
                  root = ./.;
                  buildInputs = runtime-deps;
                  nativeBuildInputs = build-deps;
                  overrideMain = attrs: {
                    fixupPhase = ''
                      wrapProgram $out/bin/${program_name} \
                        --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtime-deps}
                    '';
                    patchPhase = ''
                      sed -i s/\"dynamic\"// Cargo.toml
                    '';
                  };
                };
              wasm = self.packages.${system}.app.overrideAttrs
                (final: prev: {
                  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
                  fixupPhase = '''';
                });
            };
          defaultPackage.${system} = self.packages.${system}.app;
          apps.${system}.wasm = {
            type = "app";
            program = ''${pkgs.writeShellScript "wasm-run" "${nur.packages.${system}.wasm-server-runner}/bin/wasm-server-runner ${self.packages.${system}.wasm}/bin/${program_name}.wasm"}'';
          };
        }
      )
      { }
      systems;

}
