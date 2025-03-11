{
  description = "{{ description }}";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-{{ version }}";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, flake-utils }:
      flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in with pkgs; {
        packages = {
          default = pkgs.buildEnv {
            name = "{{ name }}";
            paths = with pkgs; [ {{ packages }} ];
          };
        };
      }
    );
}
