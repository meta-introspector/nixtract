{ pkgs ? import <nixpkgs> {} }:

pkgs.callPackage (
  { lib, rustPlatform, cargo, rustc, stdenv, fetchFromGitHub, openssl }:

  rustPlatform.buildRustPackage rec {
    pname = "nixtract";
    version = "0.3.0"; # Match the version we vendored

    src = ./.; # Point to the current directory (the submodule)

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    buildInputs = [ openssl ];

    doCheck = false; # Disable running tests during build

    meta = with lib; {
      description = "Extract the graph of derivations from a Nix flake";
      homepage = "https://github.com/tweag/nixtract";
      license = licenses.mit;
      maintainers = with maintainers; [ "tweag" ];
    };
  }
) {}