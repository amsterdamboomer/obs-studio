#!/usr/bin/env bash
# Fails when an ELF file in the AppDir needs a newer glibc than the floor:
#   check-glibc-floor.sh <AppDir> [floor]   (floor defaults to 2.28)
# The floor is the oldest glibc the AppImage runs on (obs-rust/obs-studio#926).
# Only the versions a file *requires* count (objdump -p, "Version References"),
# so the bundled glibc itself, which defines every GLIBC_2.x up to its own,
# does not trip the check.
set -euo pipefail

APPDIR="$(realpath "$1")"
FLOOR="${2:-2.28}"
floor_minor="${FLOOR#2.}"

over=0
total=0
while IFS= read -r -d '' f; do
    head -c 4 "$f" 2>/dev/null | grep -q $'^\x7fELF' || continue
    total=$((total + 1))
    need="$(objdump -p "$f" 2>/dev/null |
        sed -n '/^Version References:/,/^$/p' |
        grep -oE 'GLIBC_2\.[0-9]+' | sed 's/GLIBC_2\.//' | sort -n | tail -1 || true)"
    [ -n "$need" ] || continue
    if [ "$need" -gt "$floor_minor" ]; then
        echo "::error::${f#"$APPDIR"/} needs GLIBC_2.$need (floor is $FLOOR)" >&2
        over=$((over + 1))
    fi
done < <(find "$APPDIR" -type f -print0)

[ "$total" -gt 0 ] || {
    echo "::error::check-glibc-floor: no ELF files under $APPDIR" >&2
    exit 1
}
if [ "$over" -gt 0 ]; then
    echo "check-glibc-floor: FAILED, $over of $total ELF files need a glibc newer than $FLOOR" >&2
    exit 1
fi
echo "check-glibc-floor: all $total ELF files run on glibc $FLOOR"
