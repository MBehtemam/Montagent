#!/bin/sh
set -e
python3 build.py
python3 .claude/skills/montagent-character/scripts/bake_rig.py hoot.base.json character/rig.json hoot.spec.json > hoot.montagent.json 2>bake.log
montagent fmt hoot.montagent.json >/dev/null
montagent validate hoot.montagent.json --verbose 2>&1 | grep -v 'R-EASE-INERT\|N-TRACK-GAP' | grep -B1 -A3 '^\(error\|review\|unchecked\)' | grep -v 'ease=.*describes no motion' || true
