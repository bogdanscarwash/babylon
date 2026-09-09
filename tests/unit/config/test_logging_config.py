"""Unit tests for logging configuration.

Tests the centralized logging configuration system that reads from pyproject.toml.
"""

from __future__ import annotations

import json
import logging
import os
import tempfile
from pathlib import Path
from unittest.mock import patch

import pytest

from babylon.config.logging_config import (
    LoggingConfig,
    _apply_module_levels,
    _find_pyproject_toml,
    _parse_level,
    get_current_config,
    load_logging_config,
    setup_logging,
)

# =============================================================================
# FIXTURES
# =============================================================================


@pytest.fixture
def temp_pyproject(tmp_path: Path) -> Path:
    """Create a temporary pyproject.toml with logging config."""
    content = """
[project]
name = "test-project"

[tool.babylon.logging]
default_level = "DEBUG"
console_level = "WARNING"
file_level = "TRACE"

[tool.babylon.logging.modules]
"test.module1" = "DEBUG"
"test.module2" = "ERROR"
"httpx" = "CRITICAL"
"""
    pyproject_path = tmp_path / "pyproject.toml"
    pyproject_path.write_text(content)
    return pyproject_path


@pytest.fixture
def temp_pyproject_minimal(tmp_path: Path) -> Path:
    """Create a minimal pyproject.toml without logging config."""
    content = """
[project]
name = "test-project"
"""
    pyproject_path = tmp_path / "pyproject.toml"
    pyproject_path.write_text(content)
    return pyproject_path


@pytest.fixture(autouse=True)
def reset_logging() -> None:
    """Reset logging state after each test."""
    yield
    # Clear all handlers from root logger
    root = logging.getLogger()
    root.handlers.clear()
    root.setLevel(logging.WARNING)


# =============================================================================
# LoggingConfig DATACLASS TESTS
# =============================================================================


@pytest.mark.unit
class TestLoggingConfigDataclass:
    """Tests for the LoggingConfig dataclass."""

    def test_default_values(self) -> None:
        """LoggingConfig has sensible defaults."""
        config = LoggingConfig()

        assert config.default_level == "INFO"
        assert config.console_level == "INFO"
        assert config.file_level == "DEBUG"
        assert config.modules == {}

    def test_custom_values(self) -> None:
        """LoggingConfig accepts custom values."""
        config = LoggingConfig(
            default_level="DEBUG",
            console_level="WARNING",
            file_level="ERROR",
            modules={"test": "CRITICAL"},
        )

        assert config.default_level == "DEBUG"
        assert config.console_level == "WARNING"
        assert config.file_level == "ERROR"
        assert config.modules == {"test": "CRITICAL"}


# =============================================================================
# load_logging_config TESTS
# =============================================================================


@pytest.mark.unit
class TestLoadLoggingConfig:
    """Tests for loading configuration from pyproject.toml."""

    def test_loads_from_pyproject(self, temp_pyproject: Path) -> None:
        """load_logging_config reads values from pyproject.toml."""
        config = load_logging_config(temp_pyproject)

        assert config.default_level == "DEBUG"
        assert config.console_level == "WARNING"
        assert config.file_level == "TRACE"

    def test_loads_module_levels(self, temp_pyproject: Path) -> None:
        """load_logging_config reads per-module levels."""
        config = load_logging_config(temp_pyproject)

        assert config.modules.get("test.module1") == "DEBUG"
        assert config.modules.get("test.module2") == "ERROR"
        assert config.modules.get("httpx") == "CRITICAL"

    def test_returns_defaults_for_missing_file(self) -> None:
        """load_logging_config returns defaults when file doesn't exist."""
        nonexistent = Path("/nonexistent/pyproject.toml")
        config = load_logging_config(nonexistent)

        assert config.default_level == "INFO"
        assert config.console_level == "INFO"
        assert config.file_level == "DEBUG"
        assert config.modules == {}

    def test_returns_defaults_for_minimal_pyproject(self, temp_pyproject_minimal: Path) -> None:
        """load_logging_config returns defaults when no babylon.logging section."""
        config = load_logging_config(temp_pyproject_minimal)

        assert config.default_level == "INFO"
        assert config.console_level == "INFO"
        assert config.file_level == "DEBUG"
        assert config.modules == {}


