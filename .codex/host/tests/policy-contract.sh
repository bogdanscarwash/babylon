#!/bin/sh
# Source-only contract for repository recognition and the bounded host policy.
set -eu

unset CARGO_TARGET_DIR CARGO_BUILD_TARGET_DIR CARGO_BUILD_BUILD_DIR
unset CODEX_RUST_SCOPE_ACTIVE CODEX_RUST_CARGO_DISPATCHER
unset CODEX_RUST_HOST_DRY_RUN CODEX_RUST_SCCACHE_BOOTSTRAP

test_path="$(readlink -f -- "$0")"
host_dir="$(dirname -- "$(dirname -- "$test_path")")"
dispatcher="$host_dir/cargo"
temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/nnsims-host-policy.XXXXXX")"
trap 'rm -rf -- "$temporary_root"' EXIT HUP INT TERM

fail() {
  echo "host policy contract: $*" >&2
  exit 1
}

make_repository() {
  repository_path="$1"
  repository_remote="$2"
  mkdir -p "$repository_path"
  git -C "$repository_path" init --quiet
  git -C "$repository_path" remote add origin "$repository_remote"
}

dry_run() {
  repository_path="$1"
  shift
  dry_run_status=0
  if dry_run_output="$({
    cd "$repository_path"
    CODEX_RUST_HOST_DRY_RUN=1 \
      CODEX_RUST_CACHE_ROOT="${CODEX_RUST_CACHE_ROOT:-$temporary_root/cache}" \
      CODEX_RUST_TOOLS_ROOT="${CODEX_RUST_TOOLS_ROOT:-$temporary_root/tools}" \
      "$dispatcher" "$@"
  })"; then
    fail "dispatcher inspection mode returned a false-green status"
  else
    dry_run_status="$?"
  fi
  [ "$dry_run_status" -eq 88 ] || return "$dry_run_status"
  printf '%s\n' "$dry_run_output"
}

assert_line() {
  output="$1"
  expected="$2"
  printf '%s\n' "$output" | grep -Fqx "$expected" ||
    fail "missing '$expected' in:\n$output"
}

nnsims_repository="$temporary_root/nnsims"
babylon_repository="$temporary_root/babylon"
unmanaged_repository="$temporary_root/unmanaged"
make_repository "$nnsims_repository" https://github.com/percy-raskova/nnsims.git
make_repository "$babylon_repository" git@github.com:percy-raskova/babylon.git
make_repository "$unmanaged_repository" https://example.invalid/unmanaged.git
mkdir -p "$babylon_repository/rust"
mkdir -p "$temporary_root/tools/sccache/0.17.0"
printf '%s\n' '#!/bin/sh' 'printf "%s\n" "sccache 0.16.0"' \
  >"$temporary_root/tools/sccache/0.17.0/sccache"
chmod 755 "$temporary_root/tools/sccache/0.17.0/sccache"

for recognized_remote in \
  https://github.com/percy-raskova/nnsims \
  https://github.com/percy-raskova/nnsims.git \
  git@github.com:percy-raskova/nnsims \
  git@github.com:percy-raskova/nnsims.git \
  ssh://git@github.com/percy-raskova/nnsims \
  ssh://git@github.com/percy-raskova/nnsims.git; do
  git -C "$nnsims_repository" remote set-url origin "$recognized_remote"
  recognized_output="$(dry_run "$nnsims_repository" check)"
  assert_line "$recognized_output" repository=nnsims
done
git -C "$nnsims_repository" remote set-url origin https://github.com/percy-raskova/nnsims.git

nnsims_output="$(dry_run "$nnsims_repository" test --release)"
assert_line "$nnsims_output" repository=nnsims
assert_line "$nnsims_output" slice=codex-rust-nnsims.slice
assert_line "$nnsims_output" "cache=$temporary_root/cache/sccache/nnsims/0.17.0-p2"
assert_line "$nnsims_output" "socket=$temporary_root/cache/sccache-server/nnsims-0.17.0-p2.sock"
assert_line "$nnsims_output" "wrapper=$temporary_root/tools/sccache/0.17.0/bin/sccache"
assert_line "$nnsims_output" "target=$nnsims_repository/target"
assert_line "$nnsims_output" jobs=4

