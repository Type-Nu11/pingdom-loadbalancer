#!/usr/bin/env bash
set -euo pipefail

deploy_dir="${1:?deployment directory is required}"
archive="${2:?release archive is required}"
tls_cert_dir="${3:?TLS certificate directory is required}"

[[ "$deploy_dir" =~ ^/[A-Za-z0-9._/-]+$ && "$deploy_dir" != / && "$deploy_dir" != *'/..'* ]]
[[ "$archive" =~ ^/tmp/typenull-edge-[0-9a-f]{40}\.tar\.gz$ ]]
[[ "$tls_cert_dir" =~ ^/[A-Za-z0-9._/-]+$ && "$tls_cert_dir" != / && "$tls_cert_dir" != *'/..'* ]]
[[ -f "$archive" ]]
command -v docker >/dev/null
docker compose version >/dev/null

mkdir -p "$deploy_dir/releases"
release_dir="$(mktemp -d "$deploy_dir/releases/.pending.XXXXXXXX")"
cleanup() {
  rm -f -- "$archive"
  if [[ -d "$release_dir" ]]; then
    rmdir -- "$release_dir" 2>/dev/null || true
  fi
}
trap cleanup EXIT

tar -xzf "$archive" -C "$release_dir"
export TLS_CERT_DIR="$tls_cert_dir"
docker compose -p typenull-edge -f "$release_dir/deploy/rust.compose.yml" config --quiet
docker compose -p typenull-edge -f "$release_dir/deploy/rust.compose.yml" build
docker compose -p typenull-edge -f "$release_dir/deploy/rust.compose.yml" up -d --no-build
sleep 3
[[ "$(docker inspect --format '{{.State.Running}}' typenull-edge-rust)" == true ]]

# Keep the extracted release so the Compose build context remains inspectable.
printf 'Deployed release at %s\n' "$release_dir"
