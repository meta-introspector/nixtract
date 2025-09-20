{
  description = "A flake for this submodule, providing a basic development shell.";

  inputs = {
    nixpkgs.url = "github:meta-introspector/nixpkgs/feature/CRQ-016-nixify";
    flake-utils.url = "github:meta-introspector/flake-utils/feature/CRQ-016-nixify";
    naersk.url = "github:meta-introspector/naersk/feature/CRQ-016-nixify";
    # Add an input to the main project's nixpkgs
    mainNixpkgs.follows = "nixpkgs"; # This will make it follow the main project's nixpkgs
  };

  outputs = { self, nixpkgs, flake-utils, naersk, mainNixpkgs, ... }@inputs: # Add mainNixpkgs to inputs
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Use the main project's nixpkgs for consistency
        pkgs = import mainNixpkgs { inherit system; };
        # Override naersk's nixpkgs input to use the main project's nixpkgs
        naersk-lib = naersk.lib.${system}.override { nixpkgs = mainNixpkgs; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bash
            git
            shellcheck # Add shellcheck for shell script linting
            # Add any other common tools needed for your submodules here
          ];

          shellHook = ''
            echo "Welcome to the development shell of this submodule!"
          '';
        };

        packages.default = naersk-lib.buildPackage { # Define default package using naersk
          pname = "nixtract";
          version = "0.1.0"; # You might want to get this from Cargo.toml
          src = ./.;
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; [
            openssl
          ];
        };
      }
    );
}