babylon_output="$(dry_run "$babylon_repository" check)"
assert_line "$babylon_output" repository=babylon
assert_line "$babylon_output" slice=codex-rust-babylon.slice
assert_line "$babylon_output" "cache=$temporary_root/cache/sccache/babylon/0.17.0-p2"
assert_line "$babylon_output" "socket=$temporary_root/cache/sccache-server/babylon-0.17.0-p2.sock"
assert_line "$babylon_output" "target=$babylon_repository/rust/target"

[ "$(dry_run "$unmanaged_repository" metadata)" = unmanaged ] ||
  fail "an unrelated repository did not fall through"
: >"$unmanaged_repository/Cargo.toml"
[ "$(dry_run "$nnsims_repository" --manifest-path "$unmanaged_repository/Cargo.toml" check)" = unmanaged ] ||
  fail "an unrelated manifest inherited the caller's managed repository"
[ "$(dry_run "$nnsims_repository" -C "$unmanaged_repository" check)" = unmanaged ] ||
  fail "an unrelated effective directory inherited the caller's managed repository"

contaminated_directory_output="$({
  GIT_DIR="$nnsims_repository/.git" \
    GIT_WORK_TREE="$nnsims_repository" \
    dry_run "$babylon_repository" -C "$babylon_repository" check
})"
assert_line "$contaminated_directory_output" repository=babylon
contaminated_config_output="$({
  GIT_CONFIG_COUNT=1 \
    GIT_CONFIG_KEY_0=remote.origin.url \
    GIT_CONFIG_VALUE_0=https://github.com/percy-raskova/nnsims.git \
    dry_run "$babylon_repository" check
})"
assert_line "$contaminated_config_output" repository=babylon
legacy_config_output="$({
  GIT_CONFIG=/dev/null dry_run "$babylon_repository" check
})"
assert_line "$legacy_config_output" repository=babylon

outside_repository="$temporary_root/outside"
mkdir -p "$outside_repository"
: >"$nnsims_repository/Cargo.toml"
manifest_output="$(dry_run "$outside_repository" --manifest-path "$nnsims_repository/Cargo.toml" check)"
assert_line "$manifest_output" repository=nnsims
assert_line "$manifest_output" "target=$nnsims_repository/target"
directory_output="$(dry_run "$outside_repository" -C "$babylon_repository" check)"
assert_line "$directory_output" repository=babylon
assert_line "$directory_output" "target=$babylon_repository/rust/target"

managed_manifest_link="$outside_repository/nnsims-Cargo.toml"
ln -s "$nnsims_repository/Cargo.toml" "$managed_manifest_link"
linked_manifest_output="$(dry_run "$outside_repository" --manifest-path "$managed_manifest_link" check)"
assert_line "$linked_manifest_output" repository=nnsims
assert_line "$linked_manifest_output" "target=$nnsims_repository/target"
unmanaged_manifest_link="$nnsims_repository/unmanaged-Cargo.toml"
ln -s "$unmanaged_repository/Cargo.toml" "$unmanaged_manifest_link"
[ "$(dry_run "$nnsims_repository" --manifest-path "$unmanaged_manifest_link" check)" = unmanaged ] ||
  fail "a symlinked unrelated manifest inherited the caller's managed repository"

shared_target="$temporary_root/shared-target"
mkdir -p "$shared_target"
ln -s "$shared_target" "$nnsims_repository/target"
if dry_run "$nnsims_repository" check >/dev/null 2>&1; then
  fail "a symlinked NNSims target escaped the per-worktree boundary"
fi
rm -f -- "$nnsims_repository/target"
ln -s "$shared_target" "$babylon_repository/rust/target"
if dry_run "$babylon_repository" check >/dev/null 2>&1; then
  fail "a symlinked Babylon target escaped the per-worktree boundary"
fi
rm -f -- "$babylon_repository/rust/target"

mkdir -p "$nnsims_repository/.cargo"
printf '%s\n' '[alias]' 'bounded = "check -j4"' \
  >"$nnsims_repository/.cargo/config.toml"
dry_run "$nnsims_repository" bounded >/dev/null
printf '%s\n' '[alias]' 'over = "check --jobs 32"' \
  >"$nnsims_repository/.cargo/config.toml"
