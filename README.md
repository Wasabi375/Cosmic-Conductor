# Cosmic Conductor

A command line utility to control windows and working groups on the [COSMIC desktop](https://system76.com/cosmic/).

## Status

The COSMIC desktop is currently in alpha. Therefor it is expected for things to change and
break. This program especially depends on the [cosmic wayland protocols](https://github.com/pop-os/cosmic-protocols)
which are not stabalized. Therefor it is likely that functionality will break with updates
to COSMIC. 

Currently this is build against COSMIC Alpha 7.

## Changes

This project has a [changelog](CHANGELOG.md)

## Install

### NixOs

After adding this flake(`"github:Wasabi375/nix-wasabipkgs/main"`) to your inputs
you can install this using `pkgs-wasabi.cosmic-conductor`.
```nix
inputs = {
  nixpkgs.url = "nixpkgs/nixos-25.05";
  wasabi375.url = "github:Wasabi375/nix-wasabipkgs/main";
  wasabi375.inputs.nixpkgs.follows = "nixpkgs";
};

outputs = { self, nixpkgs, wasabi375, ... }:
let
  lib = nixpkgs.lib;
  pkgs-wasabi = wasabi375.legacyPackages.x86_64-linux;
in
{
  nixosConfigurations.x86_64-linux = lib.nixosSystem {
    modules = [
      ./configuration.nix
    ];
    specialArgs = {
      inherit pkgs-wasabi;
    };
  };
};
```


## From Source

After cloning this repository you can simply install cosmic-conductor using cargo.
```
cargo install --path .
```

To install man pages and completions you can go into the `xtask` directory and
run `cargo run -- man` and `cargo run -- completion`. This will generate the 
necessary files in `target/assets`.
See `cargo run -- help` for more information.


## Contributions

I welcome any contributions, from bug report, feature requests to pull requests.
However this is a hobby project and it might take some time for me to respond.
