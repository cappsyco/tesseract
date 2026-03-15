#!/bin/bash

[ -f $(dirname "$0")/screen_L.png ] && [ -f $(dirname "$0")/screen_R.png ] || { echo "Missing images"; exit 1; }

WL=$(identify -format "%w" screen_L.png)
HL=$(identify -format "%h" screen_L.png)

WR=$(identify -format "%w" screen_R.png)
HR=$(identify -format "%h" screen_R.png)

[ "$WL" -eq "$WR" ] && [ "$HL" -eq "$HR" ] || { echo "Size mismatch"; exit 1; }

magick screen_L.png \( screen_R.png \( -size ${WL}x${HL} xc:black -fill white -draw "polygon 0,${HL} ${WL},${HL} ${WL},0" \) -alpha off -compose copy_opacity -composite \) -compose over -composite screenshot.png