if dry_run "$nnsims_repository" over >/dev/null 2>&1; then
  fail "a Cargo config alias escaped the four-job ceiling"
fi
printf '%s\n' '[alias]' \
  "other = \"check --manifest-path $babylon_repository/Cargo.toml\"" \
  >"$nnsims_repository/.cargo/config.toml"
if dry_run "$nnsims_repository" other >/dev/null 2>&1; then
  fail "a Cargo config alias selected another repository manifest"
fi
rm -f -- "$nnsims_repository/.cargo/config.toml"
if CARGO_ALIAS_OVER='check --jobs 32' \
  dry_run "$nnsims_repository" over >/dev/null 2>&1; then
  fail "an environment Cargo alias escaped the four-job ceiling"
fi
if CARGO_ALIAS_OTHER="check -C $babylon_repository" \
  dry_run "$nnsims_repository" other >/dev/null 2>&1; then
  fail "an environment Cargo alias selected another repository directory"
fi

nnsims_two_jobs="$(CARGO_BUILD_JOBS=2 dry_run "$nnsims_repository" check)"
assert_line "$nnsims_two_jobs" jobs=2

if dry_run "$nnsims_repository" test -j5 >/dev/null 2>&1; then
  fail "-j5 escaped the four-job ceiling"
fi
if dry_run "$nnsims_repository" test --jobs 5 >/dev/null 2>&1; then
  fail "--jobs 5 escaped the four-job ceiling"
fi
if CARGO_BUILD_JOBS=5 dry_run "$nnsims_repository" check >/dev/null 2>&1; then
  fail "CARGO_BUILD_JOBS=5 escaped the four-job ceiling"
fi
dry_run "$nnsims_repository" test -j4 >/dev/null
if dry_run "$nnsims_repository" test --config build.jobs=5 >/dev/null 2>&1; then
  fail "caller-supplied --config escaped the managed Cargo boundary"
fi
dry_run "$nnsims_repository" test -- --config build.jobs=5 >/dev/null
if dry_run "$nnsims_repository" test --target-dir "$temporary_root/shared-target" >/dev/null 2>&1; then
  fail "caller-supplied --target-dir value escaped the per-worktree target boundary"
fi
if dry_run "$nnsims_repository" test --target-dir="$temporary_root/shared-target" >/dev/null 2>&1; then
  fail "caller-supplied --target-dir=value escaped the per-worktree target boundary"
fi
dry_run "$nnsims_repository" test -- --target-dir "$temporary_root/child-argument" >/dev/null
for prohibited_target_environment in \
  CARGO_TARGET_DIR \
  CARGO_BUILD_TARGET_DIR \
  CARGO_BUILD_BUILD_DIR; do
  if (
    export "$prohibited_target_environment=$temporary_root/shared-target"
    dry_run "$nnsims_repository" check
  ) >/dev/null 2>&1; then
    fail "$prohibited_target_environment escaped the per-worktree target boundary"
  fi
done

long_cache_root="$temporary_root/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
if CODEX_RUST_CACHE_ROOT="$long_cache_root" \
  dry_run "$nnsims_repository" check >/dev/null 2>&1; then
  fail "an overlong sccache Unix socket path was accepted"
fi

dry_run_false_green_status=0
if (
  cd "$nnsims_repository"
  CODEX_RUST_HOST_DRY_RUN=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$temporary_root/tools" \
    "$dispatcher" check >/dev/null
); then
  fail "an inherited dry-run flag made a managed Cargo gate exit zero"
else
  dry_run_false_green_status="$?"
fi
[ "$dry_run_false_green_status" -eq 88 ] ||
  fail "dispatcher inspection mode exited $dry_run_false_green_status instead of 88"

