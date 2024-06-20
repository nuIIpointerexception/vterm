#!/bin/bash

###
# @desc Copy images to the home directory
###

SOURCE_DIR="assets/images"
DEST_DIR="$HOME/.vterm/assets/images"

mkdir -p "$DEST_DIR"

if diff -r "$SOURCE_DIR" "$DEST_DIR" > /dev/null; then
    echo "Images are already up to date."
else
    rm -rf "$DEST_DIR"/*
    cp -r "$SOURCE_DIR"/* "$DEST_DIR"

    echo "Images have been successfully copied to $DEST_DIR"
fi
