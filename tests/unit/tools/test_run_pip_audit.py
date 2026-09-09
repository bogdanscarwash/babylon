"""Behavioral contract for the pip-audit policy wrapper (Program 14/15 Phase 3).

The wrapper (``tools/run_pip_audit.py``) enforces, loudly (Constitution
III.11): every ``[[ignore]]`` entry needs a non-empty ``id`` and ``reason``
and a parseable ISO ``expires`` date, and any entry whose ``expires`` date
has passed HARD-FAILS the run (exit 2) — ignores can never rot silently.

The synthetic-input tests are the red-phase/mutation proof that each check
actually detects its violation class; the real-file test pins that the
shipped ``security/pip-audit-ignores.toml`` itself satisfies the policy.
"""

from __future__ import annotations

import sys
from datetime import date
from pathlib import Path

import pytest

# Mirror the import path used by tools/*.py and its existing unit tests
# (see tests/unit/tools/test_repo_hygiene.py).
TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from run_pip_audit import (  # type: ignore[import-not-found]  # noqa: E402
    DEFAULT_IGNORES_FILE,
    build_ignore_vuln_args,
    build_pip_audit_command,
    find_expired_entries,
    get_ignore_entries,
    load_ignores_file,
    main,
    validate_entries,
)

VALID_TOML = """
[[ignore]]
id = "CVE-2026-3219"
reason = "torch/transformers dependency chain — no fixed release yet"
expires = "2099-01-01"

[[ignore]]
id = "GHSA-aaaa-bbbb-cccc"
reason = "false positive — dev-only extra, never shipped"
expires = "2099-06-30"
"""

EMPTY_TOML = """
# no [[ignore]] entries at all
"""

MISSING_REASON_TOML = """
[[ignore]]
id = "CVE-2026-9999"
expires = "2099-01-01"
"""

BLANK_REASON_TOML = """
[[ignore]]
id = "CVE-2026-9999"
reason = "   "
expires = "2099-01-01"
"""

MISSING_EXPIRES_TOML = """
[[ignore]]
id = "CVE-2026-9999"
reason = "some reason"
"""

GARBLED_EXPIRES_TOML = """
[[ignore]]
id = "CVE-2026-9999"
reason = "some reason"
expires = "not-a-date"
"""

EXPIRED_TOML = """
[[ignore]]
id = "CVE-2000-0001"
reason = "ancient, long fixed, entry deliberately left expired for the test"
expires = "2000-01-01"
"""


@pytest.mark.unit
class TestPolicyParsing:
    """Parsing + arg-building over synthetic policy files."""

    def test_valid_file_produces_ignore_vuln_args(self, tmp_path: Path) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(VALID_TOML)

        policy = load_ignores_file(policy_path)
        entries = get_ignore_entries(policy)

        assert validate_entries(entries) == []
        assert build_ignore_vuln_args(entries) == [
            "--ignore-vuln",
            "CVE-2026-3219",
            "--ignore-vuln",
            "GHSA-aaaa-bbbb-cccc",
        ]

    def test_command_uses_uv_by_default(self, tmp_path: Path) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(VALID_TOML)
        entries = get_ignore_entries(load_ignores_file(policy_path))

        command = build_pip_audit_command(entries, no_uv=False)

        assert command == [
            "uv",
            "run",
            "--frozen",
            "--extra",
            "ops",
            "pip-audit",
            "--ignore-vuln",
            "CVE-2026-3219",
            "--ignore-vuln",
            "GHSA-aaaa-bbbb-cccc",
        ]

    def test_command_respects_no_uv(self, tmp_path: Path) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(VALID_TOML)
        entries = get_ignore_entries(load_ignores_file(policy_path))

        command = build_pip_audit_command(entries, no_uv=True)

        assert command[:2] == ["pip-audit", "--ignore-vuln"]
        assert "uv" not in command

    def test_empty_ignores_file_is_valid_with_zero_flags(self, tmp_path: Path) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(EMPTY_TOML)

        entries = get_ignore_entries(load_ignores_file(policy_path))

        assert entries == []
        assert validate_entries(entries) == []
        assert build_ignore_vuln_args(entries) == []


