#!/bin/sh
# hoot.json is generated: edit build.py (scene + owl spec) or post.py, then run this.
set -e
cd "$(dirname "$0")"
python3 build.py
python3 .claude/skills/montagent-character/scripts/bake_rig.py hoot.base.json character/rig.json hoot.spec.json > hoot.baked.json
python3 post.py hoot.baked.json hoot.json
rm hoot.base.json hoot.baked.json
montagent fmt hoot.json
