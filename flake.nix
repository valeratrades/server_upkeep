{
  nixConfig = {
    extra-substituters = [ "https://valeratrades.cachix.org" ];
    extra-trusted-public-keys = [ "valeratrades.cachix.org-1:gXVwhzO5YB+BaiEJYT48qZgzdaErGQew6xtZcz4Fo1Q=" ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/f61125a668a320878494449750330ca58b78c557";
    rust-overlay.url = "github:oxalica/rust-overlay/7ed7e8c74be95906275805db68201e74e9904f07";
    flake-utils.url = "github:numtide/flake-utils/11707dc2f618dd54ca8739b309ec4fc024de578b";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix/ca5b894d3e3e151ffc1db040b6ce4dcc75d31c37";
    v_flakes.url = "github:valeratrades/v_flakes/257142a54b071bb8a8b2e031d69e70f416518a5f";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils, pre-commit-hooks, v_flakes }:
    let
      manifest = (nixpkgs.lib.importTOML ./Cargo.toml).package;
      pname = manifest.name;
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          allowUnfree = true;
        };
        ##NB: can't load rust-bin from nightly.latest, as there are week guarantees of which components will be available on each day.
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" "rust-docs" "rustc-codegen-cranelift-preview" ];
        });
        pre-commit-check = pre-commit-hooks.lib.${system}.run (v_flakes.files.preCommit { inherit pkgs; });
        stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;

        rs = v_flakes.rs {
          inherit pkgs rust;
          cranelift = true;
          build = {
            enable = true;
            workspace."./" = [ "git_version" "log_directives" ];
          };
        };
        github = v_flakes.github {
          inherit pkgs pname rs;
          enable = true;
          lastSupportedVersion = "nightly-2025-12-09";
          jobs.default = true;
          jobs.warnings.install = { packages = [ "mold" ]; debug = true; };
          release.default = true;
        };
        readme = v_flakes.readme-fw {
          inherit pkgs pname;
          lastSupportedVersion = "nightly-1.93";
          rootDir = ./.;
          licenses = [{ license = v_flakes.files.licenses.blue_oak; }];
          badges = [ "msrv" "crates_io" "docs_rs" "loc" "ci" ];
        };
        combined = v_flakes.utils.combine [ rs github readme ];
      in
      {
        packages =
          let
            rustc = rust;
            cargo = rust;
            rustPlatform = pkgs.makeRustPlatform {
              inherit rustc cargo stdenv;
            };
          in
          {
            default = rustPlatform.buildRustPackage {
              inherit pname;
              version = manifest.version;

              buildInputs = with pkgs; [
                openssl.dev
              ];
              nativeBuildInputs = with pkgs; [ pkg-config ];

              cargoLock.lockFile = ./Cargo.lock;
              src = pkgs.lib.cleanSource ./.;
            };
          };

        devShells.default =
          with pkgs;
          mkShell {
            inherit stdenv;
            shellHook =
              pre-commit-check.shellHook +
              combined.shellHook +
              ''
                cp -f ${(v_flakes.files.treefmt) { inherit pkgs; }} ./.treefmt.toml
              '';
            packages = [
              mold-wrapped
              openssl
              pkg-config
              rust
            ] ++ pre-commit-check.enabledPackages ++ combined.enabledPackages;

            env = {
              RUST_BACKTRACE = 1;
              RUST_LIB_BACKTRACE = 0;
              CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG = true;
            };
          };
      }
    )
    // {
      # System service running `server_upkeep monitor`. Secrets are NOT baked in:
      # the alert sink arrives as SERVER_UPKEEP__ALERT* env vars from `environmentFile`
      # (config-rs reads them; precedence is env < file < flags), so nothing sensitive
      # ever lands in the nix store. `alert` is either a command string
      # (SERVER_UPKEEP__ALERT="v_notify -a tg -l error -") that the alert text is piped
      # into, or telegram creds (SERVER_UPKEEP__ALERT__BOT_TOKEN / __ALERTS_CHAT).
      # `maxSize` is the only non-secret knob, passed the same way to avoid a config file.
      nixosModules."${pname}" = { config, lib, pkgs, ... }:
        let
          inherit (lib) mkEnableOption mkOption mkIf types optional optionalString optionalAttrs;
          cfg = config.services."${pname}";
          isNixCfg = cfg.configFile != null && lib.hasSuffix ".nix" cfg.configFile;
        in
        {
          options.services."${pname}" = {
            enable = mkEnableOption "server_upkeep disk/state monitor (alerts via telegram or a command)";

            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.default;
              description = "The package to use.";
            };

            environmentFile = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = ''
                systemd EnvironmentFile providing the secrets referenced by the
                config, e.g. SERVER_UPKEEP__ALERT (a command string) or
                SERVER_UPKEEP__ALERT__BOT_TOKEN + SERVER_UPKEEP__ALERT__ALERTS_CHAT
                for telegram. The unit stays inactive until this file exists, so it
                never crash-loops before provisioning.
              '';
            };

            configFile = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = ''
                Optional config file (.nix/.toml/…). Leave null to configure purely
                via env vars. A `.nix` config makes the binary shell out to `nix
                eval` at runtime, so `nix` is added to the unit PATH automatically.
                Never commit secrets here — use environmentFile.
              '';
            };

            maxSize = mkOption {
              type = types.str;
              default = "10GB";
              description = "Alert threshold for the monitored state dir size.";
            };
          };

          config = mkIf cfg.enable {
            systemd.services."${pname}" = {
              description = "server_upkeep monitor (disk + state-dir thresholds -> alert)";
              after = [ "network-online.target" ];
              wants = [ "network-online.target" ];
              wantedBy = [ "multi-user.target" ];
              path = optional isNixCfg pkgs.nix;
              environment = {
                "SERVER_UPKEEP__MONITOR__MAX_SIZE" = cfg.maxSize;
                # DynamicUser has no $HOME — v_utils reads it directly (xdg.rs) and
                # panics otherwise. Point both it and XDG_STATE_HOME at StateDirectory
                # so dirs::state_dir() resolves and xdg_state_file writes land there.
                HOME = "/var/lib/${pname}";
                XDG_STATE_HOME = "/var/lib/${pname}";
                SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
              };
              unitConfig = optionalAttrs (cfg.environmentFile != null) {
                ConditionPathExists = cfg.environmentFile;
              };
              serviceConfig = {
                ExecStart = "${cfg.package}/bin/${pname} monitor"
                  + optionalString (cfg.configFile != null) " --config ${cfg.configFile}";
                EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;
                Restart = "on-failure";
                RestartSec = 10;
                DynamicUser = true;
                StateDirectory = pname;
              };
            };
          };
        };
    };
}
