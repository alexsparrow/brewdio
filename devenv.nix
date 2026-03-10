{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [ pkgs.git ];

  # https://devenv.sh/languages/
  # languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  # Shim for Cargokit (FRB) which requires rustup but we use Nix-managed Rust.
  scripts.rustup.exec = ''
    case "$1" in
      toolchain)
        if [ "$2" = "list" ]; then
          echo "stable-aarch64-apple-darwin (default)"
          exit 0
        elif [ "$2" = "install" ]; then
          exit 0
        fi
        ;;
      target)
        if [ "$2" = "list" ]; then
          SYSROOT=$(rustc --print sysroot 2>/dev/null)
          if [ -d "$SYSROOT/lib/rustlib" ]; then
            ls "$SYSROOT/lib/rustlib/" | grep -v -E '^(src|etc)$' | grep -v '\.'
          fi
          exit 0
        elif [ "$2" = "add" ]; then
          exit 0
        fi
        ;;
      component)
        exit 0
        ;;
      run)
        shift 2
        exec "$@"
        ;;
    esac
    echo "rustup-shim: unhandled args: $@" >&2
    exit 1
  '';

  # https://devenv.sh/basics/
  enterShell = ''
    hello         # Run scripts directly
    git --version # Use packages
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';

  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;

  languages.rust = {
	enable = true;
	channel = "stable";
	targets = [ "wasm32-unknown-unknown" "aarch64-unknown-linux-gnu" "aarch64-linux-android" "x86_64-linux-android" "armv7-linux-androideabi" "i686-linux-android" ];
  };

  android = {
    enable = true;
    flutter.enable = true;
    platforms.version = [ "36" ];
    systemImageTypes = [ "google_apis_playstore" ];
    abis = [ "arm64-v8a" ];
    buildTools.version = [ "35.0.0" "36.0.0" ];
    emulator.enable = true;
    systemImages.enable = true;
    ndk.enable = true;
    sources.enable = false;
    extraLicenses = [
      "android-sdk-preview-license"
    ];
  };
}
