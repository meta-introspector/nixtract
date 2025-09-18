{
  description = "A flake for the submodule, providing a development shell for Gemini CLI.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rusty-hook-src.url = "path:../../../../vendor/hooks/rusty-hook"; # Reference parent's rusty-hook
  };

  outputs = { self, nixpkgs, flake-utils, rusty-hook-src, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rusty-hook-pkg = pkgs.rustPlatform.buildRustPackage {
          pname = "rusty-hook";
          version = "0.11.2"; # Match the version used in parent's .pre-commit-config.yaml
          src = rusty-hook-src;
          cargoLock = rusty-hook-src + "/Cargo.lock";
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bash
            git
            asciinema
            pre-commit # Add pre-commit to the devShell
            shellcheck # Add shellcheck to the devShell
            direnv # Add direnv to the devShell
            rusty-hook-pkg # Add rusty-hook to the devShell
          ];

          shellHook = ''
            echo "Welcome to the submodule Gemini CLI development shell!"
            pre-commit install # Install pre-commit hooks when entering the shell
          '';
        };
      }
    );
}
