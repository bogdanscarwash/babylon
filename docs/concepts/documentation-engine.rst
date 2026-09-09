Documentation Engine
====================

Meta-documentation for Babylon's documentation system itself.

This document explains how the documentation you're reading is built,
themed, and organized. It serves as both reference and maintenance guide.

Architecture Overview
---------------------

The documentation system is built on three pillars:

**Sphinx**
   Python documentation generator that transforms source files into
   multiple output formats (HTML, PDF, EPUB).

**Diataxis**
   Information architecture framework organizing content into four
   distinct quadrants: tutorials, how-to guides, concepts, reference.

**Bunker Constructivism**
   Visual identity applied to both HTML and PDF outputs, derived from
   :doc:`aesthetics`.

.. code-block:: text

   docs/
   ├── tutorials/          # Learning-oriented (Diataxis)
   ├── how-to/             # Goal-oriented (Diataxis)
   ├── concepts/           # Understanding-oriented (Diataxis)
   ├── reference/          # Information-oriented (Diataxis)
   ├── api/                # Auto-generated API docs
   ├── commentary/         # Meta-level (outside Diataxis)
   ├── conf.py             # Sphinx configuration
   └── index.rst           # Root document

Source Formats
--------------

reStructuredText (RST)
~~~~~~~~~~~~~~~~~~~~~~

The primary format for structured documentation. Native Sphinx format
with full feature support.

.. code-block:: rst

   Section Title
   =============

   Subsection
   ----------

   .. note::

      Admonitions use directive syntax.

   Cross-references: :doc:`/concepts/architecture`, :func:`calculate_rent`

MyST Markdown
~~~~~~~~~~~~~

Extended Markdown with RST-equivalent features. Enabled via ``myst-parser``.
Used for narrative content like the Vibe Coding Manifesto.

Enabled extensions:

