# Project Notes

## Overview

This project implements a side-by-side diff viewer.
It supports syntax highlighting and inline word diffs.
Files can be opened via the toolbar or dragged onto the panel.

## Features

- Side-by-side comparison
- Inline word-level highlighting
- Copy lines left to right
- Copy lines right to left
- Ignore whitespace option
- Light and dark themes

## Usage

Open two files using the toolbar buttons.
You can also drag and drop files directly onto the panels.
Use the arrow buttons to copy individual lines between sides.

## Known Issues

The scroll position resets when files are reloaded.
Very large files may cause slow initial rendering.
Unicode filenames are supported on Linux and macOS.

## Changelog

### v0.1.0

- Initial release
- Basic diff functionality
- Light/dark theme toggle
