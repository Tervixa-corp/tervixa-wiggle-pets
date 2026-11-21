# Tervixa Wiggle releases

Artifacts follow the installer layout produced by
`scripts/package-releases.sh`:

- `windows/` — `wiggle.exe` (CLI/TUI), `wiggle-desktop.exe` plus the
  NSIS `.exe` / `.msi` installers when bundled on CI.
- `linux/` — `wiggle`, `wiggle-desktop`, `.deb`, `.rpm`, `.AppImage`.
- `macos/` — `wiggle`, `wiggle-desktop`, `.dmg`.
- `SHA256SUMS.txt` — sha256 of every binary/installer artifact.

Installers bundle `bundled-skills/` and ship `README.md`, `WIGGLE.md`
and `LICENSE` alongside the binaries. Releases are cut by pushing a
`v*` tag; the `release` workflow builds all three OSes and attaches
the artifacts to the GitHub release.
