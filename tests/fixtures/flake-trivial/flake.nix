{
  inputs.nixpkgs.url = "github:meta-introspector/nixpkgs?ref=feature/CRQ-016-nixify";
  inputs.flake-utils.url = "github:meta-introspector/flake-utils?ref=feature/CRQ-016-nixify";

  outputs = { flake-utils, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.default = builtins.derivation {
          name = "trivial-1.0";
          system = system;
          outputs = [ "out" ];
          builder = "/bin/sh";
          args = [ "-c" "echo trivial > $out" ];
        };
      });
}
