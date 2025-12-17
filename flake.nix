{
  description = "Game Experiment";
  inputs = {
    naersk.url = "github:nmattia/naersk/master";
    # This must be the stable nixpkgs if you're running the app on a
    # stable NixOS install.  Mixing EGL library versions doesn't work.
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = github:edolstra/flake-compat;
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, naersk, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        libPath = with pkgs; lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          doCheck = true;
          pname = "game-experiment";
          nativeBuildInputs = [ pkgs.makeWrapper ];
          buildInputs = with pkgs; [
            xorg.libxcb
          ];
          postInstall = ''
            wrapProgram "$out/bin/game-experiment" --prefix LD_LIBRARY_PATH : "${libPath}"
          '';
        };

        defaultApp = utils.lib.mkApp {
          drv = self.defaultPackage."${system}";
        };

        devShell = with pkgs; mkShell {
          packages = [
            pkg-config
            rustc
            rustPlatform.rustLibSrc
          ];
          nativeBuildInputs = [
            cargo
            cargo-insta
            pre-commit
            rust-analyzer
            rustPackages.clippy
            rustc
            rustfmt
            tokei
            xorg.libxcb
          ];
          shellHook = ''
            [ -d ".devenv/profile" ] && exit 0
            mkdir -p .devenv/profile/{bin,lib/rustlib/src}
            ln -sfn ${rustc}/bin/* .devenv/profile/bin/
            ln -sfn ${rustPlatform.rustLibSrc} .devenv/profile/lib/rustlib/src/rust
            echo ""
            echo "Development environment initialized with local toolchain paths"
            echo "Toolchain: $PWD/.devenv/profile/bin"
            echo "Standard library: $PWD/.devenv/profile/lib/rustlib/src/rust"
            echo ""
          '';
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = libPath;
          GIT_EXTERNAL_DIFF = "${difftastic}/bin/difft";
        };
      });


  nixConfig = {
    extra-experimental-features = "flakes nix-command";
    extra-substituters = [
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
}
