#!/usr/bin/env bash

set -euo pipefail

readonly google_fonts_repository="https://github.com/google/fonts.git"
readonly google_fonts_revision="${GOOGLE_FONTS_REVISION:?GOOGLE_FONTS_REVISION must be set}"
readonly font_install_dir="${BLOG_FONT_INSTALL_DIR:-${XDG_DATA_HOME:-${HOME}/.local/share}/fonts/blog}"
readonly revision_marker="${font_install_dir}/.google-fonts-revision"
readonly expected_font_file_count=14

font_file_count() {
  find "${font_install_dir}" -maxdepth 1 -type f -name '*.ttf' | wc -l | tr -d ' '
}

for command_name in git fc-cache install find; do
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    echo "required command is unavailable: ${command_name}" >&2
    exit 1
  fi
done

mkdir -p "${font_install_dir}"

if [[ -f "${revision_marker}" ]] \
  && [[ "$(<"${revision_marker}")" == "${google_fonts_revision}" ]] \
  && [[ "$(font_file_count)" -eq "${expected_font_file_count}" ]]; then
  fc-cache --force "${font_install_dir}" >/dev/null
  echo "font cache already contains revision ${google_fonts_revision}"
  exit 0
fi

readonly temporary_root="$(mktemp -d)"
readonly checkout_dir="${temporary_root}/google-fonts"

cleanup() {
  rm -rf "${temporary_root}"
}
trap cleanup EXIT

git init --quiet "${checkout_dir}"
git -C "${checkout_dir}" remote add origin "${google_fonts_repository}"
git -C "${checkout_dir}" sparse-checkout init --cone
git -C "${checkout_dir}" sparse-checkout set \
  ofl/geist \
  ofl/hinamincho \
  ofl/kaiseidecol \
  ofl/notosansjp \
  ofl/notoserifjp \
  ofl/shipporimincho \
  ofl/mplus1code
git -C "${checkout_dir}" fetch --quiet --depth=1 origin "${google_fonts_revision}"
git -C "${checkout_dir}" checkout --quiet --detach FETCH_HEAD

while IFS= read -r -d '' font_file; do
  install -m 0644 "${font_file}" "${font_install_dir}/"
done < <(find "${checkout_dir}/ofl" -type f -name '*.ttf' -print0)

readonly installed_font_count="$(font_file_count)"
if [[ "${installed_font_count}" -ne "${expected_font_file_count}" ]]; then
  echo "expected ${expected_font_file_count} font files, but installed ${installed_font_count}" >&2
  exit 1
fi

printf '%s\n' "${google_fonts_revision}" >"${revision_marker}"
fc-cache --force "${font_install_dir}" >/dev/null
echo "installed ${installed_font_count} font files into ${font_install_dir}"
