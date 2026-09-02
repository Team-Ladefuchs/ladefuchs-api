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

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        inherit (pkgs) lib;

        sqlFilter = path: _type: null != builtins.match ".*(sql|json)$" path;
        sqlOrCargo = path: type: (sqlFilter path type) || (craneLib.filterCargoSources path type);

        src = lib.cleanSourceWith {
          src = craneLib.path ../.; # The original, unfiltered source
          filter = sqlOrCargo;
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          doCheck = false;
          nativeBuildInputs = [
            pkgs.pkg-config
          ];

        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        ladefuchs-api = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [
              pkgs.sqlx-cli
            ];

            preBuild = ''
              export SQLX_OFFLINE=true
            '';
            postFixup = ''
              mkdir -p $out/share/ladefuchs-api
              cp -r ${../docs} $out/share/ladefuchs-api/docs
            '';
          }
        );

      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit ladefuchs-api;
        };

        packages = {
          default = ladefuchs-api;
          inherit ladefuchs-api;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};
          packages = [
            pkgs.sqlx-cli
          ];
        };
      }
    ) // {
      nixosModules.default = import ./module.nix { inherit self; };

      nixosConfigurations.test = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          self.nixosModules.default
          {
            services.ladefuchs-api.enable = true;
            boot.loader.grub.devices = [ "nodev" ];
            fileSystems."/" = {
              device = "nodev";
              fsType = "tmpfs";
            };
          }
        ];
      };
    };
}
