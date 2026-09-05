#!/bin/sh
# -*- coding: utf-8 -*-

basedir="$(dirname "$(realpath "$0")")"

. "$basedir/scripts/lib.sh"

install_entry_checks()
{
    [ -x "$bin" ] || die "doff is not built! Run ./build.sh"
    [ "$(id -u)" = "0" ] || die "Must be root to install doff."
}

install_dirs()
{
    do_install \
        -o root -g root -m 0755 \
        -d /opt/doff/bin

    do_install \
        -o root -g root -m 0755 \
        -d /opt/doff/etc

    do_install \
        -o root -g root -m 0755 \
        -d /opt/doff/etc/doff
}

install_doff()
{
    do_install \
        -o root -g root -m 0755 \
        "$bin" \
        /opt/doff/bin/
}

bin="$basedir/doff"

install_entry_checks
install_dirs
install_doff
