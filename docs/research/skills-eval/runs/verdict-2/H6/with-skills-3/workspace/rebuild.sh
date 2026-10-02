#!/bin/sh
cd "$(dirname "$0")"
python3 .claude/skills/montagent-character/scripts/bake_rig.py hoot.base.json character/rig.json hoot.spec.json > hoot.json 2>bake.log || { cat bake.log; exit 1; }
montagent fmt hoot.json >/dev/null
montagent validate hoot.json | grep -v "^  character\|probed\|^CACHE" 
