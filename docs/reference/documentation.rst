Documentation System Reference
==============================

Complete reference for Babylon's documentation build system.

For conceptual overview, see :doc:`/concepts/documentation-engine`.

Build Commands
--------------

.. list-table::
   :header-rows: 1
   :widths: 35 65

   * - Command
     - Description
   * - ``mise run docs:build``
     - Build HTML documentation to ``docs/_build/html/``
   * - ``mise run docs:live``
     - Live-reload server at ``http://localhost:8000``
   * - ``mise run docs:pdf``
     - Build PDF documentation via XeLaTeX
   * - ``mise run test:doctest``
     - Run doctest examples in Python code
   * - ``mise run clean``
     - Remove build artifacts including ``docs/_build/``

Direct Sphinx Commands
~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # HTML build
   uv run sphinx-build -b html docs docs/_build/html

   # LaTeX source generation
   uv run sphinx-build -b latex docs docs/_build/latex

   # PDF compilation (after LaTeX generation)
   cd docs/_build/latex && make all-pdf

   # Check documentation coverage
   uv run sphinx-build -b coverage docs docs/_build/coverage

   # Check for broken links
   uv run sphinx-build -b linkcheck docs docs/_build/linkcheck

Sphinx Extensions
-----------------

Core Extensions
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Extension
     - Purpose
   * - ``sphinx.ext.autodoc``
     - Extract docstrings from Python modules
   * - ``sphinx.ext.autosummary``
     - Generate API stub pages automatically
   * - ``sphinx.ext.napoleon``
     - Parse Google/NumPy style docstrings
   * - ``sphinx.ext.viewcode``
     - Add links to highlighted source code
   * - ``sphinx.ext.intersphinx``
     - Cross-reference external Sphinx docs
   * - ``sphinx.ext.todo``
     - Support ``.. todo::`` directives
   * - ``sphinx.ext.coverage``
     - Track documentation coverage
   * - ``sphinx.ext.doctest``
     - Run doctest blocks in documentation

Third-Party Extensions
~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Extension
     - Purpose
   * - ``sphinx_autodoc_typehints``
     - Include type hints in autodoc output
   * - ``myst_parser``
     - Parse MyST Markdown files
   * - ``sphinxcontrib.mermaid``
     - Render Mermaid diagrams

MyST Extensions
~~~~~~~~~~~~~~~

Enabled in ``myst_enable_extensions``:

.. code-block:: python

   myst_enable_extensions = [
       "colon_fence",      # ::: directive syntax
       "deflist",          # Definition lists
       "fieldlist",        # Field lists
       "replacements",     # Typography replacements
       "smartquotes",      # Smart quote conversion
       "substitution",     # Substitution references
       "strikethrough",    # ~~deleted~~ syntax
       "linkify",          # Auto-link URLs
       "tasklist",         # - [ ] checkbox syntax
       "attrs_inline",     # {#id .class} attributes
       "dollarmath",       # $math$ and $$math$$
       "html_admonition",  # HTML-style admonitions
       "html_image",       # HTML-style images
   ]

Intersphinx Mappings
--------------------

External documentation cross-references:

.. code-block:: python

   intersphinx_mapping = {
       "python": ("https://docs.python.org/3", None),
       "pydantic": ("https://docs.pydantic.dev/latest/", None),
       "networkx": ("https://networkx.org/documentation/stable/", None),
   }

Usage examples:

- ``:py:class:`list``` → Python list class
- ``:class:`pydantic.BaseModel``` → Pydantic BaseModel
- ``:func:`networkx.betweenness_centrality``` → NetworkX function

PDF Configuration
-----------------

LaTeX Documents
~~~~~~~~~~~~~~~

Two books are generated:

.. code-block:: python

   latex_documents = [
       # (source, filename, title, author, documentclass)
       (
           "docs-pdf-index",
           "babylon-docs.tex",
           "Babylon: The Fall of America",
           "Persephone Raskova",
           "manual",
       ),
       (
           "commentary/index",
           "babylon-commentary.tex",
           "Babylon: The Fall of America\\\\{\\large Design Philosophy ...}",
           "Persephone Raskova",
           "manual",
       ),
   ]

LaTeX Elements
~~~~~~~~~~~~~~

.. code-block:: python

   latex_elements = {
       "papersize": "letterpaper",
       "pointsize": "11pt",
       "extraclassoptions": "openany",  # No blank pages before chapters
       "fncychap": r"\usepackage[Sonny]{fncychap}",
       "printindex": r"\footnotesize\raggedright\printindex",
       "preamble": "...",  # See Bunker Constructivism Theme below
   }

Necropolis Codex Theme
~~~~~~~~~~~~~~~~~~~~~~

Complete LaTeX preamble for PDF styling (see ``docs/conf.py``):