# Exercise the already-scoped path without invoking Cargo or systemd. The fake
# real-Cargo proxy observes the exact process boundary after exec.
managed_host="$temporary_root/managed-host"
managed_dispatcher="$managed_host/cargo"
managed_result="$temporary_root/managed-result"
managed_tools="$temporary_root/managed-tools"
mkdir -p "$managed_host/real" "$managed_tools/sccache/0.17.0/bin"
install -m 755 "$dispatcher" "$managed_dispatcher"
install -m 644 "$host_dir/cargo-config.py" "$managed_host/cargo-config.py"
install -m 644 "$host_dir/policy.sh" "$managed_host/policy.sh"
printf '%s\n' \
  '#!/bin/sh' \
  'case "${1:-}" in' \
  '  --version) printf "%s\n" "sccache 0.17.0" ;;' \
  '  --show-stats)' \
  '    printf "cache=%s\nsocket=%s\n" "${SCCACHE_DIR:-}" "${SCCACHE_SERVER_UDS:-}"' \
  '    ;;' \
  '  *) exit 70 ;;' \
  'esac' \
  >"$managed_tools/sccache/0.17.0/bin/sccache"
chmod 755 "$managed_tools/sccache/0.17.0/bin/sccache"
printf '%s\n' \
  '#!/bin/sh' \
  'set -eu' \
  '{' \
  '  printf "cwd=%s\n" "$PWD"' \
  '  printf "jobs=%s\n" "${CARGO_BUILD_JOBS:-unset}"' \
  '  printf "target=%s\n" "${CARGO_TARGET_DIR:-unset}"' \
  '  printf "build_dir=%s\n" "${CARGO_BUILD_BUILD_DIR:-unset}"' \
  '  printf "rustc_wrapper=%s\n" "${RUSTC_WRAPPER+set}"' \
  '  printf "workspace_wrapper=%s\n" "${RUSTC_WORKSPACE_WRAPPER+set}"' \
  '  printf "cargo_wrapper=%s\n" "${CARGO_BUILD_RUSTC_WRAPPER+set}"' \
  '  printf "cargo_workspace_wrapper=%s\n" "${CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER+set}"' \
  '  printf "managed_dispatcher=%s\n" "${CODEX_RUST_CARGO_DISPATCHER:-unset}"' \
  '  printf "cache=%s\n" "${SCCACHE_DIR:-unset}"' \
  '  printf "socket=%s\n" "${SCCACHE_SERVER_UDS:-unset}"' \
  '  printf "cache_size=%s\n" "${SCCACHE_CACHE_SIZE:-unset}"' \
  '  for observed_argument in "$@"; do printf "<%s>\n" "$observed_argument"; done' \
  '} >"$HOST_CONTRACT_RESULT"' \
  'if [ "${HOST_CONTRACT_REENTER:-0}" = 1 ]; then' \
  '  reentry_status=0' \
  '  CODEX_RUST_HOST_DRY_RUN=1 "$CODEX_RUST_CARGO_DISPATCHER" check >"$HOST_CONTRACT_REENTRY_RESULT" || reentry_status="$?"' \
  '  [ "$reentry_status" -eq 88 ] || exit "$reentry_status"' \
  'fi' \
  'if [ "${HOST_CONTRACT_SIGNAL:-}" = TERM ]; then kill -TERM "$$"; fi' \
  'exit "${HOST_CONTRACT_EXIT:-0}"' \
  >"$managed_host/real/cargo"
chmod 755 "$managed_host/real/cargo"

# A caller-exported recursion flag is not evidence of the selected repository's
# scope membership. This contract itself runs in NNSims's slice, so selecting
# Babylon proves the exact dispatcher refuses the mismatched real cgroup before
# the fixture substitutes an NNSims membership record for the positive path.
if (
  cd "$babylon_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" check
) >/dev/null 2>&1; then
  fail "a stale scope-active flag bypassed the selected repository slice"
fi
managed_cgroup="$temporary_root/managed-cgroup"
printf '0::/user.slice/codex-rust.slice/codex-rust-nnsims.slice/run-test.scope\n' \
  >"$managed_cgroup"
grep -Fqx 'cgroup_membership_file=/proc/self/cgroup' "$managed_dispatcher" ||
  fail "the dispatcher does not name the kernel cgroup membership file"
sed -i \
  "s|cgroup_membership_file=/proc/self/cgroup|cgroup_membership_file=$managed_cgroup|" \
  "$managed_dispatcher"

if (
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_SCCACHE_BOOTSTRAP=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" check
) >/dev/null 2>&1; then
  fail "an inherited sccache-bootstrap flag disabled the wrapper for an ordinary command"
