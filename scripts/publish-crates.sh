#!/usr/bin/env bash
set -euo pipefail

# GLASTOR® Strata — Publication Pipeline for crates.io
# Strict topological dependency order:
# 1. glastor-aria-core
# 2. glastor-css-binder
# 3. wait 30s for crates.io index propagation
# 4. glastor-ui-cli

PUBLISH_ARGS=()
IS_DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      PUBLISH_ARGS+=("--dry-run")
      IS_DRY_RUN=true
      shift
      ;;
    --token)
      export CARGO_REGISTRY_TOKEN="$2"
      shift 2
      ;;
    *)
      echo "Unknown flag: $1"
      exit 1
      ;;
  esac
done

echo "========================================================"
echo "  GLASTOR® Strata — Pipeline de Publicación a crates.io"
echo "========================================================"

# 1. Pre-flight verification
echo -e "\n[Paso 1/4] Ejecutando suite de pruebas y linters..."
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check -p glastor-aria-core --no-default-features
cargo test --workspace --all-features

# 2. Publish glastor-aria-core
echo -e "\n[Paso 2/4] Publicando glastor-aria-core v0.1.0..."
cargo publish -p glastor-aria-core "${PUBLISH_ARGS[@]}"

# 3. Publish glastor-css-binder
echo -e "\n[Paso 3/4] Publicando glastor-css-binder v0.1.0..."
cargo publish -p glastor-css-binder "${PUBLISH_ARGS[@]}"

# 4. Wait for propagation if not dry-run
if [[ "${IS_DRY_RUN}" == "false" ]]; then
  echo -e "\n[Pausa] Esperando 30 segundos para actualización del índice de crates.io..."
  sleep 30
fi

# 5. Publish glastor-ui-cli
echo -e "\n[Paso 4/4] Publicando glastor-ui-cli v0.1.0..."
cargo publish -p glastor-ui-cli "${PUBLISH_ARGS[@]}"

echo -e "\n========================================================"
echo "  ¡Publicación completada exitosamente!"
echo "  https://crates.io/crates/glastor-aria-core"
echo "  https://crates.io/crates/glastor-css-binder"
echo "  https://crates.io/crates/glastor-ui-cli"
echo "========================================================"