- ``dollarmath``: LaTeX math (``$E=mc^2$``)
- ``colon_fence``: Directive syntax (``::: note``)
- ``tasklist``: GitHub-style checkboxes
- ``strikethrough``: ~~deleted text~~
- ``linkify``: Auto-link URLs
- ``deflist``: Definition lists
- ``fieldlist``: Field lists
- ``attrs_inline``: Inline attributes ``{#id .class}``

.. code-block:: markdown

   # MyST Section

   :::{note}
   Admonitions use colon-fence syntax.
   :::

   Math: $P(S|R) = \frac{O}{R}$

PDF Generation
--------------

Two-Book Output
~~~~~~~~~~~~~~~

The build produces two PDF books:

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Book
     - Contents
   * - ``babylon-docs.pdf``
     - Main documentation: tutorials, how-to, concepts, reference, API
   * - ``babylon-commentary.pdf``
     - Meta-level: design philosophy, theoretical foundations

LaTeX Engine
~~~~~~~~~~~~

We use **XeLaTeX** for PDF generation:

- Native Unicode support (Greek letters Φ, σ, arrows →, em dashes —)
- OpenType font support via ``fontspec``
- Required for the Bunker Constructivism color definitions

The ``pdflatex`` engine cannot handle the Unicode characters used
throughout the documentation.

Build Commands
~~~~~~~~~~~~~~

.. code-block:: bash

   # PDF build
   mise run docs:pdf

   # HTML build
   mise run docs:build

   # Live-reload development server
   mise run docs:live

Luxe Gothic Theme
------------------

The PDF output applies the "Luxe Gothic" aesthetic---derived from the Kitty
terminal palette. Dark covers with crimson and gold accents frame white body
pages. See :doc:`aesthetics` for the full design system.

   *"The collapse of American hegemony is not the end of history.
   It is the revelation of what was always underneath."*

Color Palette (LaTeX)
~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 25 20 55

   * - Name
     - Hex
     - Usage
   * - BgDark
     - ``#1A0000``
     - Cover background (terminal darkness)
   * - DeepCrimson
     - ``#8B0A1A``
     - Section headings, structural emphasis (congealed violence)
   * - Crimson
     - ``#DC143C``
     - Active accents, borders, structural markers (arterial urgency)
   * - Gold
     - ``#FFD700``
     - Cover rules, category labels (extracted wealth)
   * - Silver
     - ``#C0C0C0``
     - Cover subtitle, body accents (industrial metal)
   * - DimGray
     - ``#696969``
     - Page numbers, inactive elements (concrete)
   * - BodyText
     - ``#2D2D2D``
     - Body text (near-black, AAA contrast)
   * - WarmGray
     - ``#F7F5F3``
     - Code backgrounds, blockquote fills (institutional neutral)
   * - BuriedHope
     - ``#1A3A1A``
     - Cover line (barely visible seed)
   * - ForestDim
     - ``#2A6B2A``
     - Revolutionary section headings
   * - PhosphorGreen
     - ``#39FF14``
     - Key phrases: :hope:`Organization is the difference.`

Custom RST Roles
~~~~~~~~~~~~~~~~

The ``:hope:`` role renders text in PhosphorGreen (PDF) or bright green (HTML):

.. code-block:: rst

   :hope:`Organization is the difference.`

Use only for revolutionary organization content.

Typography
~~~~~~~~~~

- **Main text**: TeX Gyre Termes (Times-like serif)
- **Headings**: TeX Gyre Heros (Helvetica-like sans)
- **Code**: DejaVu Sans Mono (Unicode coverage)
- **Chapter style**: Bjornstrup (professional book aesthetic)

These TeX Gyre fonts are included in TeX Live and provide consistent
cross-platform rendering.

Configuration Reference
-----------------------

Key settings from ``docs/conf.py``:

.. code-block:: python

   # LaTeX engine for Unicode support
   latex_engine = "xelatex"

   # Two-book output
   latex_documents = [
       ("docs-pdf-index", "babylon-docs.tex", ...),
       ("commentary/index", "babylon-commentary.tex", ...),
   ]

   # Eliminate blank pages before chapters
   latex_elements = {
       "extraclassoptions": "openany",
       "preamble": r"""
   \usepackage{fontspec}
   \usepackage{xcolor}
   \definecolor{PhosphorBurn}{HTML}{D40000}
   % ... color definitions
   \usepackage{sectsty}
   \chapterfont{\color{PhosphorBurn}}
   \sectionfont{\color{ThermalWarning}}
   """,
   }

API Documentation
-----------------

Autodoc System
~~~~~~~~~~~~~~

API documentation is auto-generated from Python docstrings using:

- ``sphinx.ext.autodoc``: Extract docstrings from modules
- ``sphinx.ext.autosummary``: Generate stub pages for modules
- ``sphinx.ext.napoleon``: Parse Google/NumPy style docstrings
- ``sphinx_autodoc_typehints``: Include type hints in docs

Docstring Format
~~~~~~~~~~~~~~~~

All public APIs use RST-format docstrings compatible with Sphinx:

.. code-block:: python

   def calculate_imperial_rent(wages: Currency, value: Currency) -> Currency:
       """Calculate imperial rent extracted via unequal exchange.

       Args:
           wages: Currency amount paid to workers in core.
           value: Currency amount of value actually produced.

       Returns:
           Imperial rent (Phi) extracted from periphery workers.

       Raises:
           ValueError: If wages or value are negative.

       Example:
           >>> calculate_imperial_rent(Currency(100.0), Currency(80.0))
           Currency(20.0)

       See Also:
           :func:`calculate_exploitation_rate`: Related metric.
       """

Intersphinx
~~~~~~~~~~~

Cross-references to external documentation:

- Python standard library: ``:py:class:`list```
- Pydantic: ``:class:`pydantic.BaseModel```
- NetworkX: ``:func:`networkx.algorithms.centrality.betweenness_centrality```

Mermaid Diagrams
----------------

Diagrams are written in Mermaid syntax and rendered via ``sphinxcontrib-mermaid``.

In RST files:

.. code-block:: rst

   .. mermaid::

      graph LR
          A[Source] --> B[Sphinx] --> C[HTML/PDF]

In MyST Markdown files:

.. code-block:: markdown

   ```{mermaid}
   graph LR
       A[Source] --> B[Sphinx] --> C[HTML/PDF]
   ```

.. note::

   Mermaid diagrams render in HTML but appear as placeholders in PDF.
   For critical diagrams in print, use ASCII art or static images.

Maintenance
-----------

Adding New Documents
~~~~~~~~~~~~~~~~~~~~

1. Create ``.rst`` or ``.md`` file in appropriate Diataxis quadrant
2. Add to parent ``index.rst`` toctree
3. Build and verify: ``mise run docs:live``

Updating PDF Theme
~~~~~~~~~~~~~~~~~~

Edit the ``latex_elements["preamble"]`` in ``docs/conf.py``. Key packages:

- ``xcolor``: Color definitions
- ``sectsty``: Section heading colors
- ``fontspec``: Font selection (XeLaTeX only)
- ``fncychap``: Chapter title styling

Testing Documentation
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Run doctest examples in code
   mise run test:doctest

   # Check documentation coverage
   uv run sphinx-build -b coverage docs docs/_build/coverage

   # Check for broken links
   uv run sphinx-build -b linkcheck docs docs/_build/linkcheck

See Also
--------

- :doc:`aesthetics` - Visual design guidelines (source of PDF theme)
- :doc:`vibe-coding/index` - Development methodology manifesto
- :doc:`/commentary/design-philosophy` - Architectural principles
