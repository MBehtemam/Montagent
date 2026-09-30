#!/bin/sh
# Prototype: retries for the shots the filter blocked.
D=$(dirname "$0")
F="$D/flux.py"
S="Cinematic film still, anamorphic widescreen, dramatic lighting, rich contrast, photorealistic, 35mm film grain."
export W=1920 H=1088
python3 "$F" "$D/img/casino.png" "$S An ornate belle-epoque palace on the Mediterranean coast at night, its facade lit in warm gold, wet cobblestone square with reflections, a vintage sports car in front." &
python3 "$F" "$D/img/control.png" "$S A dark futuristic operations room with a giant curved wall of monitors glowing with a red map of the world and orbit lines, people at desks seen as silhouettes." &
python3 "$F" "$D/img/heli.png" "$S A black helicopter flying low over jagged snow-covered mountain peaks at golden hour, wind blowing snow off the ridges, seen from a distance." &
wait
