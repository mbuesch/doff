#!/bin/sh
set -e

basedir="$(dirname "$(realpath "$0")")"
cd "$basedir"

dx build --desktop --release

cp ./target/dx/doff/release/linux/app/doff \
   ./doff
