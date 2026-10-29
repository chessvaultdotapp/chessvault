# Unreleased

## Added

- Added an Apple Silicon macOS app bundle, distributed as a ZIP in development releases.
  Extract `ChessVault.app` and move it to Applications if desired. The bundle includes a native Finder and Dock icon.
  Intel Macs are not supported by this download.
- Added native macOS directories for saved window state and logs. State defaults to
  `~/Library/Application Support/chessvault`, or an absolute `XDG_STATE_HOME` override with `chessvault` appended.
  Logs use `~/Library/Logs/chessvault`. User and system data searches honor `XDG_DATA_HOME` and `XDG_DATA_DIRS`,
  defaulting to `~/Library/Application Support` and `/Library/Application Support`, respectively.

## Known limitations

- macOS downloads are ad-hoc signed, not Developer ID signed or notarized, so Gatekeeper may block launch.
  macOS 13.0 is the provisional deployment target, not a verified compatibility guarantee.
