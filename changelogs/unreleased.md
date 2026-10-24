# Unreleased

## Changed

- Linux directory resolution now logs an info message when an XDG variable is unset and a warning when its value
  is not an absolute path, explaining the fallback to the default directory. These platform-dirs logs are enabled
  by default in the desktop app, including the F12 console, unless overridden by `RUST_LOG`.

- Saved window dimensions now use a binary format. Previously saved dimensions are not migrated, so the window
  returns to its default size on the first launch after updating. Subsequent launches restore the newly saved size.
