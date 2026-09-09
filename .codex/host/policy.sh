#!/bin/sh
# Shared constants for the Codex Rust host boundary. Keep this file POSIX so
# the PATH dispatcher and the environment installer read one policy.

CODEX_RUST_HOST_POLICY_VERSION=11
CODEX_RUST_SCCACHE_POLICY_KEY=0.17.0-p2
CODEX_RUST_SCCACHE_VERSION=0.17.0
CODEX_RUST_MAX_ARGS=512
CODEX_RUST_MAX_JOBS=4
CODEX_RUST_PARENT_SLICE=codex-rust.slice
CODEX_RUST_NNSIMS_SLICE=codex-rust-nnsims.slice
CODEX_RUST_BABYLON_SLICE=codex-rust-babylon.slice
CODEX_RUST_NNSIMS_TARGET_SUBDIR=target
CODEX_RUST_BABYLON_TARGET_SUBDIR=rust/target

codex_rust_repository_for_remote() {
  case "$1" in
    https://github.com/percy-raskova/nnsims|\
    https://github.com/percy-raskova/nnsims.git|\
    git@github.com:percy-raskova/nnsims|\
    git@github.com:percy-raskova/nnsims.git|\
    ssh://git@github.com/percy-raskova/nnsims|\
    ssh://git@github.com/percy-raskova/nnsims.git)
      printf '%s\n' nnsims
      ;;
    https://github.com/percy-raskova/babylon|\
    https://github.com/percy-raskova/babylon.git|\
    git@github.com:percy-raskova/babylon|\
    git@github.com:percy-raskova/babylon.git|\
    ssh://git@github.com/percy-raskova/babylon|\
    ssh://git@github.com/percy-raskova/babylon.git)
      printf '%s\n' babylon
      ;;
    *)
      return 1
      ;;
  esac
}

codex_rust_slice_for_repository() {
  case "$1" in
    nnsims)
      printf '%s\n' "$CODEX_RUST_NNSIMS_SLICE"
      ;;
    babylon)
      printf '%s\n' "$CODEX_RUST_BABYLON_SLICE"
      ;;
    *)
      return 1
      ;;
  esac
}

codex_rust_target_dir_for_repository() {
  case "$1" in
    nnsims)
      printf '%s/%s\n' "$2" "$CODEX_RUST_NNSIMS_TARGET_SUBDIR"
      ;;
    babylon)
      printf '%s/%s\n' "$2" "$CODEX_RUST_BABYLON_TARGET_SUBDIR"
      ;;
    *)
      return 1
      ;;
  esac
}

codex_rust_bootstrap_toolchain_for_repository() {
  case "$1" in
    nnsims)
      printf '\n'
      ;;
    babylon)
      printf '%s\n' +1.91.1
      ;;
    *)
      return 1
      ;;
  esac
}

codex_rust_cache_root() {
  if [ -n "${CODEX_RUST_CACHE_ROOT:-}" ]; then
    printf '%s\n' "$CODEX_RUST_CACHE_ROOT"
  elif [ -d /media/user/data ] && [ -w /media/user/data ]; then
    printf '%s\n' /media/user/data/codex-cache
  else
    printf '%s\n' "${XDG_CACHE_HOME:-$HOME/.cache}/codex-rust"
  fi
}

codex_rust_tools_root() {
  if [ -n "${CODEX_RUST_TOOLS_ROOT:-}" ]; then
    printf '%s\n' "$CODEX_RUST_TOOLS_ROOT"
  elif [ -d /media/user/data ] && [ -w /media/user/data ]; then
    printf '%s\n' /media/user/data/codex-tools
  else
    printf '%s\n' "${XDG_DATA_HOME:-$HOME/.local/share}/codex-tools"
  fi
}

codex_rust_sccache_matches_version() {
  codex_rust_sccache_candidate="$1"
  [ -x "$codex_rust_sccache_candidate" ] &&
    [ "$("$codex_rust_sccache_candidate" --version 2>/dev/null || true)" = \
      "sccache $CODEX_RUST_SCCACHE_VERSION" ]
}

