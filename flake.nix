{
  description = "Dev shell with sqlx and podman-compose";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;

    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          isLinux = pkgs.stdenv.isLinux;
          rustStable = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rustfmt"
              "clippy"
              "rust-analyzer"
              "rust-src"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs =
              with pkgs;
              [
                sqlx-cli
                rustStable
              ]
              ++ pkgs.lib.optionals isLinux [
                podman-compose
                postgresql
              ];

            shellHook = ''
              echo "api dev ready"

              ${pkgs.lib.optionalString isLinux ''
                if command -v podman > /dev/null; then
                  if ! podman ps | grep -q postgres; then
                    podman-compose \
                      --file $(pwd)/docker-compose/docker-compose.yml up -d > /dev/null 2>&1
                    echo "Started postgres"
                  fi
                fi
              ''}
            '';
          };
        }
      );
    };
}
