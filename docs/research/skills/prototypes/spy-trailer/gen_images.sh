#!/bin/sh
# Prototype: the trailer's shots, drawn by FLUX.2-pro. Cinematic stills, no real people.
D=$(dirname "$0")
F="$D/flux.py"
mkdir -p "$D/img"
S="Cinematic film still, anamorphic widescreen, dramatic lighting, rich contrast, shallow depth of field, photorealistic, 35mm film grain."
export W=1920 H=1088
python3 "$F" "$D/img/rooftop.png" "$S A lone man in a black tuxedo seen from behind as a dark silhouette, standing on the edge of a rooftop at night, looking over a glittering city skyline in the rain, backlit by city lights." &
python3 "$F" "$D/img/casino.png" "$S The grand facade of an elegant belle-epoque casino at night, wet cobblestones reflecting warm golden lights, luxury cars parked in front, light rain." &
python3 "$F" "$D/img/cards.png" "$S Close-up of a green baize card table in a casino, a hand in a tuxedo sleeve turning over an ace of spades beside stacks of chips, warm lamp light, dark background." &
python3 "$F" "$D/img/lair.png" "$S A futuristic concrete fortress built into a glacier on a snowy mountain at dusk, its long windows glowing orange, icy blue light, vast and ominous." &
python3 "$F" "$D/img/control.png" "$S A dark high-tech control room with a huge wall of screens showing a glowing red world map and satellite orbits, silhouettes of operators at desks." &
python3 "$F" "$D/img/boat.png" "$S A sleek black speedboat racing across a turquoise sea at sunset, huge white spray arcing behind it, dramatic low angle." &
python3 "$F" "$D/img/heli.png" "$S A black helicopter flying low over jagged snowy mountain peaks at golden hour, snow blowing off the ridges." &
python3 "$F" "$D/img/stairs.png" "$S A grand marble staircase in an opulent palace at night lit by a chandelier, a man in a tuxedo halfway up, seen from far below as a small silhouette." &
python3 "$F" "$D/img/fire.png" "$S A huge orange fireball blooming over a dark harbour at night, reflections on the water, sparks and smoke, seen from a distance." &
wait
ls "$D/img"
