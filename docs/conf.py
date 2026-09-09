# Configuration file for the Sphinx documentation builder.
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import os
import sys
import tomllib
from pathlib import Path

# Add source directory to path for autodoc
sys.path.insert(0, os.path.abspath("../src"))

# Add _ext directory for custom extensions
sys.path.insert(0, os.path.abspath("_ext"))

# -- Project information -----------------------------------------------------
project = "Babylon"
copyright = "2025, Babylon Project Contributors"
author = "Babylon Project Contributors"

# The release manual follows the same tracked project version as packaging.
with (Path(__file__).resolve().parents[1] / "pyproject.toml").open("rb") as metadata_file:
    release = tomllib.load(metadata_file)["project"]["version"]
version = ".".join(release.split(".")[:2])

# -- General configuration ---------------------------------------------------
extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.napoleon",
    "sphinx.ext.viewcode",
    "sphinx.ext.intersphinx",
    "sphinx.ext.todo",
    "sphinx.ext.coverage",  # Track documentation coverage
    "sphinx.ext.doctest",  # Run doctest examples
    "sphinx_autodoc_typehints",
    "myst_parser",
    "sphinxcontrib.mermaid",  # Mermaid diagram support
    "hope_roles",  # Necropolis Codex :hope: role
]

# Templates path
templates_path = ["_templates"]

# Patterns to exclude
# Exclude legacy/orphan markdown files that aren't in any toctree
exclude_patterns = [
    "_build",
    "Thumbs.db",
    ".DS_Store",
    # Legacy markdown files not in toctree
    "AESTHETICS.md",
    "AI_COMMS.md",
    "CHANGELOG.md",
    "CHROMA.md",
    "CONFIGURATION.md",
    "CONTEXT_WINDOW.md",
    "ERROR_CODES.md",
    "GUI_PLAN.md",
    "LOGGING.md",
    "OBJECT_TRACKING.md",
    "SEMANTIC_VERSIONING_SPECIFICATION.md",
    "census/census_tool_blueprint.md",
    "census/census_tool_blueprint_v2.md",
    "chroma-troubleshooting.md",
]

# Source file suffixes
source_suffix = {
    ".rst": "restructuredtext",
    ".md": "markdown",
}

# The master toctree document
master_doc = "index"

# -- Options for HTML output -------------------------------------------------
html_theme = "sphinx_rtd_theme"
html_static_path = ["_static"]  # Necropolis styling

html_theme_options = {
    "navigation_depth": 4,
    "collapse_navigation": False,
    "sticky_navigation": True,
    "includehidden": True,
    "titles_only": False,
    "logo_only": False,
}

# HTML titles for search and breadcrumbs
html_title = "Babylon Documentation"
html_short_title = "Babylon"

# -- Extension configuration -------------------------------------------------

# Autodoc settings
autodoc_default_options = {
    "members": True,
    "member-order": "bysource",
    "special-members": "__init__",
    "undoc-members": True,
    "exclude-members": "__weakref__",
    "show-inheritance": True,
    # Note: inherited-members disabled to prevent Pydantic field duplication warnings
}

# Preserve default argument values in signatures
autodoc_preserve_defaults = True

autodoc_typehints = "description"
autodoc_typehints_format = "short"

# Napoleon settings (for Google/NumPy style docstrings)
napoleon_google_docstring = True
napoleon_numpy_docstring = True
napoleon_include_init_with_doc = True
napoleon_include_private_with_doc = False
napoleon_include_special_with_doc = True
napoleon_use_admonition_for_examples = True
napoleon_use_admonition_for_notes = True
napoleon_use_admonition_for_references = True
napoleon_use_ivar = False
napoleon_use_param = True
napoleon_use_rtype = True
napoleon_attr_annotations = True
napoleon_preprocess_types = True  # Enable type cross-references

# Intersphinx mapping
intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
    "pydantic": ("https://docs.pydantic.dev/latest/", None),
    "networkx": ("https://networkx.org/documentation/stable/", None),
}

