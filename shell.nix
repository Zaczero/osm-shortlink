{ }:

let
  # Update packages with `nixpkgs-update` command
  pkgs =
    import
      (fetchTarball "https://github.com/NixOS/nixpkgs/archive/677fbe97984e7af3175b6c121f3c39ee5c8d62c9.tar.gz")
      { };

  packages' = with pkgs; [
    coreutils
    curl
    jq
    maturin
    python314
    ruff
    uv
    (lib.optional stdenv.isDarwin libiconv)

    (writeShellScriptBin "make" "maturin develop --uv")
    (writeShellScriptBin "run-tests" ''
      python -m pytest . \
        --verbose \
        --no-header
    '')
    (writeShellScriptBin "nixpkgs-update" ''
      hash=$(
        curl -fsSL \
          https://prometheus.nixos.org/api/v1/query \
          -d 'query=channel_revision{channel="nixpkgs-unstable"}' \
        | jq -r ".data.result[0].metric.revision")
      sed -i "s|nixpkgs/archive/[0-9a-f]\\{40\\}|nixpkgs/archive/$hash|" shell.nix
      echo "Nixpkgs updated to $hash"
    '')
  ];

  shell' = with pkgs; ''
    export TZ=UTC
    export NIX_SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt
    export SSL_CERT_FILE=$NIX_SSL_CERT_FILE
    export PYTHONNOUSERSITE=1
    export PYTHONPATH=""

    current_python=$(readlink -e .venv/bin/python || echo "")
    current_python=''${current_python%/bin/*}
    [ "$current_python" != "${python314}" ] && rm -rf .venv/

    echo "Installing Python dependencies"
    export UV_NATIVE_TLS=true
    export UV_PYTHON="${python314}/bin/python"
    NIX_ENFORCE_PURITY=0 uv sync --frozen

    echo "Activating Python virtual environment"
    source .venv/bin/activate
  '';
in
pkgs.mkShell {
  buildInputs = packages';
  shellHook = shell';
}
