# Release Assets

This folder tracks materials for the v1.0.0 release.

## Structure
- `screenshots/`: application screenshots for store/release pages
- `marketing/`: short description, feature highlights, and release copy
- `sample-scripts/`: example JSON scripts for new users

## Build Output
- Release binary verified at `src-tauri/target/release/nobody.exe`
- Installer bundling (`msi`/`nsis`) is currently blocked in restricted network environments because Tauri needs to download external tooling.
