set -e
python3 build.py
python3 .claude/skills/montagent-character/scripts/bake_rig.py scene.montagent.json character/rig.json owl.spec.json > hoot.montagent.json
montagent fmt hoot.montagent.json > /dev/null
montagent validate hoot.montagent.json
