#!/usr/bin/env python3
"""Build project.json for Brief H3: the set, laptop, crate and tag, then bake the owl rig
from owl.spec.json with the character skill's bake_rig.py, then lift the thinking arm
above the head while its hand is at the beak."""
import json, math, subprocess, sys

FPS = 30
VOICE_AT = 1000            # the line starts at 1 s
BAKE = '.claude/skills/montagent-character/scripts/bake_rig.py'


def F(n):
    """The instant frame n is drawn at."""
    return math.ceil(n * 1000 / FPS)


def snap(ms):
    """The first drawn instant at or after ms."""
    return F(math.ceil(ms * FPS / 1000 - 1e-9))


words = json.load(open('character/voice/line-3.json'))['words']
W = {w['word'].strip('.!…'): (w['start'] + VOICE_AT, w['end'] + VOICE_AT) for w in words}

INK, PAPER, SIGNAL = '#101418', '#F5F0E6', '#FF5A36'
OUTLINE = '#3A2A22'
END = 8000

# Laptop: 708x566 drawn at 566x453; its screen is x 120-587, y 41-312 of the image.
LX, LY, LW, LH = 818, 415, 566, 453
SX, SY, SW, SH = 914, 448, 374, 217          # the screen, rounded inwards
SCX, SCY = SX + SW // 2, SY + SH // 2

# Tag above the laptop.
TAG_X, TAG_Y = SCX, 338
POP = [0.34, 1.56, 0.64, 1]

t_two = snap(W['two'][0])        # tag pops in on "two"
t_fixed = snap(W['Fixed'][0])    # tag reads 0 frames from "Fixed!"
t_mark = snap(W['beat'][1] + 60)  # screen changes just after "beat"


def el(**kw):
    return kw


tracks = []


def track(name, layer, *elements):
    tracks.append({'name': name, 'layer': layer, 'elements': list(elements)})


track('set', 0, el(id='set', type='image', start=0, end=END, source='character/study.png',
                   x=0, y=0, origin='top-left', width=1920, height=1080, fit='literal'))

# Soft contact shadows on the floor.
track('shadow-crate', 1, el(id='shadow-crate', type='ellipse', start=0, end=END, x=1100, y=954,
                            origin='center', width=680, height=34, fill='#6B4A2C55'))
track('shadow-owl', 2, el(id='shadow-owl', type='ellipse', start=0, end=END, x=660, y=990,
                          origin='center', width=300, height=40, fill='#6B4A2C66',
                          scale=[{'t': snap(5600), 'v': [1.0, 1.0]},
                                 {'t': snap(5833), 'v': [0.72, 0.72], 'ease': 'ease-out'},
                                 {'t': snap(6067), 'v': [1.0, 1.0], 'ease': 'ease-in'}]))

# A plain low crate in the set's flat style: a box with end posts and one plank seam.
CX, CY, CW, CH = 800, 862, 600, 90
track('crate', 3, el(id='crate', type='rect', start=0, end=END, x=CX, y=CY, origin='top-left',
                     width=CW, height=CH, fill='#C8925E', stroke=OUTLINE, stroke_width=6, radius=8))
track('crate-seam', 4, el(id='crate-seam', type='rect', start=0, end=END, x=CX + 46, y=CY + 42,
                          origin='top-left', width=CW - 92, height=5, fill='#8A5A36'))
track('crate-post-l', 5, el(id='crate-post-l', type='rect', start=0, end=END, x=CX, y=CY,
                            origin='top-left', width=52, height=CH, fill='#B07A48', stroke=OUTLINE,
                            stroke_width=6, radius=8))
track('crate-post-r', 5, el(id='crate-post-r', type='rect', start=0, end=END, x=CX + CW - 52, y=CY,
                            origin='top-left', width=52, height=CH, fill='#B07A48', stroke=OUTLINE,
                            stroke_width=6, radius=8))

# The laptop resting on the crate, and what its screen shows.
track('laptop', 6, el(id='laptop', type='image', start=0, end=END, source='character/laptop.png',
                      x=LX, y=LY, origin='top-left', width=LW, height=LH, fit='literal'))
track('screen-ground', 7,
      el(id='screen-session', type='image', start=0, end=t_mark, source='stills/session-02.png',
         x=SX - 5, y=SY, origin='top-left', width=385, height=217, fit='cover', clip=[SX, SY, SW, SH]),
      el(id='screen-paper', type='rect', start=t_mark, end=END, x=SX, y=SY, origin='top-left',
         width=SW, height=SH, fill=PAPER))
