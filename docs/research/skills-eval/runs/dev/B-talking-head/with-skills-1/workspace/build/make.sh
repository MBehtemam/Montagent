set -e
cd "$(dirname "$0")/.."
python3 build/build.py
python3 .claude/skills/montagent-footage/scripts/captions.py social.montagent.json presenter/take-1.words.json build/captions.spec.json > build/out.json
mv build/out.json social.montagent.json
montagent fmt social.montagent.json >/dev/null
montagent validate social.montagent.json --verbose