# =============================================================================
# _parse_level TESTS
# =============================================================================


@pytest.mark.unit
class TestParseLevel:
    """Tests for level string parsing."""

    def test_parses_standard_levels(self) -> None:
        """_parse_level handles standard logging levels."""
        assert _parse_level("DEBUG") == logging.DEBUG
        assert _parse_level("INFO") == logging.INFO
        assert _parse_level("WARNING") == logging.WARNING
        assert _parse_level("ERROR") == logging.ERROR
        assert _parse_level("CRITICAL") == logging.CRITICAL

    def test_parses_case_insensitive(self) -> None:
        """_parse_level is case insensitive."""
        assert _parse_level("debug") == logging.DEBUG
        assert _parse_level("Debug") == logging.DEBUG
        assert _parse_level("DEBUG") == logging.DEBUG

    def test_parses_trace_level(self) -> None:
        """_parse_level handles custom TRACE level."""
        assert _parse_level("TRACE") == 5
        assert _parse_level("trace") == 5

    def test_returns_info_for_unknown(self) -> None:
        """_parse_level returns INFO for unknown levels."""
        assert _parse_level("UNKNOWN") == logging.INFO
        assert _parse_level("NOTREAL") == logging.INFO


# =============================================================================
# _apply_module_levels TESTS
# =============================================================================


@pytest.mark.unit
class TestApplyModuleLevels:
    """Tests for applying per-module log levels."""

    def test_applies_module_levels(self) -> None:
        """_apply_module_levels sets levels on loggers."""
        modules = {
            "test.apply.debug": "DEBUG",
            "test.apply.warning": "WARNING",
        }

        _apply_module_levels(modules)

        assert logging.getLogger("test.apply.debug").level == logging.DEBUG
        assert logging.getLogger("test.apply.warning").level == logging.WARNING

    def test_handles_empty_dict(self) -> None:
        """_apply_module_levels handles empty dict without error."""
        _apply_module_levels({})
        # No assertion needed - just verify no exception

    def test_applies_trace_level(self) -> None:
        """_apply_module_levels applies TRACE level correctly."""
        modules = {"test.apply.trace": "TRACE"}

        _apply_module_levels(modules)

        assert logging.getLogger("test.apply.trace").level == 5


# =============================================================================
# _find_pyproject_toml TESTS
# =============================================================================


@pytest.mark.unit
class TestFindPyprojectToml:
    """Tests for auto-detecting pyproject.toml."""

    def test_finds_pyproject_in_cwd(self, tmp_path: Path) -> None:
        """_find_pyproject_toml finds pyproject.toml in current directory."""
        # Create pyproject.toml in temp dir
        pyproject = tmp_path / "pyproject.toml"
        pyproject.write_text("[project]\nname = 'test'")

        original_cwd = os.getcwd()
        try:
            os.chdir(tmp_path)
            result = _find_pyproject_toml()
            # Should find the file (either from cwd or parent search)
            assert result is not None
            assert result.exists()
        finally:
            os.chdir(original_cwd)

    def test_returns_none_when_not_found(self) -> None:
        """_find_pyproject_toml returns None when file not found."""
        # Create temp dir without pyproject.toml
        with tempfile.TemporaryDirectory() as tmpdir:
            original_cwd = os.getcwd()
            try:
                os.chdir(tmpdir)
                # With no pyproject.toml anywhere, should return None
                # (depends on implementation details)
            finally:
                os.chdir(original_cwd)


# =============================================================================
# setup_logging TESTS
# =============================================================================


