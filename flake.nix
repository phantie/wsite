#
# nix build .#myServer -o "backend/build"
# nix build .#myClient -o "frontend/build"
#
{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # The version of wasm-bindgen-cli needs to match the version in Cargo.lock
    # Update this to include the version you need
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/4e6868b1aa3766ab1de169922bb3826143941973";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, nixpkgs-for-wasm-bindgen, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchain).overrideScope (_final: _prev: {
          # The version of wasm-bindgen-cli needs to match the version in Cargo.lock. You
          # can unpin this if your nixpkgs commit contains the appropriate wasm-bindgen-cli version
          inherit (import nixpkgs-for-wasm-bindgen { inherit system; }) wasm-bindgen-cli;
        });

        # When filtering sources, we want to allow assets other than .rs files
        src = lib.cleanSourceWith {
          src = ./.; # The original, unfiltered source
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            (lib.hasSuffix "favicon.ico" path) ||
            # Example of a folder for images, icons, etc
            (lib.hasInfix "/assets/" path) ||
            # Default filter from crane (allow .rs files)
            (craneLib.filterCargoSources path type)
          ;
        };


        # Arguments to be used by both the client and the server
        # When building a workspace with crane, it's a good idea
        # to set "pname" and "version".
        commonArgs = {
          inherit src;
          pname = "workspace";
          version = "0.1.0";
          strictDeps = true;

          buildInputs = with pkgs; [
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
        };

        # Native packages

        nativeArgs = commonArgs // {
          pname = "workspace-native";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly nativeArgs;

        # Simple JSON API that can be queried by the client
        myServer = craneLib.buildPackage (nativeArgs // {
          inherit cargoArtifacts;
          pname = "workspace-server";
          cargoExtraArgs = "--package=backend";
        });


        # Wasm packages

        # it's not possible to build the server on the
        # wasm32 target, so we only build the client.
        wasmArgs = commonArgs // {
          pname = "trunk-workspace";
          cargoExtraArgs = "--package=frontend";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };

        cargoArtifactsWasm = craneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });

        # Build the frontend of the application.
        # This derivation is a directory you can put on a webserver.
        myClient = craneLib.buildTrunkPackage (wasmArgs // {
          pname = "workspace-client";
          cargoArtifacts = cargoArtifactsWasm;
          trunkIndexPath = "frontend/index.html";
          # The version of wasm-bindgen-cli here must match the one from Cargo.lock.
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
            version = "0.2.89";
            hash = "sha256-IPxP68xtNSpwJjV2yNMeepAS0anzGl02hYlSTvPocz8=";
            cargoHash = "sha256-pBeQaG6i65uJrJptZQLuIaCb/WCQMhba1Z1OhYqA8Zc=";
          };
        });

      in
      {
        packages =
          {
            inherit
              myServer
              myClient
              ;
            default = myClient;
          };

        apps.default = flake-utils.lib.mkApp {
          name = "backend";
          drv = myServer;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = with pkgs; [
            trunk
          ];
        };

        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit myServer myClient;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          my-app-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Check formatting
          my-app-fmt = craneLib.cargoFmt commonArgs;
        };

      });
}
