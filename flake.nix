{
  description = "Dev shell with sqlx and podman-compose";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
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
          pkgs = import nixpkgs { inherit system; };
          isLinux = pkgs.stdenv.isLinux;
        in
        {
          default = pkgs.mkShell {
            buildInputs =
              with pkgs;
              [
                sqlx-cli
                cargo
                rustc
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
                      --file $(pwd)/docker-compose/docker-compose.yml up -d
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