fi
if (
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_SCCACHE_BOOTSTRAP=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" install --locked --force --version =0.17.0 \
      --root "$managed_tools/sccache/not-0.17.0" sccache
) >/dev/null 2>&1; then
  fail "a malformed sccache-bootstrap command disabled the wrapper"
fi
(
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_SCCACHE_BOOTSTRAP=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" install --locked --force --version =0.17.0 \
      --root "$managed_tools/sccache/0.17.0" sccache
)
grep -Fqx '<build.rustc-wrapper="">' "$managed_result" ||
  fail "the exact pinned sccache bootstrap did not disable its self-wrapper"
grep -Fqx '<install>' "$managed_result" ||
  fail "the exact pinned sccache bootstrap did not reach Cargo"

printf '0::/user.slice/codex-rust.slice/codex-rust-babylon.slice/run-test.scope\n' \
  >"$managed_cgroup"
if (
  cd "$babylon_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_SCCACHE_BOOTSTRAP=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" +1.97.1 install --locked --force --version =0.17.0 \
      --root "$managed_tools/sccache/0.17.0" sccache
) >/dev/null 2>&1; then
  fail "Babylon accepted a bootstrap toolchain other than its pinned +1.91.1"
fi
(
  cd "$babylon_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_SCCACHE_BOOTSTRAP=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" +1.91.1 install --locked --force --version =0.17.0 \
      --root "$managed_tools/sccache/0.17.0" sccache
)
grep -Fqx '<+1.91.1>' "$managed_result" ||
  fail "Babylon's exact pinned bootstrap selector did not reach Cargo"
grep -Fqx '<build.rustc-wrapper="">' "$managed_result" ||
  fail "Babylon's exact pinned bootstrap did not disable its self-wrapper"
printf '0::/user.slice/codex-rust.slice/codex-rust-nnsims.slice/run-test.scope\n' \
  >"$managed_cgroup"

(
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    RUSTC_WRAPPER=stale \
    RUSTC_WORKSPACE_WRAPPER=stale \
    CARGO_BUILD_RUSTC_WRAPPER=stale \
    CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=stale \
    "$managed_dispatcher" +1.91.1 check "argument with spaces" -- -j99
)
grep -Fqx "cwd=$nnsims_repository" "$managed_result" ||
  fail "the dispatcher changed the caller's working directory"
grep -Fqx 'jobs=4' "$managed_result" ||
  fail "the dispatcher did not pin the inner Cargo job count"
grep -Fqx "target=$nnsims_repository/target" "$managed_result" ||
  fail "the dispatcher did not pin the worktree-local target directory"
grep -Fqx "build_dir=$nnsims_repository/target" "$managed_result" ||
  fail "the dispatcher did not pin Cargo's build directory to the worktree-local target"
grep -Fqx "managed_dispatcher=$managed_dispatcher" "$managed_result" ||
  fail "the dispatcher did not publish its path for nested Cargo tools"
for cleared_wrapper in \
  rustc_wrapper \
  workspace_wrapper \
  cargo_wrapper \
  cargo_workspace_wrapper; do
  grep -Fqx "$cleared_wrapper=" "$managed_result" ||
    fail "the dispatcher preserved inherited $cleared_wrapper state"
done
grep -Fqx "<build.rustc-wrapper=\"$managed_tools/sccache/0.17.0/bin/sccache\">" "$managed_result" ||
  fail "the dispatcher did not pin the compiler wrapper"
grep -Fqx '<env.SCCACHE_DIR="'"$temporary_root"'/cache/sccache/nnsims/0.17.0-p2">' "$managed_result" ||
  fail "the dispatcher did not override the NNSims cache directory"
grep -Fqx '<env.SCCACHE_SERVER_UDS="'"$temporary_root"'/cache/sccache-server/nnsims-0.17.0-p2.sock">' "$managed_result" ||
  fail "the dispatcher did not override the NNSims server socket"
grep -Fqx '<env.SCCACHE_CACHE_SIZE="10G">' "$managed_result" ||
  fail "the dispatcher did not override the cache-size ceiling"
grep -Fqx '<env.SCCACHE_CLIENT_SIDE="0">' "$managed_result" ||
  fail "the dispatcher did not keep client-side mode disabled"
