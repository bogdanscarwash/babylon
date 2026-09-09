#!/bin/sh
# Install or verify the per-repository Cargo dispatcher and user slices.
set -eu

installer_path="$(readlink -f -- "$0")"
host_source_dir="$(dirname -- "$installer_path")"
. "$host_source_dir/policy.sh"

case "${1:-install}" in
  install)
    operation=install
    ;;
  check|--check)
    operation=check
    ;;
  *)
    echo "usage: $0 [install|--check]" >&2
    exit 64
    ;;
esac
if [ "$#" -gt 1 ]; then
  echo "usage: $0 [install|--check]" >&2
  exit 64
fi

config_root="${XDG_CONFIG_HOME:-$HOME/.config}"
data_root="${XDG_DATA_HOME:-$HOME/.local/share}"
local_bin="${CODEX_RUST_LOCAL_BIN:-$HOME/.local/bin}"
policy_root="$data_root/codex-rust-host"
install_root="$policy_root/v$CODEX_RUST_HOST_POLICY_VERSION"
install_lock="$policy_root/install.lock"
policy_high_water="$policy_root/highest-policy-version"
dispatcher_bin="$policy_root/bin"
systemd_root="$config_root/systemd/user"
cargo_shim="$local_bin/cargo"
dispatcher_shim="$dispatcher_bin/cargo"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
real_cargo_proxy="$cargo_home/bin/cargo"
managed_marker_prefix='# Managed by nnsims .codex/host/install.sh (policy v'
managed_marker="$managed_marker_prefix$CODEX_RUST_HOST_POLICY_VERSION)."

fail() {
  echo "codex Rust host install: $*" >&2
  exit 1
}

acquire_install_lock() {
  command -v flock >/dev/null 2>&1 || fail "flock is required to serialize host-policy installation"
  mkdir -p "$policy_root"
  if [ -L "$install_lock" ] || { [ -e "$install_lock" ] && [ ! -f "$install_lock" ]; }; then
    fail "refusing non-regular install lock $install_lock"
  fi
  exec 9>>"$install_lock" || fail "cannot open install lock $install_lock"
  flock 9 || fail "cannot acquire install lock $install_lock"
}

read_policy_high_water() {
  installed_highest_version=0
  if [ ! -e "$policy_high_water" ] && [ ! -L "$policy_high_water" ]; then
    return
  fi
  if [ -L "$policy_high_water" ] || [ ! -f "$policy_high_water" ]; then
    fail "refusing non-regular policy high-water mark $policy_high_water"
  fi
  if [ "$(wc -l <"$policy_high_water" | tr -d ' ')" -ne 1 ] ||
    ! grep -Eq '^[0-9]+$' "$policy_high_water"; then
    fail "invalid policy high-water mark in $policy_high_water"
  fi
  installed_highest_version="$(sed -n '1p' "$policy_high_water")"
  if [ "$installed_highest_version" -gt "$CODEX_RUST_HOST_POLICY_VERSION" ]; then
    fail "refusing to downgrade host policy from v$installed_highest_version"
  fi
}

persist_policy_high_water() {
  if [ "$installed_highest_version" -eq "$CODEX_RUST_HOST_POLICY_VERSION" ]; then
    return
  fi
  high_water_temp="$(mktemp "$policy_root/.highest-policy-version.XXXXXX")" ||
    fail "cannot create a policy high-water mark"
  printf '%s\n' "$CODEX_RUST_HOST_POLICY_VERSION" >"$high_water_temp"
  chmod 600 "$high_water_temp"
  mv -f -- "$high_water_temp" "$policy_high_water"
}

assert_source() {
  source_name="$1"
  [ -f "$host_source_dir/$source_name" ] || fail "missing source file $source_name"
}