.. code-block:: latex

   % Typography (xelatex fontspec)
   \usepackage{fontspec}
   \setmainfont{TeX Gyre Termes}
   \setsansfont{TeX Gyre Heros}
   \setmonofont{DejaVu Sans Mono}

   % Color definitions - Necropolis Codex Palette
   \definecolor{AbsoluteVoid}{HTML}{0A0707}    % Cover top gradient
   \definecolor{DriedBlood}{HTML}{4A1818}      % Chapter headings
   \definecolor{Rust}{HTML}{6B4A3A}            % Section headings
   \definecolor{AshPaper}{HTML}{D4C9B8}        % Page backgrounds
   \definecolor{AshInk}{HTML}{3D3A36}          % Body text
   \definecolor{Bone}{HTML}{8B7B6B}            % Cover title, page numbers

   % Accent Colors (Buried Hope)
   \definecolor{BuriedHope}{HTML}{1A3A1A}      % Cover line
   \definecolor{ForestDim}{HTML}{2A6B2A}       % Revolutionary headings
   \definecolor{PhosphorGreen}{HTML}{39FF14}   % Key phrases

   % Revolutionary hope commands
   \newcommand{\hope}[1]{{\color{PhosphorGreen}#1}}
   \newcommand{\hopedim}[1]{{\color{ForestDim}#1}}

   % Hyperlink styling
   \hypersetup{
       colorlinks=true,
       linkcolor=Rust,
       urlcolor=Bone,
       citecolor=AshInk,
   }

   % Section heading colors
   \usepackage{sectsty}
   \chapterfont{\color{DriedBlood}}
   \sectionfont{\color{Rust}}
   \subsectionfont{\color{AshInk}}

Custom RST Role
~~~~~~~~~~~~~~~

The ``:hope:`` role is defined in ``docs/_ext/hope_roles.py``:

.. code-block:: rst

   :hope:`Organization is the difference.`

Use only for content about revolutionary organization.

Autodoc Configuration
---------------------

.. code-block:: python

   autodoc_default_options = {
       "members": True,
       "member-order": "bysource",
       "special-members": "__init__",
       "undoc-members": True,
       "exclude-members": "__weakref__",
       "show-inheritance": True,
   }

   autodoc_preserve_defaults = True
   autodoc_typehints = "description"
   autodoc_typehints_format = "short"

Napoleon (Docstring) Configuration
----------------------------------

.. code-block:: python

   napoleon_google_docstring = True
   napoleon_numpy_docstring = True
   napoleon_include_init_with_doc = True
   napoleon_use_admonition_for_examples = True
   napoleon_use_admonition_for_notes = True
   napoleon_use_param = True
   napoleon_use_rtype = True
   napoleon_attr_annotations = True
   napoleon_preprocess_types = True

Mermaid Configuration
---------------------

.. code-block:: python

   mermaid_version = "11.4.1"
   mermaid_init_config = {
       "startOnLoad": True,
       "theme": "neutral",
   }

   # Enable mermaid fences in MyST
   myst_fence_as_directive = ["mermaid"]

File Exclusions
---------------

Files excluded from documentation build:

.. code-block:: python

   exclude_patterns = [
       "_build",
       "Thumbs.db",
       ".DS_Store",
       # Legacy markdown files not in toctree
       "AESTHETICS.md",
       "AI_COMMS.md",
       "CHANGELOG.md",
       # ... (orphan files from migration)
   ]

Output Artifacts
----------------

Build Locations
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 60

   * - Path
     - Contents
   * - ``docs/_build/html/``
     - HTML documentation site
   * - ``docs/_build/latex/``
     - LaTeX source and PDF files
   * - ``docs/_build/latex/babylon-docs.pdf``
     - Main documentation (~600 pages)
   * - ``docs/_build/latex/babylon-commentary.pdf``
     - Commentary book (~15 pages)
   * - ``docs/_build/coverage/``
     - Documentation coverage report
   * - ``docs/_build/linkcheck/``
     - Broken link report

Generated Files
~~~~~~~~~~~~~~~

The autosummary extension generates stub files in:

.. code-block:: text

   docs/api/_autosummary/
   ├── babylon.intelligence.ai.rst
   ├── babylon.config.rst
   ├── babylon.engine.rst
   ├── babylon.models.rst
   └── ... (all public modules)

These files are auto-generated and should not be manually edited.

Known Limitations
-----------------

Box-Drawing Characters
~~~~~~~~~~~~~~~~~~~~~~

The TeX Gyre Cursor font lacks box-drawing characters (``└``, ``─``).
ASCII art tree structures in code blocks will show missing character
warnings but render with placeholder glyphs.

Mermaid in PDF
~~~~~~~~~~~~~~

Mermaid diagrams render in HTML but not in PDF. The PDF shows placeholder
text. For critical diagrams, use ASCII art or embed static images.

Emoji Support
~~~~~~~~~~~~~

Emoji and special Unicode symbols are not used in documentation.
The TeX Gyre fonts do not include emoji glyphs, so all content uses
text-based alternatives for cross-platform compatibility.

See Also
--------

- :doc:`/concepts/documentation-engine` - Conceptual overview
- :doc:`/concepts/aesthetics` - Visual design source
- :doc:`design-system` - Complete design tokens
