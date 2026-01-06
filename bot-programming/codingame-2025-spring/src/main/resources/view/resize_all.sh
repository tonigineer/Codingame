#!/bin/bash

# Directory containing the subfolders with PNGs
main_dir="/home/jpn/Pictures/SC2025 -SuperSoaker/Assets/Animes_(30%)/"

# Check if the directory exists
if [ ! -d "$main_dir" ]; then
    echo "Directory $main_dir does not exist."
    exit 1
fi

# Loop through each subfolder
find "$main_dir" -type d | while read -r subfolder; do
    # Resize each PNG file in the subfolder to 30% of its original size
    find "$subfolder" -maxdepth 1 -type f -name "*.png" | while read -r img; do
        echo "Resizing $img"
        mogrify -resize 30% "$img"
    done
done

echo "All PNG images have been resized to 30% of their original size."
