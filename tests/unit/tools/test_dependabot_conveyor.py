"""Behavior contracts for the serial Dependabot merge conveyor."""

from __future__ import annotations

import subprocess

import pytest
import tools.dependabot_conveyor as conveyor

HEAD_SHA = "a" * 40
MOVED_SHA = "b" * 40

MINOR_MESSAGE = (
    "Bump pillow from 10.0.0 to 10.1.0\n\n"
    "Bumps [pillow](https://github.com/python-pillow/Pillow) "
    "from 10.0.0 to 10.1.0.\n\n"
    "---\nupdated-dependencies:\n"
    "- dependency-name: pillow\n"
    "  update-type: version-update:semver-minor\n"
    "...\n"
)

MAJOR_MESSAGE = (
    "Bump actions/checkout from 4 to 5\n\n"
    "Bumps [actions/checkout](https://github.com/actions/checkout) "
    "from 4 to 5.\n\n"
    "---\nupdated-dependencies:\n"
    "- dependency-name: actions/checkout\n"
    "  update-type: version-update:semver-major\n"
    "...\n"
)

CARGO_ZERO_X_MESSAGE = (
    "Bump serde from 0.9.0 to 0.10.0\n\n"
    "Bumps [serde](https://github.com/serde-rs/serde) "
    "from 0.9.0 to 0.10.0.\n\n"
    "---\nupdated-dependencies:\n"
    "- dependency-name: serde\n"
    "  update-type: version-update:semver-minor\n"
    "...\n"
)


CARGO_HEAD_REF = "dependabot/cargo/rust/serde-0.10.0"


def _open_pr(
    number: int,
    state: str = "CLEAN",
    mergeable: str = "MERGEABLE",
    head_oid: str = HEAD_SHA,
    head_ref: str = "dependabot/uv/pillow-10.1.0",
) -> conveyor.OpenPr:
    return conveyor.OpenPr(
        number=number,
        state=state,
        mergeable=mergeable,
        head_oid=head_oid,
        head_ref=head_ref,
    )


def _completed(
    args: object,
    returncode: int,
    stdout: str = "",
    stderr: str = "",
) -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess(
        args=args, returncode=returncode, stdout=stdout, stderr=stderr
    )


# ---------------------------------------------------------------------------
# Bounded subprocess timeouts
# ---------------------------------------------------------------------------


def test_gh_json_uses_bounded_timeout(monkeypatch: pytest.MonkeyPatch) -> None:
    observed: list[object] = []

    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        observed.append(kwargs.get("timeout"))
        return _completed(args[0], 0, stdout="[]")

    monkeypatch.setattr(subprocess, "run", fake_run)
    assert conveyor.gh_json(["pr", "list"]) == []
    assert observed == [conveyor.GH_TIMEOUT_SECONDS]


def test_resolve_repo_uses_bounded_timeout(monkeypatch: pytest.MonkeyPatch) -> None:
    observed: list[object] = []

    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        observed.append(kwargs.get("timeout"))
        return _completed(args[0], 0, stdout="percy-raskova/babylon\n")

    monkeypatch.setattr(subprocess, "run", fake_run)
    assert conveyor.resolve_repo() == "percy-raskova/babylon"
    assert observed == [conveyor.GH_TIMEOUT_SECONDS]


def test_request_rebase_uses_bounded_timeout(monkeypatch: pytest.MonkeyPatch) -> None:
    observed: list[object] = []

    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        observed.append(kwargs.get("timeout"))
        return _completed(args[0], 0)

    monkeypatch.setattr(subprocess, "run", fake_run)
    assert conveyor.request_rebase("percy-raskova/babylon", 5) is True
    assert observed == [conveyor.GH_TIMEOUT_SECONDS]


def test_try_merge_uses_generous_timeout(monkeypatch: pytest.MonkeyPatch) -> None:
    observed: list[object] = []

    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        observed.append(kwargs.get("timeout"))
        return _completed(args[0], 0)

    monkeypatch.setattr(subprocess, "run", fake_run)
    assert conveyor.try_merge(5) == conveyor.MergeOutcome.SUCCESS
    assert observed == [conveyor.PR_MERGE_TIMEOUT_SECONDS]


# ---------------------------------------------------------------------------
# Repository resolution (no hard-coded REPO)
# ---------------------------------------------------------------------------


