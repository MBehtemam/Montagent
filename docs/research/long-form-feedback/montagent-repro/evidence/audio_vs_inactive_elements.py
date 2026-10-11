import json,os,subprocess,time
P=json.load(open('project-part1-A.json')); AT,LEN=1500000,3000
def active(e): return e.get('start',0)<=AT+LEN and e.get('end',10**12)>=AT
def mk(noaudio=False, onlyactive=False, dropart=False):
    q=json.loads(json.dumps(P))
    for t in q['tracks']:
        t['elements']=[e for e in t['elements'] if not (noaudio and e['type']=='audio') and not (onlyactive and not active(e)) and not (dropart and e['id'].startswith('art-'))]
    q['tracks']=[t for t in q['tracks'] if t['elements']]; return q
tests=[('no audio, all inactive kept: everything',mk(noaudio=True)),('no audio, all inactive kept: drop art',mk(noaudio=True,dropart=True)),
 ('audio kept, only active elements: everything',mk(onlyactive=True)),('audio kept, only active elements: drop art',mk(onlyactive=True,dropart=True)),
 ('no audio + only active (= the kit): everything',mk(noaudio=True,onlyactive=True)),('no audio + only active (= the kit): drop art',mk(noaudio=True,onlyactive=True,dropart=True))]
for i,(l,q) in enumerate(tests):
    json.dump(q,open('_c.json','w')); t=time.time()
    r=subprocess.run(['montagent','render','_c.json','--from',str(AT),'--to',str(AT+LEN),'--output',f'out/_d{i}.mp4'],capture_output=True,text=True); el=time.time()-t
    print(f'{l:52s} rc={r.returncode} {el/90:.4f} s/frame',flush=True); os.remove(f'out/_d{i}.mp4')
os.remove('_c.json'); print('done')
