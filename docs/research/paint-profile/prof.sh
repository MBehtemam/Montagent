#!/bin/sh
# usage: prof.sh <project.json> <tag>  — samples montagent (whole run) and its ffmpeg child
S=${PAINT_PROFILE_OUT:?set PAINT_PROFILE_OUT to a scratch dir holding target/, bench/ and p/}
cd "$(dirname "$1")"
$S/target/release/montagent render "$(basename "$1")" > /dev/null 2>&1 &
PID=$!
sleep 1
sample $PID 1000 1 -mayDie -file $S/p/$2.sample.txt > /dev/null 2>&1 &
# ffmpeg cpu at the end: poll its cputime until it exits
FF=$(pgrep -P $PID ffmpeg | head -1)
last=""
while kill -0 $PID 2>/dev/null; do
  [ -n "$FF" ] && t=$(ps -o time= -p $FF 2>/dev/null) && [ -n "$t" ] && last=$t
  sleep 2
done
echo "$2 ffmpeg cputime(last seen)=$last" | tee -a $S/p/results.txt
wait