def test_resolve_repo_fails_loudly_when_gh_errors(monkeypatch: pytest.MonkeyPatch) -> None:
    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        return _completed(args[0], 1, stderr="not authenticated")

    monkeypatch.setattr(subprocess, "run", fake_run)
    with pytest.raises(RuntimeError, match="repo view"):
        conveyor.resolve_repo()


def test_resolve_repo_rejects_malformed_name(monkeypatch: pytest.MonkeyPatch) -> None:
    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        return _completed(args[0], 0, stdout="garbage-output\n")

    monkeypatch.setattr(subprocess, "run", fake_run)
    with pytest.raises(RuntimeError, match="nameWithOwner"):
        conveyor.resolve_repo()


# ---------------------------------------------------------------------------
# Pre-merge classification
# ---------------------------------------------------------------------------


def test_classify_update_allows_minor_and_patch() -> None:
    classification = conveyor.classify_update(MINOR_MESSAGE, "dependabot/uv/pillow-10.1.0")
    assert classification == conveyor.UpdateClass(False, "")


def test_classify_update_parks_semver_major() -> None:
    classification = conveyor.classify_update(
        MAJOR_MESSAGE, "dependabot/github_actions/actions-checkout-5"
    )
    assert classification.manual_review is True
    assert "semver-major" in classification.reason


def test_classify_update_parks_cargo_zero_x_minor() -> None:
    classification = conveyor.classify_update(CARGO_ZERO_X_MESSAGE, CARGO_HEAD_REF)
    assert classification.manual_review is True
    assert "0.x" in classification.reason


def test_classify_update_ignores_zero_x_minor_outside_cargo() -> None:
    classification = conveyor.classify_update(
        CARGO_ZERO_X_MESSAGE, "dependabot/uv/somepackage-0.10.0"
    )
    assert classification.manual_review is False


def test_classify_update_rejects_missing_trailers() -> None:
    with pytest.raises(RuntimeError, match="update metadata"):
        conveyor.classify_update("Bump something without trailers", CARGO_HEAD_REF)


# ---------------------------------------------------------------------------
# Rebase request de-duplication
# ---------------------------------------------------------------------------


def test_should_request_rebase_deduplicates_unchanged_head() -> None:
    state = conveyor.ConveyorState()
    assert conveyor.should_request_rebase(state, 5, HEAD_SHA) is True
    state.rebase_requested[5] = HEAD_SHA
    assert conveyor.should_request_rebase(state, 5, HEAD_SHA) is False
    assert conveyor.should_request_rebase(state, 5, MOVED_SHA) is True


# ---------------------------------------------------------------------------
# Typed pr:merge outcomes
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "exit_code",
    [0, 1, 2, 3, 4],
)
def test_try_merge_preserves_typed_outcome(
    monkeypatch: pytest.MonkeyPatch,
    exit_code: int,
) -> None:
    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        return _completed(args[0], exit_code)

    monkeypatch.setattr(subprocess, "run", fake_run)
    assert conveyor.try_merge(5) == conveyor.MergeOutcome(exit_code)


def test_try_merge_rejects_unknown_exit(monkeypatch: pytest.MonkeyPatch) -> None:
    def fake_run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
        return _completed(args[0], 9)

    monkeypatch.setattr(subprocess, "run", fake_run)
    with pytest.raises(RuntimeError, match="unknown exit"):
        conveyor.try_merge(5)


# ---------------------------------------------------------------------------
# conveyor_pass orchestration
# ---------------------------------------------------------------------------


def _pass_fakes(
    monkeypatch: pytest.MonkeyPatch,
    prs_by_pass: list[list[conveyor.OpenPr]],
    outcomes: list[conveyor.MergeOutcome],
) -> dict[str, object]:
    counters = {"merge": 0, "rebase": 0, "messages": 0}
    prs_remaining = list(prs_by_pass)

    def fake_list(repo: str) -> list[conveyor.OpenPr]:
        if prs_remaining:
            return prs_remaining.pop(0)
        return []

    def fake_merge(number: int) -> conveyor.MergeOutcome:
        counters["merge"] += 1
        return outcomes[min(counters["merge"] - 1, len(outcomes) - 1)]

    def fake_messages(repo: str, number: int) -> list[str]:
        counters["messages"] += 1
        return [MINOR_MESSAGE]

    def fake_rebase(repo: str, number: int) -> bool:
        counters["rebase"] += 1
        return True

    monkeypatch.setattr(conveyor, "list_open_prs", fake_list)
    monkeypatch.setattr(conveyor, "try_merge", fake_merge)
    monkeypatch.setattr(conveyor, "fetch_commit_messages", fake_messages)
    monkeypatch.setattr(conveyor, "request_rebase", fake_rebase)
    return counters


