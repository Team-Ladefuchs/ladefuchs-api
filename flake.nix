{
  description = "Dev shell with sqlx and podman-compose";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          sqlx-cli
          podman-compose
          cargo
          rustc
        ];
        shellHook = ''
          if ! podman ps | grep -q postgres; then
            podman-compose --file $(pwd)/docker-compose/docker-compose.yml up -d
            echo "Started postgres"
          fi
          echo "api dev ready"
        '';
      };
    };
}
