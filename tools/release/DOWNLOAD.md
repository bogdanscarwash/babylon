# Download and run the Linux preview

Download `babylon-<version>-linux-x86_64.tar.gz` and its adjacent `.sha256`
file from this release. This is the current Michigan economy observer: advance
four-week periods, inspect production and staffing, revisit history, and compare
saved campaigns. Player actions are not implemented yet.

**Prerequisites:** Ubuntu 24.04 x86_64 desktop (glibc 2.39+), Python 3.12+,
a working Vulkan graphics driver, and local Docker Engine with the Docker Compose
plugin. Docker must be usable by your account: `docker info` and
`docker compose version` should succeed. Other distributions are not qualified.

Install any missing desktop libraries:

```sh
sudo apt-get install python3 libpq5 libasound2t64 libudev1 libwayland-client0 \
  libxkbcommon0 libxkbcommon-x11-0 libx11-6 libxi6 libxrandr2 libxcursor1 \
  libvulkan1 mesa-vulkan-drivers
```

Verify the archive with `sha256sum --check <downloaded-file>.tar.gz.sha256`,
extract it, open a terminal in the extracted directory, and run:

```sh
./babylon
```

The first launch downloads and builds the pinned PostgreSQL image and can take
several minutes. It needs internet access and several GB of free disk. Later
launches reuse that image and database. You need no checkout, Rust compiler,
mise, uv, pip installation, raw datasets, or separate asset download. The small
Python launcher dependency set is included; Rust owns simulation and persistence.

| Key | Action |
| --- | --- |
| Enter / Space | Advance one four-week period / play or pause |
| P / M | Production view / map |
| H | History |
| [ / ] | Previous / next committed period |
| Escape | Campaign menu, including Open and Compare |
| N / D | New standard / delayed-delivery campaign from the menu |
| Q | Quit from the menu |

The campaign menu separates Regional proofs from Statewide Michigan. Statewide
offers baseline, freight constraint, packaging shortage, and both constraints.
New campaigns read `content/scenarios/michigan/defines.toml` and the adjacent
pinned statewide source files; existing campaigns retain their saved parameters
and routes. The initial window is 1366 × 768.

For a check without a window, run `./babylon --smoke`. It creates a campaign,
commits one period, restarts the runtime, reopens that state without reading the
defines file, and verifies it through the native reader. This does not test your GPU.

Saves live in the `saves` Docker volume belonging to the isolated Compose project
`babylon-preview-<uid>-<version-with-hyphens>`. The database binds an automatically
selected **loopback-only** port. Logs and the continuation pointer live under
`babylon-preview/<version>/` in your usual XDG state/data directories. Different
preview versions have independent saves; keep the old archive to revisit an
older release. There is no old-save migration.

Quitting allows the runtime to finish its transaction. Stop the database while
retaining saves with `./babylon --stop-database`; the next launch starts it again.
WAL durability stays enabled. Do not remove the Docker volume to retain campaigns.

`release.json` records the version, exact source commit/link, binary hashes, and
launcher dependency hashes. `sha256sum --check SHA256SUMS` verifies all extracted
files. Project, font, audio-rendering and Rust dependency notices are under
`notices/`; Python dependency notices are in their bundled `.dist-info` directories.
The original theme music is included with its author's distribution permission.
Road source attribution is in `content/scenarios/michigan/NOTICE`.

The adjacent `release-provenance.json` links this tested archive to the main
release commit and its GitHub qualification run. The source commit recorded
inside the archive has the same source tree as that main release.