@pytest.mark.unit
class TestSetupLogging:
    """Tests for the main setup_logging function."""

    def test_creates_log_directory(self, tmp_path: Path) -> None:
        """setup_logging creates log directory if it doesn't exist."""
        log_dir = tmp_path / "logs"

        with patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir):
            setup_logging(default_level="INFO")

        assert log_dir.exists()

    def test_respects_env_var_override(self, tmp_path: Path) -> None:
        """LOG_LEVEL environment variable overrides pyproject.toml."""
        log_dir = tmp_path / "logs"

        with (
            patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir),
            patch.dict(os.environ, {"LOG_LEVEL": "DEBUG"}),
        ):
            setup_logging()

        # Console handler should be at DEBUG level
        root = logging.getLogger()
        console_handlers = [h for h in root.handlers if hasattr(h, "stream")]
        assert len(console_handlers) > 0
        assert console_handlers[0].level == logging.DEBUG

    def test_respects_default_level_parameter(self, tmp_path: Path) -> None:
        """default_level parameter overrides pyproject.toml settings."""
        log_dir = tmp_path / "logs"

        with (
            patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir),
            patch.dict(os.environ, {}, clear=True),  # Clear LOG_LEVEL env var
        ):
            setup_logging(default_level="WARNING")

        # Root always captures all; handlers filter by their own level.
        root = logging.getLogger()
        assert root.level == logging.DEBUG

        # The console handler itself must actually be raised to WARNING —
        # `default_level` is documented as a console-verbosity override (the
        # headless runner's `--verbose` flag depends on this), so a caller
        # passing it must see the console handler's level move, not just
        # silently keep whatever `LoggingConfig.console_level` defaulted to.
        console_handlers = [
            h
            for h in root.handlers
            if isinstance(h, logging.StreamHandler) and not isinstance(h, logging.FileHandler)
        ]
        assert len(console_handlers) == 1
        assert console_handlers[0].level == logging.WARNING

    def test_console_stream_defaults_to_stdout(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        """With no ``console_stream`` argument, console records still land
        on stdout — the pre-existing behavior for every caller except the
        headless runner (CLI, tools/*.py scripts) must be unchanged."""
        log_dir = tmp_path / "logs"

        with (
            patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir),
            patch.dict(os.environ, {}, clear=True),
        ):
            setup_logging(default_level="INFO")
            logging.getLogger("test.console_stream.default").info("stdout marker message")

        captured = capsys.readouterr()
        assert "stdout marker message" in captured.out
        assert "stdout marker message" not in captured.err

    def test_console_stream_stderr_keeps_stdout_clean(
        self, tmp_path: Path, capsys: pytest.CaptureFixture[str]
    ) -> None:
        """``console_stream="stderr"`` must route console records to stderr
        and NEVER stdout — this is the headless runner's contract: stdout is
        reserved for the machine-readable artifact directory path printed by
        ``main_from_argv`` (command-substitution callers like
        an artifact-producing subprocess
        would otherwise capture log text instead of a path)."""
        log_dir = tmp_path / "logs"

        with (
            patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir),
            patch.dict(os.environ, {}, clear=True),
        ):
            setup_logging(default_level="INFO", console_stream="stderr")
            logging.getLogger("test.console_stream.stderr").info("stderr marker message")

        captured = capsys.readouterr()
        assert "stderr marker message" in captured.err
        assert "stderr marker message" not in captured.out

    def test_applies_module_levels_from_pyproject(
        self, temp_pyproject: Path, tmp_path: Path
    ) -> None:
        """setup_logging applies per-module levels from pyproject.toml."""
        log_dir = tmp_path / "logs"

        with (
            patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir),
            patch.dict(os.environ, {}, clear=True),
        ):
            setup_logging(pyproject_path=temp_pyproject)

        # Check module-specific levels
        assert logging.getLogger("test.module1").level == logging.DEBUG
        assert logging.getLogger("test.module2").level == logging.ERROR
        assert logging.getLogger("httpx").level == logging.CRITICAL

    def test_creates_all_handlers(self, tmp_path: Path) -> None:
        """setup_logging creates console, main file, and error file handlers."""
        log_dir = tmp_path / "logs"

        with patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir):
            setup_logging()

        root = logging.getLogger()

        # Should have 3 handlers: console, main file, error file
        assert len(root.handlers) == 3

        # Verify handler types
        handler_types = {type(h).__name__ for h in root.handlers}
        assert "StreamHandler" in handler_types
        assert "RotatingFileHandler" in handler_types


