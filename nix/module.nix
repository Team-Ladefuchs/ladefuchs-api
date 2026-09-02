{
  self,
}: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.ladefuchs-api;
  settingsFormat = pkgs.formats.keyValue {};
in {
  options.services.ladefuchs-api = {
    enable = lib.mkEnableOption "the Ladefuchs API server";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.ladefuchs-api;
      defaultText = lib.literalExpression "ladefuchs-api.packages.\${pkgs.stdenv.hostPlatform.system}.ladefuchs-api";
      description = "The ladefuchs-api package to run.";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "ladefuchs-api";
      description = ''
        User account the service runs as. By default a systemd dynamic user
        named ladefuchs-api is used (see
        {option}`services.ladefuchs-api.dynamicUser`). To run as an existing
        user, e.g. one whose home directory holds the images, set this and
        turn {option}`services.ladefuchs-api.dynamicUser` off.
      '';
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = cfg.user;
      defaultText = lib.literalExpression "config.services.ladefuchs-api.user";
      description = "Group account the service runs as.";
    };

    dynamicUser = lib.mkOption {
      type = lib.types.bool;
      default = cfg.user == "ladefuchs-api";
      defaultText = lib.literalExpression "config.services.ladefuchs-api.user == \"ladefuchs-api\"";
      description = "Whether to run under a systemd dynamic user instead of a persistent account.";
    };

    settings = lib.mkOption {
      type = lib.types.submodule {
        freeformType = settingsFormat.type;
        options = {
          PORT = lib.mkOption {
            type = lib.types.port;
            default = 3000;
            description = "TCP port the API listens on.";
          };
          LISTEN = lib.mkOption {
            type = lib.types.str;
            default = "127.0.0.1";
            description = "IP address the API binds to. Loopback by default; expose it via a reverse proxy (e.g. caddy).";
          };
          DOCS_DIR = lib.mkOption {
            type = lib.types.str;
            default = "${cfg.package}/share/ladefuchs-api/docs";
            defaultText = lib.literalExpression "\${config.services.ladefuchs-api.package}/share/ladefuchs-api/docs";
            description = "Directory with the OpenAPI docs served under /docs.";
          };
        };
      };
      default = {};
      description = ''
        Environment variables passed to the API (see config.env.example).

        Secrets (DATABASE_URL, JWT_KEY, ECO_MOVEMENT_API_KEY, SLACK_TOKEN, ADMIN_PWD, ...)
        should go into {option}`services.ladefuchs-api.environmentFile`
        instead, so they do not end up in the world-readable Nix store.
      '';
      example = lib.literalExpression ''
        {
          DOMAIN = "https://api.ladefuchs.app";
          ADMIN_DOMAIN = "https://admin.ladefuchs.app";
          IMPORT_ON_START = true;
        }
      '';
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = ''
        Environment file with secrets, loaded by systemd at runtime.
        Required at minimum: DATABASE_URL (e.g. a
        `postgres://user:password@host/ladefuchs` connection string) and JWT_KEY.
      '';
    };

  };

  config = lib.mkIf cfg.enable {
    systemd.services.ladefuchs-api = {
      description = "Ladefuchs API";
      wantedBy = ["multi-user.target"];
      wants = ["network-online.target"];
      after =
        ["network-online.target"]
        ++ lib.optional config.services.postgresql.enable "postgresql.service";


      # tree_magic_mini needs the freedesktop mime DB (Dockerfile installs shared-mime-info)
      environment.XDG_DATA_DIRS = lib.makeSearchPath "share" [ pkgs.shared-mime-info ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/ladefuchs-api";
        EnvironmentFile = [
          (settingsFormat.generate "ladefuchs-api.env" cfg.settings)
        ] ++ lib.optional (cfg.environmentFile != null) cfg.environmentFile;
        WorkingDirectory = "/var/lib/ladefuchs-api";
        StateDirectory = "ladefuchs-api";
        Restart = "on-failure";
        RestartSec = 5;
        User = cfg.user;
        Group = cfg.group;
        DynamicUser = cfg.dynamicUser;
        NoNewPrivileges = true;
        PrivateTmp = true;
      };
    };
  };
}
