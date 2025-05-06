weather-cli:
{
  pkgs,
  lib,
  config,
  ...
}@args:
let
  cfg = config.programs.weather-cli;
  pkg = weather-cli.packages.${pkgs.system}.default;
in
{
  options.programs.weather-cli = {
    enable = lib.mkEnableOption "Weather-cli";

    package = lib.mkPackageOption pkgs "weather-cli" { } // {
      default = pkg;
    };

    provider = lib.mkOption {
      type = lib.types.enum [
        "open-meteo"
        "open-weather-map"
      ];
      default = "open-meteo";
    };

    apiKey = lib.mkOption {
      type = with lib.types; nullOr str;
      default = null;
    };

    location = lib.mkOption {
      type = with lib.types; nullOr (either (listOf str) (listOf float));
      default = null;
      example = [
        "Berlin"
        "DE"
      ];
    };

    units = lib.mkOption {
      type = lib.types.enum [
        "metric"
        "imperial"
      ];
      default = "metric";
    };

    timeFormat = lib.mkOption {
      type = lib.types.enum [
        "12h"
        "24h"
      ];
      default = "24h";
    };

    cachingDuration = lib.mkOption {
      type = lib.types.strMatching "^[0-9]+(min|h)$";
      default = "1h";
    };
  };

  config = lib.mkIf cfg.enable {

    home.packages = [ cfg.package ];

    xdg.configFile."weather-cli.toml".text =
      ''provider = "${cfg.provider}"
${if (cfg.apiKey != null) then "api_key = \"${cfg.apiKey}\"" else ""}
${if (cfg.location != null) then "location = [${lib.concatStringsSep ", " (map (loc: if builtins.isString loc then "\"${loc}\"" else toString loc) cfg.location)}]" else "" }
units = "${cfg.units}"
time_format = "${cfg.timeFormat}"
caching_duration = "${cfg.cachingDuration}"'';
  };
}