# MyST parser settings
# Full GFM (GitHub Flavored Markdown) support plus extended features
myst_enable_extensions = [
    # Core formatting
    "colon_fence",
    "deflist",
    "fieldlist",
    "replacements",
    "smartquotes",
    "substitution",
    # GFM features
    "strikethrough",  # ~~text~~ support
    "linkify",  # Auto-link URLs
    "tasklist",  # - [ ] checkbox support
    # Extended features
    "attrs_inline",  # {#id .class} inline attributes
    "dollarmath",  # $math$ and $$math$$ support
    "html_admonition",  # HTML-style admonitions
    "html_image",  # HTML-style images
]
myst_heading_anchors = 3  # Generate anchors for h1-h3 headings

# Todo extension
todo_include_todos = True

# The API overview links the authoritative Rust source and current Python
# periphery explicitly. Retired recursive Python API generation has no consumer.
autosummary_generate = False

# Suppress known-benign warnings that don't affect documentation quality
# This allows CI to run with -W (warnings as errors) while ignoring noise
suppress_warnings = [
    # Duplicate object descriptions from Pydantic models re-exported in __init__.py
    # These occur because autosummary documents both the original and re-exported location
    "autodoc",
    "autodoc.import_object",
    # Reference warnings (intersphinx, cross-refs)
    "ref.python",
    "ref.ref",
    # MyST cross-reference warnings
    "myst.xref_missing",
    # Docutils inline markup warnings (usually from docstrings with special chars)
    "docutils",
]

# Mermaid configuration (sphinxcontrib-mermaid)
# See: https://github.com/mgaitan/sphinxcontrib-mermaid
mermaid_version = "11.4.1"  # Latest stable from jsdelivr CDN
mermaid_init_js = """
mermaid.initialize({
    startOnLoad: true,
    theme: 'neutral',
    themeVariables: {
        primaryColor: '#8B0A1A',
        primaryTextColor: '#2D2D2D',
        primaryBorderColor: '#DC143C',
        lineColor: '#DC143C',
        secondaryColor: '#F7F5F3',
        tertiaryColor: '#696969'
    }
});
"""

# HTML output: use 'raw' for browser-side JavaScript rendering (fast, no mmdc needed)
# LaTeX/PDF output: sphinxcontrib-mermaid automatically uses mmdc for LaTeX regardless
mermaid_output_format = "raw"

# PDF/LaTeX builds require mermaid-cli (mmdc) with Puppeteer
# Install: npm install -g @mermaid-js/mermaid-cli
# Then: npx puppeteer browsers install chrome-headless-shell
mermaid_cmd = "mmdc"
mermaid_cmd_shell = True
mermaid_params = [
    "-p",
    "puppeteer-config.json",  # Puppeteer config for headless Chrome
    "--theme",
    "neutral",
    "--backgroundColor",
    "transparent",
    "--width",
    "800",
]
# Enable verbose output to debug mmdc issues
mermaid_verbose = True

# Enable mermaid fences in MyST markdown files
myst_fence_as_directive = ["mermaid"]

# -- Options for LaTeX/PDF output --------------------------------------------

# 2 PDF books: main documentation + commentary
latex_documents = [
    # 1. Main documentation (tutorials, how-to, concepts, reference, API)
    (
        "docs-pdf-index",
        "babylon-docs.tex",
        "Babylon: The Fall of America",
        "Persephone Raskova",
        "manual",
    ),
    # 2. Meta-commentary book (design philosophy, theoretical foundations)
    (
        "commentary/index",
        "babylon-commentary.tex",
        "Babylon: The Fall of America\\\\{\\large Design Philosophy \\& Theoretical Foundations}",
        "Persephone Raskova",
        "manual",
    ),
]

