"""The trailer's timeline, in ms, shared by score.py and scene.py."""

DURATION = 36000

COLD = (0, 3200)               # black, ticking; gun-barrel dots, then the rifling iris
DOTS = [300, 620, 940, 1260]   # each dot enters
IRIS = 1700                    # the rifling opens
BOOM = 3200                    # flash; the iris blows open onto the rooftop

SHOTS = [  # (name, image, start, end)
    ("s1", "rooftop", 3200, 5000),
    ("s2", "stairs", 5000, 6800),
    ("s3", "cards", 7900, 10900),
    ("s4", "lair", 12000, 16200),
    ("s6", "rooftop", 17400, 19700),
    ("s7", "fire", 27400, 30400),
]
CARDS = [  # (id, text, start, end, style)
    ("c1", "EVERY SECRET", 6800, 7900, "track"),
    ("c2", "HAS A PRICE", 10900, 12000, "slam"),
    ("c3", "NO SECOND CHANCES", 19700, 20800, "split"),
]
HUD = (16200, 17400)
MONTAGE = (20800, 24800)
MONTAGE_SHOTS = ["boat", "fire", "lair", "stairs", "cards", "boat", "fire", "rooftop"]
DROP = (24800, 27400)          # black, ticking: "This winter..."
HIT = 27400                    # the big hit into the fireball
TITLE = (30400, 35500)
COMING = 33000
FADE = (35000, 35600)

VO = {  # line -> timeline start
    "n1": 3500, "n2": 7950, "villain": 12300, "agent": 17600, "n3": 25000, "n4": 27600,
}
HITS = [BOOM, 6800, 10900, 19700, HIT, TITLE[0]]
BIG_HITS = [HIT, TITLE[0]]
