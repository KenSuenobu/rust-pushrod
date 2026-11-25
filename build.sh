#!/usr/bin/env bash
#
# Builds the application

set -e

# On macOS, make sure the SDL2 libraries are installed through Homebrew and
# that the linker can find them.
if [[ "$(uname)" == "Darwin" ]]; then
  if ! command -v brew >/dev/null 2>&1; then
    echo "Homebrew is required on macOS: https://brew.sh" >&2
    exit 1
  fi

  for pkg in sdl2 sdl2_image sdl2_ttf; do
    if ! brew list --versions "$pkg" >/dev/null 2>&1; then
      echo "Installing $pkg ..."
      brew install "$pkg"
    fi
  done

  export LIBRARY_PATH="${LIBRARY_PATH:+$LIBRARY_PATH:}$(brew --prefix)/lib"

  echo
  echo "I have installed SDL2: you need to add the following command to your local profile:"
  echo "export LIBRARY_PATH=\"\${LIBRARY_PATH:+$LIBRARY_PATH:}\$(brew --prefix)/lib\""
  echo
  echo "Run that command as well, or source your profile, then you can continue after"
  echo "the build terminates successfully."
  echo
fi

cargo build --workspace
cargo doc --workspace --no-deps