assert_managed_unit_or_absent() {
  unit_name="$1"
  unit_path="$systemd_root/$unit_name"
  if [ ! -e "$unit_path" ] && [ ! -L "$unit_path" ]; then
    return
  fi
  if [ -L "$unit_path" ]; then
    fail "refusing to overwrite user-unit symlink $unit_path"
  fi
  installed_marker="$(sed -n '1p' "$unit_path" 2>/dev/null || true)"
  if [ "$installed_marker" = "$managed_marker" ]; then
    cmp -s "$host_source_dir/systemd/$unit_name" "$unit_path" ||
      fail "installed $unit_name differs within policy v$CODEX_RUST_HOST_POLICY_VERSION"
    return
  fi
  case "$installed_marker" in
    "$managed_marker_prefix"*').')
      installed_version="${installed_marker#"$managed_marker_prefix"}"
      installed_version="${installed_version%).}"
      case "$installed_version" in
        ''|*[!0-9]*) fail "invalid managed marker in $unit_path" ;;
      esac
      if [ "$installed_version" -gt "$CODEX_RUST_HOST_POLICY_VERSION" ]; then
        fail "refusing to downgrade $unit_name from policy v$installed_version"
      fi
      ;;
    *)
    fail "refusing to overwrite unrelated user unit $unit_path"
      ;;
  esac
}

install_root_missing=0
assert_complete_install_root_or_absent() {
  if [ ! -e "$install_root" ] && [ ! -L "$install_root" ]; then
    install_root_missing=1
    return
  fi
  if [ -L "$install_root" ] || [ ! -d "$install_root" ]; then
    fail "refusing non-directory versioned policy root $install_root"
  fi
  unexpected_top="$(
    find "$install_root" -mindepth 1 -maxdepth 1 \
      ! -name cargo ! -name cargo-config.py ! -name policy.sh \
      ! -name real ! -name systemd -print -quit
  )"
  [ -z "$unexpected_top" ] || fail "versioned policy root contains an unrelated entry: $unexpected_top"
  for installed_dir in real systemd; do
    [ -d "$install_root/$installed_dir" ] && [ ! -L "$install_root/$installed_dir" ] ||
      fail "incomplete versioned policy bundle at $install_root/$installed_dir"
  done
  unexpected_real="$(find "$install_root/real" -mindepth 1 -maxdepth 1 ! -name cargo -print -quit)"
  [ -z "$unexpected_real" ] || fail "versioned real-Cargo directory contains an unrelated entry: $unexpected_real"
  unexpected_systemd="$(
    find "$install_root/systemd" -mindepth 1 -maxdepth 1 \
      ! -name codex-rust.slice \
      ! -name codex-rust-nnsims.slice \
      ! -name codex-rust-babylon.slice -print -quit
  )"
  [ -z "$unexpected_systemd" ] || fail "versioned unit directory contains an unrelated entry: $unexpected_systemd"
  for installed_name in cargo cargo-config.py policy.sh \
    systemd/codex-rust.slice \
    systemd/codex-rust-nnsims.slice \
    systemd/codex-rust-babylon.slice; do
    installed_path="$install_root/$installed_name"
    if [ -L "$installed_path" ] || [ ! -f "$installed_path" ]; then
      fail "incomplete versioned policy bundle at $installed_path"
    fi
    cmp -s "$host_source_dir/$installed_name" "$installed_path" ||
      fail "installed $installed_name differs within policy v$CODEX_RUST_HOST_POLICY_VERSION"
  done
  installed_proxy="$install_root/real/cargo"
  [ -L "$installed_proxy" ] || fail "$installed_proxy is not a managed symlink"
  [ "$(readlink -f -- "$installed_proxy")" = "$(readlink -f -- "$real_cargo_proxy")" ] ||
    fail "installed real-Cargo proxy differs within policy v$CODEX_RUST_HOST_POLICY_VERSION"
}

stage_complete_install_root() {
  [ "$install_root_missing" -eq 1 ] || return 0
  staged_install_root="$(mktemp -d "$policy_root/.v$CODEX_RUST_HOST_POLICY_VERSION.stage.XXXXXX")" ||
    fail "cannot stage the versioned policy bundle"
  trap 'if [ -n "${staged_install_root:-}" ]; then rm -rf -- "$staged_install_root"; fi' EXIT
  mkdir -p "$staged_install_root/real" "$staged_install_root/systemd"
  install -m 755 "$host_source_dir/cargo" "$staged_install_root/cargo"
  install -m 644 "$host_source_dir/cargo-config.py" "$staged_install_root/cargo-config.py"
  install -m 644 "$host_source_dir/policy.sh" "$staged_install_root/policy.sh"
  for unit_name in codex-rust.slice codex-rust-nnsims.slice codex-rust-babylon.slice; do
    install -m 644 "$host_source_dir/systemd/$unit_name" "$staged_install_root/systemd/$unit_name"
  done
  ln -s "$real_cargo_proxy" "$staged_install_root/real/cargo"
  mv -- "$staged_install_root" "$install_root"
  staged_install_root=""
}

