#!/bin/bash

# Get the directory of the script
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Determine Inkscape command
if command -v inkscape >/dev/null 2>&1; then
    INKSCAPE="inkscape"
else
    # Try the Mac path
    if [ -x /Applications/Inkscape.app/Contents/MacOS/inkscape ]; then
        INKSCAPE="/Applications/Inkscape.app/Contents/MacOS/inkscape"
    else
        echo "Error: Inkscape command not found. Ensure you have Inkscape installed."
        exit 1
    fi
fi

# Remove and recreate the output directory
rm -rf "$DIR/output"
mkdir -p "$DIR/output"

# Enable nullglob so that the array is empty if no files are found
shopt -s nullglob

# Get all SVG files
svg_files=("$DIR/input/"*.svg)

# Check if there are any SVG files
if [ ${#svg_files[@]} -eq 0 ]; then
    echo "No SVG files found in $DIR/input/"
    exit 1
fi

# Process each SVG file
for svg_file in "${svg_files[@]}"; do
    # Get the base filename without extension
    filename=$(basename "$svg_file" .svg)
    # Output file path
    png_file="$DIR/output/$filename.png"
    # Run Inkscape to convert SVG to PNG with size 512x512
    echo "Converting $svg_file to $png_file"
    "$INKSCAPE" --export-type=png --export-filename="$png_file" --export-width=512 --export-height=512 "$svg_file"
    if [ $? -ne 0 ]; then
        echo "Error converting $svg_file"
    fi
done
