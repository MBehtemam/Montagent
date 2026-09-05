import json
photos = [
 ("photo-05-intro",0,3018,0,15000),
 ("photo-05",3018,17472,3018,18018),
 ("photo-06",17472,30603,17472,32472),
 ("photo-07",30603,42763,30603,45603),
 ("photo-08",42763,53856,42763,57763),
 ("photo-05-quiz",53856,64016,53856,68856),
 ("photo-05-loop",64016,65216,64016,79016),
]
def ser(v): return json.dumps(v, separators=(",",":"))
scalar_lines, pair_lines = [], []
for pid,s,e,t1,t2 in photos:
    ksc = f'"scale":[{{"t":{t1},"v":1.0,"ease":"linear"}},{{"t":{t2},"v":1.08}}]'
    kpr = f'"scale":[{{"t":{t1},"v":[1.0,1.0],"ease":"linear"}},{{"t":{t2},"v":[1.08,1.08]}}]'
    scalar_lines.append(ksc); pair_lines.append(kpr)
print("SCALAR form (photo-06):"); print(scalar_lines[2])
print("PAIR   form (photo-06):"); print(pair_lines[2])
sc = sum(len(l) for l in scalar_lines); pc = sum(len(l) for l in pair_lines)
print(f"\nTotal chars across 7 lists: scalar={sc}  pair={pc}  overhead={pc-sc} (+{100*(pc-sc)/sc:.0f}%)")
print("Numbers per list: scalar=2 values, pair=4 values; per file: scalar=14, pair=28 (14 of which are duplicates)")
