{
  description = ":3";
  inputs = {
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust = {
      url = "github:arcnmx/nixexprs-rust";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs = inputs @ { self, rust, flake-utils, nixpkgs, ... }: let
    nixlib = nixpkgs.lib;
    rust-w64-overlay = { rust-w64 }: let
      ccEnvVar = n: builtins.replaceStrings [ "-" ] [ "_" ] n;
      cargoEnvVar = n: ccEnvVar (nixlib.toUpper n);
      #stdenv = rust-w64.pkgs.clangStdenv;
      stdenv = rust-w64.pkgs.stdenv;
      targetEnv = {
        inherit (rust-w64) pkgs;
        inherit stdenv;
        ar = "${rust-w64.pkgs.stdenv.cc.cc}/bin/x86_64-w64-mingw32-gcc-ar";
      };
    in cself: csuper: let
      target = rust-w64.lib.rustTargetEnvironment (targetEnv // {
        rustcFlags = [
          "-L native=${rust-w64.pkgs.windows.pthreads}/lib"
        ];
      });
      cxxflags = self.lib.cxxflags;
      rustcflags = [
        #"-Clink-arg=-Wl,--gc-sections"
      ];
    in {
      targetRustLld = csuper.targetRustLld || true;
      sysroot-std = csuper.sysroot-std ++ [ cself.manifest.targets.${target.triple}.rust-std ];
      cargo-cc = csuper.cargo-cc // cself.context.rlib.cargoEnv {
        inherit target;
      } // {
        "CFLAGS_${ccEnvVar target.triple}" = toString cxxflags;
        "CXXFLAGS_${ccEnvVar target.triple}" = toString cxxflags;
        RUSTC_FLAGS = toString rustcflags;
      };
      rust-cc = csuper.rust-cc // cself.context.rlib.rustCcEnv {
        inherit target;
      };
    };
  in flake-utils.lib.eachDefaultSystem (system: let
    legacyPackages = self.legacyPackages.${system};
    packages = self.packages.${system};
    pkgs = nixpkgs.legacyPackages.${system};
    pkgs-w64 =
      if pkgs.hostPlatform.isWindows then pkgs
      else pkgs.pkgsCross.mingwW64;
    rust-w64 = import rust { pkgs = pkgs-w64; };
    rust-w64-overlay' = rust-w64-overlay { inherit rust-w64; };
    channel-w64 = rust-w64.latest.override {
      channelOverlays = [ rust-w64-overlay' ];
    };
    rust'builders = {
      wrapSource = pkgs.callPackage rust.builders.wrapSource {};
      cargoOutputHashes = pkgs.callPackage rust.builders.cargoOutputHashes {};
      generateFiles = pkgs.callPackage rust.builders.generateFiles {};
    };
  in {
    devShells = import ./shell.nix {
      inherit inputs;
      inherit (legacyPackages) rust pkgs pkgs-w64 rust-w64-overlay;
    };

    legacyPackages = {
      inherit pkgs pkgs-w64;
      inherit rust-w64 channel-w64;
      rust = import rust { inherit pkgs; };
      rust-w64-overlay = rust-w64-overlay';

      source = rust'builders.wrapSource self.lib.crate.src;

      generate = rust'builders.generateFiles {
        paths = {
          "lock.nix" = legacyPackages.outputHashes;
        };
      };

      outputHashes = rust'builders.cargoOutputHashes {
        inherit (self.lib) crate;
      };
    };

    packages = {
      default = packages.arcloader;
      arcloader = pkgs-w64.callPackage ./derivation.nix {
        ${if !pkgs-w64.buildPlatform.isWindows then "rustPlatform" else null} = channel-w64.rustPlatform;
        #source = self.lib.crate.src;
        inherit (legacyPackages) source;
        inherit (self.lib) crate;
      };
      arcloader-debug = packages.arcloader.override {
        buildType = "debug";
      };
    };
  }) // {
    lib = {
      crate = rust.lib.importCargo {
        path = ./Cargo.toml;
        inherit (import ./lock.nix) outputHashes;
      };
      cxxflags = [
        "-Oz"
        "-march=x86-64-v3"
        "-fno-rtti" "-fno-exceptions" "-fnothrow-opt"
        "-fno-threadsafe-statics" "-fuse-cxa-atexit" # "-fvisibility-inlines-hidden"
      ];
    };
    inherit (self.lib.crate) version;
  };
}