@pytest.mark.unit
class TestSyntheticViolations:
    """Each validation rule must catch a planted violation (detection proof)."""

    def test_missing_reason_key_detected(self) -> None:
        entries = [{"id": "CVE-2026-9999", "expires": "2099-01-01"}]
        errors = validate_entries(entries)
        assert len(errors) == 1
        assert "CVE-2026-9999" in errors[0]
        assert "reason" in errors[0]

    def test_blank_reason_detected(self) -> None:
        entries = [{"id": "CVE-2026-9999", "reason": "   ", "expires": "2099-01-01"}]
        errors = validate_entries(entries)
        assert len(errors) == 1
        assert "reason" in errors[0]

    def test_missing_id_detected(self) -> None:
        entries = [{"reason": "some reason", "expires": "2099-01-01"}]
        errors = validate_entries(entries)
        assert len(errors) == 1
        assert "id" in errors[0]

    def test_missing_expires_detected(self) -> None:
        entries = [{"id": "CVE-2026-9999", "reason": "some reason"}]
        errors = validate_entries(entries)
        assert len(errors) == 1
        assert "CVE-2026-9999" in errors[0]
        assert "expires" in errors[0]

    def test_garbled_expires_detected(self) -> None:
        entries = [{"id": "CVE-2026-9999", "reason": "some reason", "expires": "not-a-date"}]
        errors = validate_entries(entries)
        assert len(errors) == 1
        assert "CVE-2026-9999" in errors[0]
        assert "expires" in errors[0]

    def test_valid_entry_passes(self) -> None:
        entries = [{"id": "CVE-2026-9999", "reason": "some reason", "expires": "2099-01-01"}]
        assert validate_entries(entries) == []

    def test_expired_entry_detected(self) -> None:
        entries = [
            {"id": "CVE-2000-0001", "reason": "old", "expires": "2000-01-01"},
            {"id": "CVE-2099-0002", "reason": "future", "expires": "2099-01-01"},
        ]
        expired = find_expired_entries(entries, today=date(2026, 7, 11))
        assert [entry["id"] for entry in expired] == ["CVE-2000-0001"]

    def test_entry_expiring_today_is_not_expired(self) -> None:
        entries = [{"id": "CVE-2026-1234", "reason": "r", "expires": "2026-07-11"}]
        expired = find_expired_entries(entries, today=date(2026, 7, 11))
        assert expired == []


@pytest.mark.unit
class TestMainExitCodes:
    """End-to-end exit codes via main(), without ever invoking real pip-audit."""

    def test_missing_reason_exits_2(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(MISSING_REASON_TOML)

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        assert exit_code == 2
        assert "reason" in capsys.readouterr().err

    def test_blank_reason_exits_2(self, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(BLANK_REASON_TOML)

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        assert exit_code == 2
        assert "reason" in capsys.readouterr().err

    def test_missing_expires_exits_2(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(MISSING_EXPIRES_TOML)

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        assert exit_code == 2
        assert "expires" in capsys.readouterr().err

    def test_garbled_expires_exits_2(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(GARBLED_EXPIRES_TOML)

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        assert exit_code == 2
        assert "expires" in capsys.readouterr().err

    def test_expired_entry_exits_2_naming_id(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(EXPIRED_TOML)

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        stderr = capsys.readouterr().err
        assert exit_code == 2
        assert "CVE-2000-0001" in stderr

    def test_check_only_success_path(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(VALID_TOML)

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        assert exit_code == 0
        assert "OK" in capsys.readouterr().out

    def test_check_only_on_empty_ignores_succeeds(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text(EMPTY_TOML)

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        assert exit_code == 0

    def test_missing_file_exits_2(self, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
        missing_path = tmp_path / "does-not-exist.toml"

        exit_code = main(["--ignores-file", str(missing_path), "--check-only"])

        assert exit_code == 2
        assert "not found" in capsys.readouterr().err

    def test_garbled_toml_exits_2(self, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
        policy_path = tmp_path / "ignores.toml"
        policy_path.write_text("this is [not valid toml")

        exit_code = main(["--ignores-file", str(policy_path), "--check-only"])

        assert exit_code == 2
        assert "TOML" in capsys.readouterr().err


@pytest.mark.unit
class TestRealPolicyFile:
    """The shipped policy file itself satisfies the contract."""

    def test_default_ignores_file_exists(self) -> None:
        assert DEFAULT_IGNORES_FILE.is_file()

    def test_real_policy_passes_check_only(self) -> None:
        assert main(["--check-only"]) == 0

    def test_real_policy_pins_the_emptied_policy_exactly(self) -> None:
        """The shipped policy carries no ignores at all.

        The item-41 residue stayed cleared (dependabot-wave-20260711), and the
        one later entry — CVE-2026-11332 (ansible-core 2.20.3, dev tooling
        only, time-boxed to 2026-08-15) — was removed 2026-08-17 because its
        own removal condition landed: uv.lock carries ansible-core 2.21.2
        (stable, >= the 2.20.7 fix line). Any ignore appearing without review
        must fail this pin.
        """
        entries = get_ignore_entries(load_ignores_file(DEFAULT_IGNORES_FILE))
        assert entries == []
