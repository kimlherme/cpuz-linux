{
  description = "CPU-Z-like benchmark for Linux";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      packageFor = system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        import ./default.nix { inherit pkgs; };
    in
    {
      packages = forAllSystems (system: {
        default = packageFor system;
        cpu-z-bench-linux = packageFor system;
      });

      apps = forAllSystems (system:
        let
          package = packageFor system;
        in
        {
          default = {
            type = "app";
            program = "${package}/bin/cpu-z-bench-linux";
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = [ pkgs.gcc pkgs.gnumake ];
          };
        });
    };
}
