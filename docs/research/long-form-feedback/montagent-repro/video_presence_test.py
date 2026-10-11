"""Decisive test: does a video element that is NOT active in the rendered range slow every frame? Run from this folder: python3 video_presence_test.py (about 30 s). Reference result in evidence/video_presence_test-reference.log."""
import json,os,subprocess,time
P=json.load(open('project.json')); AT,LEN=1500000,3000
art=[e for t in P['tracks'] for e in t['elements'] if e['id'].startswith('art-')][0]
def base():
    q=json.loads(json.dumps(P))
    for t in q['tracks']: t['elements']=[e for e in t['elements'] if not e['id'].startswith('art-')]
    q['tracks']=[t for t in q['tracks'] if t['elements']]; return q
def with_video(start,end,w=740,h=416):
    q=base(); e=json.loads(json.dumps(art)); e['id']='far-video'; e['start']=start; e['end']=end
    for k in ('opacity','effects','blend','scale','rotation','overrun'): e.pop(k,None)
    e['source_start']=0; e['source_end']=end-start
    e['width']=w; e['height']=h
    q['tracks'].append({'name':'far','layer':50,'elements':[e]}); return q
def with_image(start,end):
    q=base(); q['tracks'].append({'name':'farimg','layer':50,'elements':[{'id':'far-img','type':'image','start':start,'end':end,'source':'assets/table-raw.png','x':0,'y':0,'origin':'top-left','width':300,'height':170,'fit':'contain'}]}); return q
tests=[('art dropped, nothing else (fast reference)',base()),
 ('+ a small VIDEO element far outside the slice (2,000 s)',with_video(2000000,2010000)),
 ('+ a small IMAGE element far outside the slice (control)',with_image(2000000,2010000)),
 ('+ a small VIDEO element far outside the slice (again)',with_video(2000000,2010000)),
 ('art dropped, nothing else (again)',base())]
for i,(l,q) in enumerate(tests):
    q['output']='out/x.mp4'; json.dump(q,open('_c.json','w')); t=time.time()
    r=subprocess.run(['montagent','render','_c.json','--from',str(AT),'--to',str(AT+LEN),'--output',f'out/_e{i}.mp4'],capture_output=True,text=True); el=time.time()-t
    print(f'{l:62s} rc={r.returncode} {el/90:.4f} s/frame',('' if r.returncode==0 else (r.stdout+r.stderr)[-150:]),flush=True)
    if os.path.exists(f'out/_e{i}.mp4'): os.remove(f'out/_e{i}.mp4')
os.remove('_c.json'); print('done')
