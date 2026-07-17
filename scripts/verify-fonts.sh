#!/usr/bin/env bash

set -euo pipefail

readonly required_fonts=(
  "Geist"
  "Hina Mincho"
  "Kaisei Decol"
  "Noto Sans JP"
  "Noto Serif JP"
  "Shippori Mincho"
)
readonly available_fonts="$(typst fonts)"

missing_font_count=0
for font_family in "${required_fonts[@]}"; do
  if ! grep --fixed-strings --line-regexp --quiet "${font_family}" <<<"${available_fonts}"; then
    echo "Typstが必要なフォントを認識していない: ${font_family}" >&2
    missing_font_count=$((missing_font_count + 1))
  fi
done

if [[ "${missing_font_count}" -ne 0 ]]; then
  exit 1
fi

printf '%s\n' "${required_fonts[@]}"