# LaTeX styling for professional book output
# "Luxe Gothic" aesthetic — Kitty terminal palette applied to Sphinx PDF
# Color scheme: ~/.config/kitty/current-theme.conf
# Design philosophy: ~/.config/kitty/docs/04-DESIGN-RATIONALE.md
# Matches md-to-pdf skill template: ~/.claude/skills/md-to-pdf/assets/template.tex
latex_elements = {
    "papersize": "letterpaper",
    "pointsize": "11pt",
    # Book-style layout
    "extraclassoptions": "openany,twoside",
    # CRITICAL: Pass options to xcolor BEFORE Sphinx loads it
    # This fixes "Option clash for package xcolor" errors
    "passoptionstopackages": r"\PassOptionsToPackage{svgnames,dvipsnames,table}{xcolor}",
    # Professional fonts via fontspec (requires xelatex)
    # DejaVu Sans Mono has better Unicode coverage (box-drawing chars, etc.)
    "fontpkg": r"""
\usepackage{fontspec}
\setmainfont{TeX Gyre Termes}
\setsansfont{TeX Gyre Heros}
\setmonofont{DejaVu Sans Mono}
""",
    # Custom preamble: Luxe Gothic — Kitty terminal palette for Sphinx PDF
    "preamble": r"""
% ============================================================================
% LUXE GOTHIC THEME — Kitty Terminal Palette for Sphinx PDF
% Color scheme from ~/.config/kitty/current-theme.conf (Luxe Gothic / ksbc)
% Design philosophy from ~/.config/kitty/docs/04-DESIGN-RATIONALE.md:
%   Gold    = interactive / navigational (links, markers, separators)
%   Crimson = structural accent (headings, borders, rules)
%   Dimgray = inactive / structural (headers, attributions)
%
% Cover: dark (#1A0000) with crimson/gold accents.
% Body pages: white with deep crimson headings, crimson accents.
% Matches: ~/.claude/skills/md-to-pdf/assets/template.tex
% ============================================================================

% Better typography - subtle kerning and spacing improvements
\usepackage{microtype}

% Fix overfull hbox warnings - allow flexible line breaking
% Long function names in API docs cause strict width violations
\sloppy
\emergencystretch=2em

% Fix fancyhdr headheight warning
\setlength{\headheight}{24pt}
\addtolength{\topmargin}{-12pt}

% ============================================================================
% COLOR DEFINITIONS — Kitty terminal "Luxe Gothic" palette
% Source: ~/.config/kitty/current-theme.conf
%         ~/.config/kitty/docs/02-DESIGN-TOKENS.md
% ============================================================================

% --- Core terminal colors (used on cover page) ---
\definecolor{bgdark}{HTML}{1A0000}         % Terminal background: deep burgundy-black
\definecolor{fglight}{HTML}{E8E8E8}        % Bright foreground
\definecolor{silver}{HTML}{C0C0C0}         % Terminal foreground: silver
\definecolor{gold}{HTML}{FFD700}           % color3 / interactive: gold
\definecolor{crimson}{HTML}{DC143C}        % color1 / active border: crimson
\definecolor{dimgray}{HTML}{696969}        % Muted gray (readable on light bg)

% --- Semantic colors for light body pages ---
\definecolor{bodytext}{HTML}{2D2D2D}       % Near-black: body text on white pages
\definecolor{deepcrimson}{HTML}{8B0A1A}    % Deep crimson: chapter/section headings
\definecolor{subheadcolor}{HTML}{3D3D3D}   % Charcoal: subsection headings
\definecolor{darkgoldenrod}{HTML}{B8860B}  % Dark gold: external links
\definecolor{boxbg}{HTML}{F0EDED}          % Warm light gray: admonitions, callouts
\definecolor{codebg}{HTML}{F5F2F0}         % Pale warm gray: code blocks
\definecolor{tablerule}{HTML}{CCCCCC}      % Light gray: table rules

% --- ANSI palette (for syntax highlighting) ---
\definecolor{ansigreen}{HTML}{228B22}      % color2: forest green
\definecolor{ansicyan}{HTML}{008B8B}       % color6: dark cyan

% --- Revolutionary hope accent (conditional on organization) ---
\definecolor{ForestDim}{HTML}{2A6B2A}      % Dim green: hope in revolutionary content
\definecolor{PhosphorGreen}{HTML}{39FF14}  % Phosphor: "Organization is the difference"

% ============================================================================
% PAGE STYLING — White background, near-black body text
% Cover uses TikZ overlay for dark background.
% ============================================================================
\color{bodytext}

% Enhanced PDF bookmarks (better than hyperref alone)
\usepackage{bookmark}

% TikZ for cover page graphics (must be loaded BEFORE sphinxmaketitle definition)
\usepackage{tikz}

% ============================================================================
% CUSTOM COVER PAGE — Luxe Gothic
% Dark burgundy-black with crimson accent bar, gold rules.
% Matches md-to-pdf skill template cover design.
% ============================================================================
\makeatletter
\renewcommand{\sphinxmaketitle}{%
  \begin{titlepage}%
    \begin{tikzpicture}[remember picture,overlay]
      % Full page dark background (Luxe Gothic terminal bg)
      \fill[bgdark] (current page.south west) rectangle (current page.north east);

      % Subtle vertical crimson line on left (active border accent)
      \fill[crimson]
        ([xshift=0.9in]current page.south west) rectangle
        ([xshift=0.93in]current page.north west);

      % Gold rule near top
      \draw[gold, line width=0.8pt]
        ([xshift=1.15in, yshift=-2.2in]current page.north west) --
        ([xshift=-1.15in, yshift=-2.2in]current page.north east);

      % Category / subtitle (gold — navigational)
      \node[anchor=north west, text width=5.5in, inner sep=0pt]
        at ([xshift=1.3in, yshift=-2.6in]current page.north west)
        {{\fontsize{11}{13}\selectfont\sffamily\color{gold}%
          A SIMULATION OF IMPERIAL COLLAPSE}};

      % Main title (bright white — emphasis)
      \node[anchor=north west, text width=5.5in, inner sep=0pt]
        at ([xshift=1.3in, yshift=-3.2in]current page.north west)
        {{\fontsize{28}{34}\selectfont\sffamily\bfseries\color{fglight}%
          \@title}};

      % Mantra (silver italic)
      \node[anchor=north west, text width=5.5in, inner sep=0pt]
        at ([xshift=1.3in, yshift=-5.0in]current page.north west)
        {{\fontsize{12}{15}\selectfont\itshape\color{silver}%
          ``The only escape is revolutionary organization.''}};

      % Crimson rule above author block
      \draw[crimson, line width=0.6pt]
        ([xshift=1.3in, yshift=3.2in]current page.south west) --
        ([xshift=4.0in, yshift=3.2in]current page.south west);

      % Author (bright white)
      \node[anchor=north west, text width=5in, inner sep=0pt]
        at ([xshift=1.3in, yshift=2.95in]current page.south west)
        {{\fontsize{11}{14}\selectfont\sffamily\color{fglight}%
          \@author}};

      % Version (silver, dimmer)
      \node[anchor=north west, text width=5in, inner sep=0pt]
        at ([xshift=1.3in, yshift=2.5in]current page.south west)
        {{\fontsize{9}{11}\selectfont\sffamily\color{silver}%
          Version \py@release}};

      % Gold rule near bottom
      \draw[gold, line width=0.8pt]
        ([xshift=1.15in, yshift=1.5in]current page.south west) --
        ([xshift=-1.15in, yshift=1.5in]current page.south east);

    \end{tikzpicture}%
  \end{titlepage}%
  \clearpage%
}
\makeatother

% ============================================================================
% HYPERLINK STYLING — Luxe Gothic Navigation
% Internal: deep crimson (structural accent)
% External: dark goldenrod (gold-derived, readable on white)
% ============================================================================
\hypersetup{
    colorlinks=true,
    linkcolor=deepcrimson,
    urlcolor=darkgoldenrod,
    citecolor=deepcrimson,
    bookmarks=true,
    bookmarksnumbered=true,
    bookmarksopen=true,
    bookmarksopenlevel=2,
    pdfstartview=FitH,
}

% ============================================================================
% HEADING COLORS — Luxe Gothic Hierarchy
% deepcrimson (chapters/sections) → subheadcolor (subsections)
% Crimson = structural accent, readable on white pages
% ============================================================================
\usepackage{sectsty}
\chapterfont{\color{deepcrimson}}
\sectionfont{\color{deepcrimson}}
\subsectionfont{\color{subheadcolor}}

% ============================================================================
% TABLE OF CONTENTS — Luxe Gothic palette
% ============================================================================
\usepackage[titles]{tocloft}
\renewcommand{\cftchapfont}{\bfseries\color{deepcrimson}}
\renewcommand{\cftsecfont}{\color{deepcrimson}}
\renewcommand{\cftsubsecfont}{\color{subheadcolor}}
\renewcommand{\cftchappagefont}{\bfseries\color{dimgray}}
\renewcommand{\cftsecpagefont}{\color{dimgray}}
\renewcommand{\cftsubsecpagefont}{\color{dimgray}}

% ============================================================================
% FANCY HEADERS — Luxe Gothic
% Dimgray text, crimson footer rule
% ============================================================================
\usepackage{fancyhdr}
\pagestyle{fancy}
\fancyhf{}
\fancyhead[LE,RO]{\color{dimgray}\thepage}
\fancyhead[RE]{\color{dimgray}\nouppercase{\leftmark}}
\fancyhead[LO]{\color{dimgray}\nouppercase{\rightmark}}
\renewcommand{\headrulewidth}{0pt}
\fancyfoot[C]{{\color{crimson}\rule{0.3\textwidth}{0.4pt}}}

% Admonition styling
\usepackage{tcolorbox}
\tcbuselibrary{skins,breakable}

% Custom title page elements
\newcommand{\babylonsubtitle}[1]{%
    \vspace{0.5em}%
    {\Large\color{silver}#1}%
}

% ============================================================================
% REVOLUTIONARY HOPE STYLING — Conditional hope for content about organization
% Use \hope{text} for PhosphorGreen text - only for:
% - "Organization is the difference"
% - P(S|R) > P(S|A) / Warsaw Ghetto Dynamic
% - Solidarity transmission, critical window, enforcer radicalization
% ============================================================================
\newcommand{\hope}[1]{{\color{PhosphorGreen}#1}}
\newcommand{\hopedim}[1]{{\color{ForestDim}#1}}
""",
    # Chapter heading style - Bjornstrup is professional book-like
    "fncychap": r"\usepackage[Bjornstrup]{fncychap}",
    # Sphinx-specific styling — Luxe Gothic palette
    # TitleColor: deepcrimson (139,10,26) — structural accent
    # InnerLinkColor: deepcrimson (139,10,26) — internal navigation
    # OuterLinkColor: darkgoldenrod (184,134,11) — external links
    "sphinxsetup": r"""
        TitleColor={RGB}{139,10,26},
        InnerLinkColor={RGB}{139,10,26},
        OuterLinkColor={RGB}{184,134,11},
    """,
    # Index formatting
    "printindex": r"\footnotesize\raggedright\printindex",
}

# LaTeX engine (xelatex has native Unicode support for Greek letters, arrows, etc.)
latex_engine = "xelatex"

# Use makeindex instead of xindy to avoid encoding issues
latex_use_xindy = False


# -- Custom RST Roles for Necropolis Codex -----------------------------------
# The :hope: and :hopedim: roles are defined in _ext/hope_roles.py
# They render as \hope{} and \hopedim{} in LaTeX (PhosphorGreen/ForestDim)
# See that module for usage instructions.
