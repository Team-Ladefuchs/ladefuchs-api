{
  description = "Ladefuchs API";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib dockerTools;

        craneLib = crane.lib.${system};
        cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);

        sqlFilter = path: _type: null != builtins.match ".*(sql|json)$" path;
        sqlOrCargo = path: type: (sqlFilter path type) || (craneLib.filterCargoSources path type);

        src = lib.cleanSourceWith {
          src = craneLib.path ../.; # The original, unfiltered source
          filter = sqlOrCargo;
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          nativeBuildInputs = [
            pkgs.pkg-config
          ];

        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        ladefuchs-api = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [
            pkgs.sqlx-cli
          ];

          preBuild = ''
            export SQLX_OFFLINE=true
          '';
        });

        buildImage = tag: dockerTools.buildImage {
          inherit tag;
          name = "ladefuchs-api";
          config = {
            ExposedPorts = { "3000" = { }; };
            Env = [
              "LISTEN=0.0.0.0"
              "DOMAIN=http://localhost:3000"
            ];
            Cmd = [ "${ladefuchs-api}/bin/ladefuchs-api" ];
          };
        };
        # multiple tags
        image-versioned = buildImage cargoToml.package.version;
        image-latest = buildImage "latest";
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit ladefuchs-api;
        };

        packages = {
          default = ladefuchs-api;
          inherit ladefuchs-api image-versioned image-latest;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};
          packages = [
            pkgs.sqlx-cli
          ];
        };
      });
}