track('screen-mark', 8,
      el(id='screen-mark', type='image', start=t_mark, end=END, source='brand/mark.png',
         x=SCX, y=SY + 22 + 62, origin='center', width=124, height=124, fit='literal',
         scale=[{'t': t_mark, 'v': [0.8, 0.8]}, {'t': t_mark + 234, 'v': [1.0, 1.0], 'ease': POP}]))
track('screen-word', 8,
      el(id='screen-word', type='image', start=t_mark, end=END, source='brand/wordmark.png',
         x=SCX, y=SY + 22 + 124 + 14 + 18, origin='center', width=172, height=36, fit='literal',
         opacity=[{'t': t_mark, 'v': 0.0}, {'t': t_mark + 200, 'v': 1.0, 'ease': 'ease-out'}]))

# The tag: "+2 frames" pops in on "two", reads "0 frames" from "Fixed!".
def tag(key, text, start, end, text_w, scale):
    pill_w = text_w + 76
    return [
        el(id=f'tag-{key}-pill', type='rect', start=start, end=end, x=TAG_X, y=TAG_Y, origin='center',
           width=pill_w, height=104, scale=scale, fill=SIGNAL, stroke=INK, stroke_width=6, radius=52),
        el(id=f'tag-{key}-text', type='text', start=start, end=end, x=TAG_X, y=TAG_Y, origin='center',
           width=text_w, height=72, scale=scale, font='bold', size=60, color=INK, align='center',
           caption=False, runs=[{'text': text}]),
    ]


pop_in = [{'t': t_two, 'v': [0.0, 0.0]}, {'t': t_two + 300, 'v': [1.0, 1.0], 'ease': POP}]
re_pop = [{'t': t_fixed, 'v': [0.82, 0.82]}, {'t': t_fixed + 234, 'v': [1.0, 1.0], 'ease': POP}]
late_pill, late_text = tag('late', '+2 frames', t_two, t_fixed, 300, pop_in)
fixed_pill, fixed_text = tag('fixed', '0 frames', t_fixed, END, 264, re_pop)
track('tag-pill', 40, late_pill, fixed_pill)
track('tag-text', 41, late_text, fixed_text)

track('voice', 0, el(id='voice', type='audio', start=VOICE_AT, end=VOICE_AT + 6672,
                     source='character/voice/line-3.wav', source_start=0, source_end=6672, volume=1.0))

project = {
    'frame': {'width': 1920, 'height': 1080},
    'fps': FPS,
    'background': '#101418',
    'duration': END,
    'output': 'deliverable.mp4',
    'fonts': {'bold': [{'file': 'fonts/Inter-Bold.ttf'}]},
    'fontVendor': json.load(open('project.json'))['fontVendor'],
    'tracks': tracks,
}
json.dump(project, open('base.json', 'w'), indent=2)

baked = subprocess.run([sys.executable, BAKE, 'base.json', 'character/rig.json', 'owl.spec.json'],
                       capture_output=True, text=True)
sys.stderr.write(baked.stderr)
if baked.returncode:
    sys.exit(baked.returncode)
project = json.loads(baked.stdout)

# The head draws over the arms, so a hand at the beak would hide behind it. While the
# left hand is up at the beak, lift that arm above the head and face (upper arm still
# over forearm). It switches while the arm hangs at the side, so the switch is unseen.
spec = json.load(open('owl.spec.json'))
lift_from, lift_to = 700, 4400

top = max(e.get('layer', t['layer']) if isinstance(e.get('layer', t['layer']), int) else t['layer']
          for t in project['tracks'] if t['name'].startswith('owl-') for e in t['elements'])
for t in project['tracks']:
    if t['name'] in ('owl-forearm_left', 'owl-upper_arm_left'):
        lift = top + (1 if t['name'] == 'owl-forearm_left' else 2)
        [e] = t['elements']
        a, b, c = dict(e), dict(e), dict(e)
        a['end'] = lift_from
        b['id'], b['start'], b['end'], b['layer'] = e['id'] + '-lifted', lift_from, lift_to, lift
        c['id'], c['start'] = e['id'] + '-after', lift_to
        t['elements'] = [a, b, c]

json.dump(project, open('project.json', 'w'), indent=2)
print('wrote project.json; tag pops', t_two, 'fixed', t_fixed, 'mark', t_mark, file=sys.stderr)
