# Contributing to desktop-toolkit

## What this repo is

`desktop-toolkit` is framework code.  Every change here is immediately consumed by
**all downstream tools** (`Transmittal-Builder`, `Drawing-List-Manager`, and any
future chamber-19 desktop tools) the next time they bump their pin.

**Think carefully before merging here.**

---

## Workflow

1. **Open a PR** against `main` with a clear description of what changes and why.
2. **CI must be green** before merging.
3. **Merge** via squash or merge commit (no force-push to `main`).
4. **Tag** a new SemVer release manually after merge:
   ```bash
   git tag -a v1.2.3 -m "v1.2.3 — <one-line summary>"
   git push origin v1.2.3
   ```
5. **Consumer tools bump their pin** in a follow-up PR:
   - Python: update the `@v1.2.3` ref in `pyproject.toml`/`requirements.txt`
   - JS: update the version in `package.json`

---

## Breaking changes

A change is breaking if it:
- Renames or removes a public function, class, or module
- Changes a function signature in a non-backwards-compatible way
- Modifies template placeholder names (`${TOOL_*}`, `${PRODUCT_NAME}`)

Breaking changes **must** bump the major version (`v1.x.x` → `v2.0.0`).

Add a migration note to `CHANGELOG.md` explaining what consumers need to change.

---

## Adding new utilities

- **Python**: add to `python/chamber19_desktop_toolkit/utils/` and export from
  `python/chamber19_desktop_toolkit/utils/__init__.py`.
- **JS/React**: add to `js/packages/desktop-toolkit/src/` and add an entry to
  the `exports` field in `js/packages/desktop-toolkit/package.json`.

---

## Further reading

For the full multi-repo overview (how Transmittal-Builder was refactored to
consume this framework) see:
[Transmittal Builder CONTRIBUTING.md](https://github.com/chamber-19/transmittal-builder/blob/main/CONTRIBUTING.md)