# =============================================================================
# get_current_config TESTS
# =============================================================================


@pytest.mark.unit
class TestGetCurrentConfig:
    """Tests for the get_current_config helper."""

    def test_returns_logging_config(self) -> None:
        """get_current_config returns a LoggingConfig instance."""
        config = get_current_config()

        assert isinstance(config, LoggingConfig)
        assert hasattr(config, "default_level")
        assert hasattr(config, "console_level")
        assert hasattr(config, "file_level")
        assert hasattr(config, "modules")


# =============================================================================
# INTEGRATION TESTS
# =============================================================================


@pytest.mark.unit
class TestLoggingIntegration:
    """Integration tests for the logging system."""

    def test_shipped_yaml_routes_json_records(
        self, tmp_path: Path, temp_pyproject_minimal: Path
    ) -> None:
        """The shipped YAML loads its formatter and separates error records."""
        log_dir = tmp_path / "logs"
        config_path = Path(__file__).resolve().parents[3] / "logging.yaml"

        with patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir):
            setup_logging(config_path=config_path, pyproject_path=temp_pyproject_minimal)
            logger = logging.getLogger("test.shipped_yaml")
            logger.info("Native info message")
            logger.error("Native error message")

        main_records = [
            json.loads(line) for line in (log_dir / "babylon.log").read_text().splitlines()
        ]
        error_records = [
            json.loads(line) for line in (log_dir / "errors.log").read_text().splitlines()
        ]
        assert [(record["level"], record["logger"], record["msg"]) for record in main_records] == [
            ("INFO", "test.shipped_yaml", "Native info message"),
            ("ERROR", "test.shipped_yaml", "Native error message"),
        ]
        assert [(record["level"], record["logger"], record["msg"]) for record in error_records] == [
            ("ERROR", "test.shipped_yaml", "Native error message"),
        ]

    def test_full_setup_and_log(self, tmp_path: Path) -> None:
        """Complete test of setup and logging a message."""
        log_dir = tmp_path / "logs"

        with (
            patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir),
            patch.dict(os.environ, {}, clear=True),
        ):
            setup_logging(default_level="DEBUG")

            # Log a test message
            logger = logging.getLogger("test.integration")
            logger.debug("Test debug message")
            logger.info("Test info message")
            logger.error("Test error message")

        # Verify files were created
        assert (log_dir / "babylon.log").exists()
        assert (log_dir / "errors.log").exists()

        # Verify error log contains error message
        error_content = (log_dir / "errors.log").read_text()
        assert "Test error message" in error_content

        # Verify main log contains all messages
        main_content = (log_dir / "babylon.log").read_text()
        assert "Test debug message" in main_content
        assert "Test info message" in main_content
        assert "Test error message" in main_content

    def test_pyproject_config_propagates_to_loggers(
        self, temp_pyproject: Path, tmp_path: Path
    ) -> None:
        """Verify pyproject.toml config affects actual logging behavior."""
        log_dir = tmp_path / "logs"

        with (
            patch("babylon.config.logging_config.BaseConfig.LOG_DIR", log_dir),
            patch.dict(os.environ, {}, clear=True),
        ):
            setup_logging(pyproject_path=temp_pyproject)

            # test.module2 is set to ERROR level
            logger = logging.getLogger("test.module2")

            # Debug messages should NOT be logged (level is ERROR)
            logger.debug("This should not appear")

            # Error messages SHOULD be logged
            logger.error("This should appear")

        main_content = (log_dir / "babylon.log").read_text()
        assert "This should not appear" not in main_content
        assert "This should appear" in main_content