first_forwarded_argument="$(grep '^<' "$managed_result" | sed -n '1p')"
[ "$first_forwarded_argument" = '<+1.91.1>' ] ||
  fail "the rustup toolchain selector was not Cargo's leading argument"
grep -Fqx 'cache=unset' "$managed_result" ||
  fail "the dispatcher preserved an inherited cache directory"
grep -Fqx 'socket=unset' "$managed_result" ||
  fail "the dispatcher preserved an inherited server socket"
grep -Fqx 'cache_size=unset' "$managed_result" ||
  fail "the dispatcher preserved an inherited cache-size setting"
grep -Fqx '<check>' "$managed_result" ||
  fail "the dispatcher lost the Cargo subcommand"
grep -Fqx '<argument with spaces>' "$managed_result" ||
  fail "the dispatcher changed an argument containing spaces"
grep -Fqx '<-j99>' "$managed_result" ||
  fail "the dispatcher interpreted a child argument after --"

managed_reentry_result="$temporary_root/managed-reentry-result"
(
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    HOST_CONTRACT_REENTER=1 \
    HOST_CONTRACT_REENTRY_RESULT="$managed_reentry_result" \
    "$managed_dispatcher" check
)
assert_line "$(sed -n '1p' "$managed_reentry_result")" repository=nnsims
grep -Fqx "target=$nnsims_repository/target" "$managed_reentry_result" ||
  fail "nested Cargo could not re-enter with the dispatcher's own target environment"

if (
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" --manifest-path "$unmanaged_repository/Cargo.toml" check
) >/dev/null 2>&1; then
  fail "nested managed Cargo carried its target and cache into an unmanaged manifest"
fi
if (
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" -C "$unmanaged_repository" check
) >/dev/null 2>&1; then
  fail "nested managed Cargo carried its target and cache into an unmanaged directory"
fi

symlink_cache="$temporary_root/symlink-cache"
mkdir -p "$symlink_cache/sccache" "$symlink_cache/shared-cache" \
  "$symlink_cache/sccache-server"
ln -s "$symlink_cache/shared-cache" "$symlink_cache/sccache/nnsims"
if (
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_CACHE_ROOT="$symlink_cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" check
) >/dev/null 2>&1; then
  fail "a symlinked compiler-cache namespace crossed the repository boundary"
fi

(
  cd "$unmanaged_repository"
  HOST_CONTRACT_RESULT="$managed_result" \
    CARGO_BUILD_JOBS=9 \
    RUSTC_WRAPPER=preserved \
    "$managed_dispatcher" --config build.jobs=9 check "unmanaged argument"
)
grep -Fqx "cwd=$unmanaged_repository" "$managed_result" ||
  fail "unmanaged Cargo changed its working directory"
grep -Fqx 'jobs=9' "$managed_result" ||
  fail "the dispatcher changed an unmanaged job request"
grep -Fqx 'rustc_wrapper=set' "$managed_result" ||
  fail "the dispatcher changed an unmanaged wrapper environment"
grep -Fqx 'managed_dispatcher=unset' "$managed_result" ||
  fail "the dispatcher injected managed state into an unrelated repository"
grep -Fqx '<--config>' "$managed_result" ||
  fail "the dispatcher consumed unmanaged Cargo configuration"
grep -Fqx '<unmanaged argument>' "$managed_result" ||
  fail "the dispatcher changed an unmanaged argument"

if (
  cd "$nnsims_repository"
  CODEX_RUST_SCOPE_ACTIVE=1 \
    CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    HOST_CONTRACT_EXIT=37 \
    "$managed_dispatcher" check
) >/dev/null 2>&1; then
  fail "the dispatcher hid the real Cargo proxy's failure status"
else
  managed_status="$?"
fi
[ "$managed_status" -eq 37 ] ||
  fail "the dispatcher changed exit 37 to $managed_status"