assert_managed_shim_or_absent() {
  managed_shim="$1"
  if [ ! -e "$managed_shim" ] && [ ! -L "$managed_shim" ]; then
    return
  fi
  if [ ! -L "$managed_shim" ]; then
    fail "refusing to overwrite unrelated Cargo shim $managed_shim"
  fi
  shim_target="$(readlink -f -- "$managed_shim" 2>/dev/null || true)"
  case "$shim_target" in
    "$data_root/codex-rust-host/"*)
      ;;
    *)
      fail "refusing to replace unrelated Cargo symlink $managed_shim -> $shim_target"
      ;;
  esac
}

assert_dispatcher_bin_or_absent() {
  if [ ! -e "$dispatcher_bin" ] && [ ! -L "$dispatcher_bin" ]; then
    return
  fi
  if [ -L "$dispatcher_bin" ] || [ ! -d "$dispatcher_bin" ]; then
    fail "refusing non-directory managed shim root $dispatcher_bin"
  fi
  unexpected_shim="$(find "$dispatcher_bin" -mindepth 1 -maxdepth 1 ! -name cargo -print -quit)"
  [ -z "$unexpected_shim" ] ||
    fail "managed shim root contains an unrelated entry: $unexpected_shim"
}

assert_installed_file() {
  installed_name="$1"
  [ -f "$install_root/$installed_name" ] || fail "missing installed $installed_name"
  cmp -s "$host_source_dir/$installed_name" "$install_root/$installed_name" ||
    fail "installed $installed_name differs from repository policy"
}

assert_installed_unit() {
  installed_unit="$1"
  unit_path="$systemd_root/$installed_unit"
  [ -f "$unit_path" ] || fail "missing installed user unit $installed_unit"
  cmp -s "$host_source_dir/systemd/$installed_unit" "$unit_path" ||
    fail "installed user unit $installed_unit differs from repository policy"
}

assert_unit_property() {
  unit_name="$1"
  property_name="$2"
  expected_value="$3"
  actual_value="$(systemctl --user show "$unit_name" --property="$property_name" --value 2>/dev/null || true)"
  [ "$actual_value" = "$expected_value" ] ||
    fail "$unit_name $property_name is '$actual_value', expected '$expected_value'"
}

verify_installation() {
  read_policy_high_water
  [ "$installed_highest_version" -eq "$CODEX_RUST_HOST_POLICY_VERSION" ] ||
    fail "policy high-water mark is v$installed_highest_version, expected v$CODEX_RUST_HOST_POLICY_VERSION"
  assert_installed_file cargo
  assert_installed_file cargo-config.py
  assert_installed_file policy.sh
  assert_installed_file systemd/codex-rust.slice
  assert_installed_file systemd/codex-rust-nnsims.slice
  assert_installed_file systemd/codex-rust-babylon.slice
  [ -x "$install_root/cargo" ] || fail "installed Cargo dispatcher is not executable"
  [ -L "$install_root/real/cargo" ] || fail "installed real-Cargo proxy is not a symlink"
  [ "$(readlink -f -- "$install_root/real/cargo")" = "$(readlink -f -- "$real_cargo_proxy")" ] ||
    fail "installed real-Cargo proxy does not resolve to $real_cargo_proxy"
  [ -L "$cargo_shim" ] || fail "$cargo_shim is not the managed symlink"
  [ "$(readlink -f -- "$cargo_shim")" = "$(readlink -f -- "$install_root/cargo")" ] ||
    fail "$cargo_shim does not resolve to the current dispatcher"
  assert_dispatcher_bin_or_absent
  [ -L "$dispatcher_shim" ] || fail "$dispatcher_shim is not the dedicated Cargo symlink"
  [ "$(readlink -f -- "$dispatcher_shim")" = "$(readlink -f -- "$install_root/cargo")" ] ||
    fail "$dispatcher_shim does not resolve to the current dispatcher"

  if ! command -v systemctl >/dev/null 2>&1 || ! systemctl --user show-environment >/dev/null 2>&1; then
    fail "the user systemd manager is unavailable"
  fi
  for installed_unit in codex-rust.slice codex-rust-nnsims.slice codex-rust-babylon.slice; do
    assert_installed_unit "$installed_unit"
  done
  assert_unit_property codex-rust.slice CPUQuotaPerSecUSec 8s
  assert_unit_property codex-rust.slice MemoryHigh 21474836480
  assert_unit_property codex-rust.slice MemoryMax 25769803776
  assert_unit_property codex-rust.slice MemorySwapMax 4294967296
  assert_unit_property codex-rust.slice IOWeight 500
  assert_unit_property codex-rust.slice TasksMax 1024

  for child_slice in codex-rust-nnsims.slice codex-rust-babylon.slice; do
    assert_unit_property "$child_slice" CPUQuotaPerSecUSec 4s
    assert_unit_property "$child_slice" MemoryHigh 10737418240
    assert_unit_property "$child_slice" MemoryMax 12884901888
    assert_unit_property "$child_slice" MemorySwapMax 2147483648
    assert_unit_property "$child_slice" IOWeight 500
    assert_unit_property "$child_slice" TasksMax 512
  done

  resolved_from_path="$(command -v cargo 2>/dev/null || true)"
  [ -n "$resolved_from_path" ] || fail "Cargo is absent from PATH"
  [ "$(readlink -f -- "$resolved_from_path")" = "$(readlink -f -- "$install_root/cargo")" ] ||
    fail "PATH selects $resolved_from_path instead of the managed Cargo dispatcher"
}