codex_rust_sccache_bin() (
  codex_rust_tools_root_value="$1"
  codex_rust_flat_binary="$codex_rust_tools_root_value/sccache/$CODEX_RUST_SCCACHE_VERSION/sccache"
  codex_rust_installed_binary="$codex_rust_tools_root_value/sccache/$CODEX_RUST_SCCACHE_VERSION/bin/sccache"
  if codex_rust_sccache_matches_version "$codex_rust_flat_binary"; then
    printf '%s\n' "$codex_rust_flat_binary"
  elif codex_rust_sccache_matches_version "$codex_rust_installed_binary"; then
    printf '%s\n' "$codex_rust_installed_binary"
  else
    # The setup installs here. Returning the expected path keeps the caller's
    # refusal specific when neither supported layout holds the pinned binary.
    printf '%s\n' "$codex_rust_installed_binary"
  fi
)

codex_rust_process_is_in_slice() {
  codex_rust_expected_slice="$1"
  codex_rust_cgroup_file="$2"
  [ -r "$codex_rust_cgroup_file" ] &&
    grep -F -- "/$codex_rust_expected_slice/" "$codex_rust_cgroup_file" >/dev/null 2>&1
}

codex_rust_directory_is_lexically_physical() {
  codex_rust_directory="$1"
  case "$codex_rust_directory" in
    /*)
      ;;
    *)
      return 1
      ;;
  esac
  [ -d "$codex_rust_directory" ] &&
    [ ! -L "$codex_rust_directory" ] &&
    [ "$(cd -- "$codex_rust_directory" && pwd -P)" = "$codex_rust_directory" ]
}

codex_rust_create_physical_directory() {
  codex_rust_directory="$1"
  case "$codex_rust_directory" in
    /*)
      ;;
    *)
      return 1
      ;;
  esac
  [ "$(readlink -m -- "$codex_rust_directory")" = "$codex_rust_directory" ] || return 1
  mkdir -p "$codex_rust_directory" &&
    codex_rust_directory_is_lexically_physical "$codex_rust_directory"
}

codex_rust_unit_property_is() {
  codex_rust_unit="$1"
  codex_rust_property="$2"
  codex_rust_expected="$3"
  codex_rust_actual="$(
    /usr/bin/systemctl --user show "$codex_rust_unit" \
      --property="$codex_rust_property" --value 2>/dev/null || true
  )"
  [ "$codex_rust_actual" = "$codex_rust_expected" ]
}

codex_rust_slice_matches_policy() {
  codex_rust_slice="$1"
  codex_rust_host_dir="$2"
  codex_rust_expected_fragment="$codex_rust_host_dir/systemd/$codex_rust_slice"
  codex_rust_fragment="$(
    /usr/bin/systemctl --user show "$codex_rust_slice" \
      --property=FragmentPath --value 2>/dev/null || true
  )"
  [ -n "$codex_rust_fragment" ] &&
    [ -f "$codex_rust_fragment" ] &&
    [ -f "$codex_rust_expected_fragment" ] &&
    cmp -s "$codex_rust_expected_fragment" "$codex_rust_fragment" || return 1

  case "$codex_rust_slice" in
    codex-rust.slice)
      codex_rust_unit_property_is "$codex_rust_slice" CPUQuotaPerSecUSec 8s &&
        codex_rust_unit_property_is "$codex_rust_slice" MemoryHigh 21474836480 &&
        codex_rust_unit_property_is "$codex_rust_slice" MemoryMax 25769803776 &&
        codex_rust_unit_property_is "$codex_rust_slice" MemorySwapMax 4294967296 &&
        codex_rust_unit_property_is "$codex_rust_slice" IOWeight 500 &&
        codex_rust_unit_property_is "$codex_rust_slice" TasksMax 1024
      ;;
    codex-rust-nnsims.slice|codex-rust-babylon.slice)
      codex_rust_unit_property_is "$codex_rust_slice" CPUQuotaPerSecUSec 4s &&
        codex_rust_unit_property_is "$codex_rust_slice" MemoryHigh 10737418240 &&
        codex_rust_unit_property_is "$codex_rust_slice" MemoryMax 12884901888 &&
        codex_rust_unit_property_is "$codex_rust_slice" MemorySwapMax 2147483648 &&
        codex_rust_unit_property_is "$codex_rust_slice" IOWeight 500 &&
        codex_rust_unit_property_is "$codex_rust_slice" TasksMax 512
      ;;
    *)
      return 1
      ;;
  esac
}

codex_rust_cache_dir() {
  printf '%s/sccache/%s/%s\n' "$2" "$1" "$CODEX_RUST_SCCACHE_POLICY_KEY"
}

codex_rust_server_socket() {
  printf '%s/sccache-server/%s-%s.sock\n' \
    "$2" "$1" "$CODEX_RUST_SCCACHE_POLICY_KEY"
}
