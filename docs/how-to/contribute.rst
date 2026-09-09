Submit a Pull Request
=====================

This guide walks you through submitting a pull request to Babylon, from
branching to merge.

Prerequisites
-------------

- Git installed and configured
- Repository cloned: ``git clone https://github.com/percy-raskova/babylon.git``
- mise installed: https://mise.jdx.dev/getting-started.html
- Pinned Python and uv tools installed: ``mise install``
- Dependencies installed: ``mise run install``
- All Git hooks installed: ``mise run hooks``

Branch from dev
---------------

All contributions branch from ``dev``, not ``main``:

.. code-block:: bash

   # Ensure you're on dev and up to date
   git checkout dev
   git pull origin dev

   # Create your feature branch
   git checkout -b feature/your-feature-name

**Branch naming conventions:**

.. list-table::
   :widths: 20 40 40
   :header-rows: 1

   * - Prefix
     - Purpose
     - Example
   * - ``feature/``
     - New functionality
     - ``feature/solidarity-decay``
   * - ``fix/``
     - Bug fixes
     - ``fix/ideology-overflow``
   * - ``docs/``
     - Documentation
     - ``docs/add-topology-guide``
   * - ``refactor/``
     - Code improvements
     - ``refactor/simplify-survival``
   * - ``test/``
     - Test changes
     - ``test/add-formula-coverage``

Make Your Changes
-----------------

1. **Write code** following project standards:

   - Type hints on all functions
   - Docstrings in RST format
   - No hardcoded values (use ``GameDefines``)

2. **Run pre-commit hooks** (automatic on commit):

   .. code-block:: bash

      # Manual run
      uv run pre-commit run --all-files

3. **Run tests** to ensure nothing breaks:

   .. code-block:: bash

      # Fast tests (recommended during development)
      uv run pytest -m "not ai" --tb=short

      # Or use mise
      mise run test:all

Commit Your Changes
-------------------

Use `Conventional Commits <https://www.conventionalcommits.org/>`_ format:

.. code-block:: bash

   git add .
   git commit -m "feat(topology): add solidarity edge decay"

**Commit types:**

- ``feat:`` - New feature
- ``fix:`` - Bug fix
- ``docs:`` - Documentation only
- ``refactor:`` - Code change that neither fixes nor adds
- ``test:`` - Adding or correcting tests
- ``chore:`` - Maintenance tasks

Push and Create PR
------------------

.. code-block:: bash

   # Push your branch
   git push -u origin feature/your-feature-name

Then on GitHub:

1. Navigate to the repository
2. Click "Compare & pull request"
3. **Target branch**: Select ``dev`` (not ``main``)
4. Fill out the PR template
5. Submit

The PR Template
---------------

The template asks for:

.. code-block:: text

   ## What does this PR do?
   Brief description of the change.

   ## Related Issue
   Link to issue if applicable, or "N/A".

   ## Checklist
   - [ ] I've tested my changes locally
   - [ ] My code follows the existing style
   - [ ] I've updated documentation if needed

   ## Questions for Reviewers
   Anything you're unsure about?

.. tip::

   The checklist is a guide, not a gate. Don't stress if you can't check
   every box—the maintainer can help clean things up.

What Happens Next
-----------------

1. **CI runs automatically** on your PR:

   - Lint check (Ruff)
   - Type check (MyPy)
   - Tests (Pytest)
   - Documentation build (Sphinx)

2. **Review CI results**:

   - Green ``PASS`` = all checks pass
   - Red ``FAIL`` = something needs fixing (see logs)
   - Yellow ``WARN`` = advisory only (style suggestions)

3. **Maintainer reviews** and may:

   - Approve and merge
   - Request changes
   - Fix minor issues during merge

4. **Celebrate** when merged!

Handling CI Failures
--------------------

If CI fails, don't panic:

**Lint failures (Ruff)**
   .. code-block:: bash

      uv run ruff check . --fix
      git add . && git commit -m "fix: address lint issues"

**Type failures (MyPy)**
   Check the error message for missing type hints or type mismatches.
   See :doc:`/reference/error-codes` for common issues.

**Test failures (Pytest)**
   .. code-block:: bash

      # Run the specific failing test
      uv run pytest tests/path/to/test.py -v

**Documentation failures (Sphinx)**
   Usually a malformed docstring. Check for RST syntax issues.

.. note::

   Style check failures (yellow warnings) won't block your PR.
   The maintainer can fix formatting during merge.

Keeping Your Branch Updated
---------------------------

If ``dev`` moves forward while you're working:

.. code-block:: bash

   git checkout dev
   git pull origin dev
   git checkout feature/your-feature-name
   git rebase dev

   # If conflicts, resolve them, then:
   git push --force-with-lease

See Also
--------

- :doc:`/how-to/run-ci-locally` - Test CI before pushing
- :doc:`/reference/ci-workflow` - CI technical reference
- :doc:`/tutorials/installation` - Initial setup guide
