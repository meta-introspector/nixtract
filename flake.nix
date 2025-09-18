{
  description = "A flake for the submodule, providing a development shell for Gemini CLI.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
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
          ];

          shellHook = ''
            echo "Welcome to the submodule Gemini CLI development shell!"
            pre-commit install # Install pre-commit hooks when entering the shell
          '';
        };
      }
    );
}
