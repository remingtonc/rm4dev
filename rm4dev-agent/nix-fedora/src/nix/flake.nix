{
  description = "rm4dev developer tools";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, nixpkgs-unstable, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (final: prev: {
              unstable = import nixpkgs-unstable {
                inherit system;
                config.allowUnfree = prev.config.allowUnfree or false;
              };
            })
          ];
        };

        toolPackages = import ./packages.nix { inherit pkgs; };

        profile = pkgs.buildEnv {
          name = "rm4dev-devtools";
          paths = toolPackages;
          ignoreCollisions = true;
        };
      in
      {
        packages = {
          devTools = profile;
          default = profile;
        };

        devShells.default = pkgs.mkShell {
          packages = toolPackages;
        };
      });
}