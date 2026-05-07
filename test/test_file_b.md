# Project Notes

## Overview

This project implements a side-by-side diff viewer with editing support.
It supports syntax highlighting and inline word diffs.
Files can be opened via the toolbar or dragged onto either panel.

## Features

- Side-by-side comparison
- Inline word-level highlighting
- Copy lines left to right
- Copy lines right to left
- Block copy (copy entire change group at once)
- Ignore whitespace option
- Light and dark themes
- Editable panels

## Usage

Open two files using the toolbar buttons or the panel Open buttons.
You can also drag and drop files directly onto the panels.
Use the arrow buttons to copy individual lines between sides.
Click inside a panel to start editing the content directly.

## Known Issues

Very large files may cause slow initial rendering.
Unicode filenames are supported on Linux and macOS.

## Changelog

### v0.2.0

- Added inline editing support
- Added block copy buttons
- File dialogs now default to current working directory

### v0.1.0

- Initial release
- Basic diff functionality
- Light/dark theme toggle
