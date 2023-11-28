This program is a login-shell wrapper that will delegate
~/.nix-profile/bin/login-shell if it exists and can be executed. This program
can be listed in /etc/shells while allowing users to configure their true login
shell using home-manager.

If the link does not exist or the program cannot be executed for some reason it
will fallback to shells listed in /etc/shells with a path beginning with /bin.
The limitation to shells under /bin is to avoid creating an inifinite
recursion, a better solution would be nice.

## Installation

I haven't figured out a good way of installing flake-defined packages with nix-env so I have followed a two-step process:

1. Use `nix build .` to build the package and create the result link
2. Use `sudo nix-env -i ./result` to install the package into the system profile

Then add `/nix/var/nix/profiles/default/bin/hm-login-shell-helper` to
/etc/shells so that it is a valid selection for users.

## User Configuration

The `./hm-modules/login-shell.nix` module for home-manager defines a new option
`home.loginShell`. When set this option creates a link in your user environment
from bin/login-shell to the specified path. This will get picked up by
hm-login-shell-helper and used as the delegated shell.

## Known Issues

Currently it is hardcoded to use ~/.nix-profile as the nix profile.

The restriction of falling back to shells only starting with /bin won't work in
nixos probably.