managed_signal_status="$(
  {
    set +e
    (
      cd "$nnsims_repository"
      CODEX_RUST_SCOPE_ACTIVE=1 \
        CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
        CODEX_RUST_TOOLS_ROOT="$managed_tools" \
        HOST_CONTRACT_RESULT="$managed_result" \
        HOST_CONTRACT_SIGNAL=TERM \
        "$managed_dispatcher" check
    ) >/dev/null 2>&1
    printf '%s\n' "$?"
  } 2>/dev/null
)"
[ "$managed_signal_status" -eq 143 ] ||
  fail "the dispatcher changed SIGTERM into status $managed_signal_status"

worktree_environment="$(dirname -- "$host_dir")/scripts/worktree-env.sh"
expected_compose_suffix="$(printf '%s' "$nnsims_repository" | sha256sum | cut -c 1-12)"
worktree_output="$(
  cd "$nnsims_repository"
  CODEX_WORKTREE_PATH="$nnsims_repository" sh -c \
    '. "$1"; printf "project=%s\nport=%s\n" "$COMPOSE_PROJECT_NAME" "$POSTGRES_HOST_PORT"' \
    sh "$worktree_environment"
)"
assert_line "$worktree_output" "project=nnsims_codex_$expected_compose_suffix"
assert_line "$worktree_output" 'port=0'
contaminated_worktree_output="$(
  cd "$nnsims_repository"
  GIT_DIR="$babylon_repository/.git" \
    GIT_WORK_TREE="$babylon_repository" \
    sh -c '. "$1"; printf "project=%s\n" "$COMPOSE_PROJECT_NAME"' \
      sh "$worktree_environment"
)"
assert_line "$contaminated_worktree_output" "project=nnsims_codex_$expected_compose_suffix"
if (
  cd "$nnsims_repository"
  CODEX_WORKTREE_PATH="$babylon_repository" sh -c '. "$1"' sh "$worktree_environment"
) >/dev/null 2>&1; then
  fail "the action environment accepted a stale worktree identity"
fi

cache_stats="$({
  CODEX_RUST_HOST_DRY_RUN=1 \
  CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    "$(dirname -- "$host_dir")/scripts/sccache-stats.sh"
})"
assert_line "$cache_stats" "cache=$temporary_root/cache/sccache/nnsims/0.17.0-p2"
assert_line "$cache_stats" "socket=$temporary_root/cache/sccache-server/nnsims-0.17.0-p2.sock"
assert_line "$cache_stats" 'slice=codex-rust-nnsims.slice'

grep -Fqx 'MemoryMax=24G' "$host_dir/systemd/codex-rust.slice" ||
  fail "the parent memory ceiling moved"
grep -Fqx 'MemoryMax=12G' "$host_dir/systemd/codex-rust-nnsims.slice" ||
  fail "the NNSims memory ceiling moved"
grep -Fqx 'MemoryMax=12G' "$host_dir/systemd/codex-rust-babylon.slice" ||
  fail "the Babylon memory ceiling moved"
grep -Fq 'CODEX_RUST_SCOPE_ACTIVE' "$dispatcher" ||
  fail "the nested-Cargo recursion guard disappeared"
grep -Fq 'codex_rust_process_is_in_slice "$slice" "$cgroup_membership_file"' "$dispatcher" ||
  fail "the recursion guard no longer validates the current repository slice"
grep -Fq 'build.rustc-wrapper' "$dispatcher" ||
  fail "the dispatcher no longer pins the compiler wrapper"
grep -Fq 'refusing an unbounded build' "$dispatcher" ||
  fail "recognized repositories no longer fail closed"

fake_systemctl="$temporary_root/systemctl"
printf '%s\n' \
  '#!/bin/sh' \
  'case "$*" in' \
  '  *--property=FragmentPath*) exit 0 ;;' \
  '  *) printf "%s\n" infinity ;;' \
  'esac' \
  >"$fake_systemctl"
chmod 755 "$fake_systemctl"
sed -i "s|/usr/bin/systemctl|$fake_systemctl|g" "$managed_host/policy.sh"
if (
  cd "$nnsims_repository"
  CODEX_RUST_CACHE_ROOT="$temporary_root/cache" \
    CODEX_RUST_TOOLS_ROOT="$managed_tools" \
    HOST_CONTRACT_RESULT="$managed_result" \
    "$managed_dispatcher" check
) >/dev/null 2>&1; then
  fail "a missing slice unit was admitted as an unbounded implicit slice"
fi

echo "host policy contract: PASS"
