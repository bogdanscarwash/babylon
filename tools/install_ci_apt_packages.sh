#!/usr/bin/env bash
# Install hosted-runner apt packages with a fixed retry and timeout envelope.

set -euo pipefail

readonly MAX_ATTEMPTS=3
readonly MAX_APT_TIMEOUT_SECONDS=300
readonly MAX_RETRY_DELAY_SECONDS=30
readonly APT_TIMEOUT_SECONDS="${BABYLON_CI_APT_TIMEOUT_SECONDS:-120}"
readonly RETRY_DELAY_SECONDS="${BABYLON_CI_APT_RETRY_DELAY_SECONDS:-10}"

case "$APT_TIMEOUT_SECONDS" in
  "" | 0* | *[!0-9]*)
    echo "install_ci_apt_packages: BABYLON_CI_APT_TIMEOUT_SECONDS must be a canonical positive integer (no leading zeros)" >&2
    exit 2
    ;;
esac
if (( ${#APT_TIMEOUT_SECONDS} > ${#MAX_APT_TIMEOUT_SECONDS} )) || \
  (( APT_TIMEOUT_SECONDS > MAX_APT_TIMEOUT_SECONDS )); then
  echo "install_ci_apt_packages: BABYLON_CI_APT_TIMEOUT_SECONDS cannot exceed $MAX_APT_TIMEOUT_SECONDS" >&2
  exit 2
fi

case "$RETRY_DELAY_SECONDS" in
  "" | 0[0-9]* | *[!0-9]*)
    echo "install_ci_apt_packages: BABYLON_CI_APT_RETRY_DELAY_SECONDS must be a canonical nonnegative integer (no leading zeros)" >&2
    exit 2
    ;;
esac
if (( ${#RETRY_DELAY_SECONDS} > ${#MAX_RETRY_DELAY_SECONDS} )) || \
  (( RETRY_DELAY_SECONDS > MAX_RETRY_DELAY_SECONDS )); then
  echo "install_ci_apt_packages: BABYLON_CI_APT_RETRY_DELAY_SECONDS cannot exceed $MAX_RETRY_DELAY_SECONDS" >&2
  exit 2
fi

if (( $# == 0 )); then
  echo "usage: install_ci_apt_packages.sh PACKAGE [PACKAGE ...]" >&2
  exit 2
fi

run_apt_transaction() {
  # The timed child, not this parent shell, must expand status and package arguments.
  # shellcheck disable=SC2016
  timeout --signal=TERM --kill-after=10s "${APT_TIMEOUT_SECONDS}s" \
    bash -c '
      set -euo pipefail
      sudo -n env DEBIAN_FRONTEND=noninteractive apt-get update || {
        status=$?
        echo "install_ci_apt_packages: apt-get update failed (exit $status)" >&2
        exit "$status"
      }
      sudo -n env DEBIAN_FRONTEND=noninteractive apt-get \
        install -y --no-install-recommends "$@" || {
        status=$?
        echo "install_ci_apt_packages: apt-get install failed (exit $status)" >&2
        exit "$status"
      }
    ' install_ci_apt_packages "$@"
}

for attempt in 1 2 3; do
  status=0
  if run_apt_transaction "$@"; then
    echo "install_ci_apt_packages: apt transaction succeeded on attempt $attempt"
    exit 0
  else
    status=$?
  fi

  echo "install_ci_apt_packages: attempt $attempt of $MAX_ATTEMPTS failed (exit $status)" >&2
  if (( attempt < MAX_ATTEMPTS && RETRY_DELAY_SECONDS > 0 )); then
    sleep "$RETRY_DELAY_SECONDS"
  fi
done

echo "install_ci_apt_packages: failed after $MAX_ATTEMPTS attempts" >&2
exit 1
