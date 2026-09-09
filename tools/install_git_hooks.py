#!/usr/bin/env python3
"""Install shared pre-commit hooks that execute the current checkout's environment."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

import yaml
from pre_commit.util import resource_text

GENERATED_DISPATCH = """if [ -x "$INSTALL_PYTHON" ]; then
    exec "$INSTALL_PYTHON" -mpre_commit "${ARGS[@]}"
elif command -v pre-commit > /dev/null; then
    exec pre-commit "${ARGS[@]}"
else
    echo '`pre-commit` not found.  Did you forget to activate your virtualenv?' 1>&2
    exit 1
fi
"""
CHECKOUT_DISPATCH = """if [ -L "$PWD/.venv" ] || [ ! -x "$INSTALL_PYTHON" ]; then
    echo 'Git hooks require this checkout-local .venv; remove a .venv symlink with unlink .venv, then run mise run install' >&2
    exit 1
fi
export PATH="$PWD/.venv/bin:$PATH"
exec "$INSTALL_PYTHON" -mpre_commit "${ARGS[@]}"
"""


def checkout_neutral_hook(generated: str) -> str:
    """Replace the pinned upstream launch policy without changing hook arguments."""
    if generated.count(GENERATED_DISPATCH) != 1:
        raise ValueError("pre-commit hook template changed; inspect it before installing hooks")
    neutral, count = re.subn(
        r"^INSTALL_PYTHON=.*$",
        'INSTALL_PYTHON="$PWD/.venv/bin/python"',
        generated,
        flags=re.MULTILINE,
    )
    if count != 1:
        raise ValueError("pre-commit hook template has no unique interpreter declaration")
    return neutral.replace(GENERATED_DISPATCH, CHECKOUT_DISPATCH)


def main() -> int:
    # Refuse a changed upstream template before touching shared hooks.
    checkout_neutral_hook(resource_text("hook-tmpl"))
    config = yaml.safe_load(Path(".pre-commit-config.yaml").read_text())
    hook_types = config["default_install_hook_types"]
    if hook_types != ["pre-commit", "commit-msg", "pre-push"]:
        raise ValueError("default_install_hook_types must contain the three governed Git hooks")
    # Preserve pre-commit's refusal of core.hooksPath and its legacy-hook migration.
    result = subprocess.run(  # noqa: S603
        [sys.executable, "-m", "pre_commit", "install"], check=False
    )
    if result.returncode:
        return result.returncode
    hooks = Path(
        subprocess.check_output(  # noqa: S603
            ["git", "rev-parse", "--path-format=absolute", "--git-path", "hooks"],  # noqa: S607
            text=True,
        ).strip()
    )
    rendered = {
        hooks / name: checkout_neutral_hook((hooks / name).read_text()) for name in hook_types
    }
    for path, content in rendered.items():
        path.write_text(content)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