acquire_install_lock

for required_source in cargo cargo-config.py policy.sh \
  systemd/codex-rust.slice \
  systemd/codex-rust-nnsims.slice \
  systemd/codex-rust-babylon.slice; do
  assert_source "$required_source"
done

if [ "$operation" = check ]; then
  verify_installation
  echo "codex Rust host policy v$CODEX_RUST_HOST_POLICY_VERSION is installed"
  exit 0
fi

[ -x "$real_cargo_proxy" ] || fail "rustup Cargo proxy is unavailable at $real_cargo_proxy"
read_policy_high_water
if [ "$(readlink -f -- "$real_cargo_proxy")" = "$(readlink -f -- "$cargo_shim" 2>/dev/null || true)" ]; then
  fail "real Cargo proxy would recurse through the managed shim"
fi
assert_dispatcher_bin_or_absent
assert_managed_shim_or_absent "$cargo_shim"
assert_managed_shim_or_absent "$dispatcher_shim"
assert_complete_install_root_or_absent
for unit_name in codex-rust.slice codex-rust-nnsims.slice codex-rust-babylon.slice; do
  assert_managed_unit_or_absent "$unit_name"
done
if ! command -v systemd-analyze >/dev/null 2>&1; then
  fail "systemd-analyze is required to validate the resource units"
fi
systemd-analyze --user verify "$host_source_dir"/systemd/*.slice

mkdir -p "$local_bin" "$dispatcher_bin" "$systemd_root"
stage_complete_install_root
for unit_name in codex-rust.slice codex-rust-nnsims.slice codex-rust-babylon.slice; do
  install -m 644 "$install_root/systemd/$unit_name" "$systemd_root/$unit_name"
done
ln -sfn "$install_root/cargo" "$cargo_shim"
ln -sfn "$install_root/cargo" "$dispatcher_shim"
hash -r 2>/dev/null || true

systemctl --user daemon-reload
persist_policy_high_water
verify_installation

cache_root="$(codex_rust_cache_root)"
for repository in nnsims babylon; do
  cache_dir="$(codex_rust_cache_dir "$repository" "$cache_root")"
  server_socket="$(codex_rust_server_socket "$repository" "$cache_root")"
  socket_parent="$(dirname -- "$server_socket")"
  codex_rust_create_physical_directory "$cache_dir" ||
    fail "repository cache resolves through a symlink: $cache_dir"
  codex_rust_create_physical_directory "$socket_parent" ||
    fail "repository socket directory resolves through a symlink: $socket_parent"
done

echo "installed codex Rust host policy v$CODEX_RUST_HOST_POLICY_VERSION"
