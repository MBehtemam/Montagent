import json,subprocess,sys
S='.claude/skills/montagent-footage/scripts/captions.py'
def run(src,words,spec,dst):
    out=subprocess.run(['python3',S,src,words,spec],capture_output=True,text=True)
    sys.stderr.write(out.stderr)
    if out.returncode: sys.exit(1)
    open(dst,'w').write(out.stdout)
run('base.montagent.json','build/words.all.json','build/spec.duck.json','_p1.json')
run('_p1.json','build/words.a.json','build/spec.a.json','_p2.json')
run('_p2.json','build/words.b.json','build/spec.b.json','_p3.json')
run('_p3.json','build/words.payoff.json','build/spec.payoff.json','_p4.json')
p=json.load(open('_p4.json'))
p['output']='deliverable.mp4'
T={t['name']:t for t in p['tracks']}
p['tracks']=[t for t in p['tracks'] if t['name'] not in('captions','captions-bg')]
E_={e['id']:e for t in p['tracks'] for e in t['elements']}
# hand-off cap-a -> cap-b
bstart=E_['cap-b-01']['start']
for i in ['cap-a-02','cap-a-bg-02']: E_[i]['end']=bstart
FADE0,FADE1=9640,9880
EASE=[0.65,0,0.35,1]
POP=[0.34,1.56,0.64,1]
# payoff: solid pill, pop in
for k in ['01','02']:
    tx,bg=E_['payoff-'+k],E_['payoff-bg-'+k]
    bg['fill']='#101418'
    s=tx['start']
    for e in (tx,bg):
        e['scale']=[{"t":s,"v":[0.85,0.85]},{"t":s+240,"v":[1.0,1.0],"ease":POP}]
for e in (E_['payoff-02'],E_['payoff-bg-02']):
    e['end']=FADE1
    e['opacity']=[{"t":FADE0,"v":1.0},{"t":FADE1-40,"v":0.0,"ease":"ease-in"}]
# presenter: push-in for the payoff, fade out after the last word
pr=E_['presenter']
pr['end']=9960; pr['overrun']='hold'
s0=E_['payoff-01']['start']
pr['scale']=[{"t":s0-80,"v":[1.0,1.0]},{"t":s0+480,"v":[1.06,1.06],"ease":EASE}]
pr['opacity']=[{"t":FADE0,"v":1.0},{"t":9920,"v":0.0,"ease":"ease-in"}]
# brand: header lockup that becomes the end card
W,H=760,176; hs=360/760
lk={"id":"lockup","type":"image","start":0,"end":12000,"source":"brand/lockup.png","x":540,"y":[{"t":0,"v":92},{"t":9880,"v":92,"ease":"linear"},{"t":10480,"v":675,"ease":EASE}],
    "origin":"center","width":W,"height":H,"fit":"contain",
    "scale":[{"t":0,"v":[hs,hs]},{"t":9880,"v":[hs,hs],"ease":"linear"},{"t":10480,"v":[1.0,1.0],"ease":EASE}]}
p['tracks'].append({"name":"brand","layer":40,"elements":[lk]})
json.dump(p,open('presenter.montagent.json','w'))
