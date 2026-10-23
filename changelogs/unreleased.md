# Unreleased

## Changed

- Saved window dimensions now use a binary format. Previously saved dimensions are not migrated, so the window
  returns to its default size on the first launch after updating. Subsequent launches restore the newly saved size.
