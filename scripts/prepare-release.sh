#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.1.0}"
if [[ "$VERSION" == v* ]]; then
  VERSION_TAG="$VERSION"
else
  VERSION_TAG="v$VERSION"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RELEASE_ROOT="$REPO_ROOT/release-assets"
EXTENSION_DIR="$REPO_ROOT/.vscode-extension"
PLUGIN_DIR="$REPO_ROOT/jetbrains-plugin"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Required command not found: $1" >&2
    exit 1
  fi
}

zip_dir() {
  local source_dir="$1"
  local output_zip="$2"

  if command -v zip >/dev/null 2>&1; then
    pushd "$source_dir" >/dev/null
    zip -q -r "$output_zip" .
    popd >/dev/null
    return
  fi

  if command -v powershell.exe >/dev/null 2>&1; then
    local source_win output_win
    source_win="$(cygpath -w "$source_dir")"
    output_win="$(cygpath -w "$output_zip")"
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Compress-Archive -Path '${source_win}\\*' -DestinationPath '${output_win}' -Force" >/dev/null
    return
  fi

  echo "Neither zip nor powershell.exe is available for archive creation." >&2
  exit 1
}

echo "Preparing release assets for $VERSION_TAG"

require_command cargo
require_command npm

rm -rf "$RELEASE_ROOT"
mkdir -p "$RELEASE_ROOT/winportkill-cli" "$RELEASE_ROOT/winportkill-gui"

pushd "$REPO_ROOT" >/dev/null
cargo build --release -p winportkill
cargo build --release -p winportkill-gui
cp "$REPO_ROOT/target/release/winportkill.exe" "$RELEASE_ROOT/winportkill-cli/"
cp "$REPO_ROOT/target/release/winportkill-gui.exe" "$RELEASE_ROOT/winportkill-gui/"
popd >/dev/null

zip_dir "$RELEASE_ROOT/winportkill-cli" "$RELEASE_ROOT/winportkill-windows-x64-$VERSION_TAG.zip"
zip_dir "$RELEASE_ROOT/winportkill-gui" "$RELEASE_ROOT/winportkill-gui-windows-x64-$VERSION_TAG.zip"

pushd "$EXTENSION_DIR" >/dev/null
npm run build
./node_modules/.bin/vsce package --allow-missing-repository --out "$RELEASE_ROOT/winportkill-vscode-$VERSION_TAG.vsix"
popd >/dev/null

if [[ -f "$PLUGIN_DIR/gradlew.bat" ]]; then
  pushd "$PLUGIN_DIR" >/dev/null
  ./gradlew.bat buildPlugin
  plugin_zip="$(find "$PLUGIN_DIR/build/distributions" -maxdepth 1 -type f -name '*.zip' | head -n 1)"
  if [[ -n "${plugin_zip:-}" ]]; then
    cp "$plugin_zip" "$RELEASE_ROOT/winportkill-jetbrains-$VERSION_TAG.zip"
  fi
  popd >/dev/null
elif [[ -f "$PLUGIN_DIR/gradlew" ]]; then
  pushd "$PLUGIN_DIR" >/dev/null
  ./gradlew buildPlugin
  plugin_zip="$(find "$PLUGIN_DIR/build/distributions" -maxdepth 1 -type f -name '*.zip' | head -n 1)"
  if [[ -n "${plugin_zip:-}" ]]; then
    cp "$plugin_zip" "$RELEASE_ROOT/winportkill-jetbrains-$VERSION_TAG.zip"
  fi
  popd >/dev/null
elif command -v gradle >/dev/null 2>&1; then
  pushd "$PLUGIN_DIR" >/dev/null
  gradle buildPlugin
  plugin_zip="$(find "$PLUGIN_DIR/build/distributions" -maxdepth 1 -type f -name '*.zip' | head -n 1)"
  if [[ -n "${plugin_zip:-}" ]]; then
    cp "$plugin_zip" "$RELEASE_ROOT/winportkill-jetbrains-$VERSION_TAG.zip"
  fi
  popd >/dev/null
else
  echo "WARNING: JetBrains Gradle wrapper and global Gradle not found. Skipping plugin package." >&2
fi

echo
echo "Release assets prepared in: $RELEASE_ROOT"
find "$RELEASE_ROOT" -maxdepth 1 -type f -printf "%f\n" | sort