def test_terminal_hard_refusal_stops_conveyor(monkeypatch: pytest.MonkeyPatch) -> None:
    counters = _pass_fakes(
        monkeypatch,
        prs_by_pass=[[_open_pr(5)]],
        outcomes=[conveyor.MergeOutcome.HARD_REFUSAL],
    )
    with pytest.raises(conveyor.ConveyorStop, match="HARD_REFUSAL"):
        conveyor.conveyor_pass("o/r", conveyor.ConveyorState())
    assert counters["merge"] == 1


def test_terminal_mutation_indeterminate_stops_conveyor(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    counters = _pass_fakes(
        monkeypatch,
        prs_by_pass=[[_open_pr(5)]],
        outcomes=[conveyor.MergeOutcome.MUTATION_INDETERMINATE],
    )
    with pytest.raises(conveyor.ConveyorStop, match="MUTATION_INDETERMINATE"):
        conveyor.conveyor_pass("o/r", conveyor.ConveyorState())
    assert counters["merge"] == 1


def test_manual_review_outcome_parks_pr_for_session(monkeypatch: pytest.MonkeyPatch) -> None:
    state = conveyor.ConveyorState()
    counters = _pass_fakes(
        monkeypatch,
        prs_by_pass=[[_open_pr(5)], [_open_pr(5)]],
        outcomes=[conveyor.MergeOutcome.DEPENDABOT_MAJOR_REVIEW],
    )
    conveyor.conveyor_pass("o/r", state)
    assert 5 in state.manual_review
    conveyor.conveyor_pass("o/r", state)
    assert counters["merge"] == 1


def test_pending_outcome_retries_next_pass(monkeypatch: pytest.MonkeyPatch) -> None:
    counters = _pass_fakes(
        monkeypatch,
        prs_by_pass=[[_open_pr(5)], [_open_pr(5)]],
        outcomes=[conveyor.MergeOutcome.EXACT_HEAD_EVIDENCE_PENDING],
    )
    conveyor.conveyor_pass("o/r", conveyor.ConveyorState())
    conveyor.conveyor_pass("o/r", conveyor.ConveyorState())
    assert counters["merge"] == 2


def test_major_classification_parks_pr_before_merge(monkeypatch: pytest.MonkeyPatch) -> None:
    state = conveyor.ConveyorState()
    counters = _pass_fakes(
        monkeypatch,
        prs_by_pass=[[_open_pr(5)]],
        outcomes=[conveyor.MergeOutcome.SUCCESS],
    )

    def fake_messages(repo: str, number: int) -> list[str]:
        return [MAJOR_MESSAGE]

    monkeypatch.setattr(conveyor, "fetch_commit_messages", fake_messages)
    remaining, merged = conveyor.conveyor_pass("o/r", state)
    assert remaining == 1
    assert merged is False
    assert counters["merge"] == 0
    assert 5 in state.manual_review


def test_merge_recounts_queue_before_reporting_remaining(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    _pass_fakes(
        monkeypatch,
        prs_by_pass=[[_open_pr(5)], []],
        outcomes=[conveyor.MergeOutcome.SUCCESS],
    )
    remaining, merged = conveyor.conveyor_pass("o/r", conveyor.ConveyorState())
    assert merged is True
    assert remaining == 0


def test_behind_pr_gets_one_rebase_per_unchanged_head(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    state = conveyor.ConveyorState()
    counters = _pass_fakes(
        monkeypatch,
        prs_by_pass=[
            [_open_pr(7, state="BEHIND")],
            [_open_pr(7, state="BEHIND")],
            [_open_pr(7, state="BEHIND", head_oid=MOVED_SHA)],
        ],
        outcomes=[conveyor.MergeOutcome.SUCCESS],
    )
    conveyor.conveyor_pass("o/r", state)
    conveyor.conveyor_pass("o/r", state)
    assert counters["rebase"] == 1
    conveyor.conveyor_pass("o/r", state)
    assert counters["rebase"] == 2
    assert state.rebase_requested[7] == MOVED_SHA
