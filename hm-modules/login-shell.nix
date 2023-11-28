{ config, pkgs, lib, ... }:
with lib;
{
  options = {
    home.loginShell = mkOption {
      type = types.pathInStore;
      default = null;
    };
  };

  config = {
    home.packages = mkIf (config.home.loginShell != null) [
     (pkgs.runCommandLocal "login-shell" {} ''
       mkdir -p $out/bin
       ln -s "${config.home.loginShell}" $out/bin/login-shell
     '')
    ];
  };
}
