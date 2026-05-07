#!/bin/sh
set -e

basedir="$(dirname "$(realpath "$0")")"
cd "$basedir"

SRC="$basedir/logo.svg"
for size in 32 48 64 128; do
    name="$(basename "$SRC" .svg)"
    DST="$basedir/${name}-${size}x${size}.png"
    echo "Generating $DST ..."
    convert "$SRC" -resize "${size}x${size}" "$DST"
done
