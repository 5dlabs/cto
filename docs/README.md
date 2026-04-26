# `docs/` layout

Material is grouped under **`docs/YYYY-MM/`** by the **month and year of the file’s first addition to this repository** (see `git log --diff-filter=A` for a given path). The path under the month folder matches the old layout, for example `docs/2026-03/intake-observer.md` for what used to be `docs/intake-observer.md`.

To repair relative links after moves, use `scripts/2026-04/repair-doc-links.py` from the repo root.
