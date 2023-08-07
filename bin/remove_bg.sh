#!/bin/sh
in_file="$1"
out_file="$2"
color=$( convert "$in_file" -format "%[pixel:p{0,0}]" info:- )
# convert "$in_file" -alpha off -bordercolor $color -border 1 \
#     \( +clone -fuzz 49% -fill none -floodfill +0+0 $color \
#        -alpha extract -geometry 200% -blur 0x0.5 \
#        -morphology erode square:1 -geometry 50% \) \
#     -compose CopyOpacity -composite -shave 1 "$out_file"

convert "$in_file" -fuzz 5% -fill none -draw "matte 0,0 floodfill" \
        -draw "matte 319,160 floodfill" \
        -channel alpha -blur 0x3 -level 50x100% +channel \
        -background transparent -flatten "$out_file"