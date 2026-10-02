import json,sys; sys.path.insert(0,'build'); from png import bbox
rig=json.load(open('character/rig.json'))
for k,p in rig['parts'].items():
    if k.startswith('mouth') : continue
    x0,y0,x1,y1=bbox('character/'+p['file'])
    ox=p['pivot'][0]-p['width']/2; oy=p['pivot'][1]-p['height']/2
    print(k,'drawing bbox',x0+ox,y0+oy,x1+ox,y1+oy)
print('laptop',bbox('character/laptop.png'))
for f in ['lockup','mark','wordmark']: print(f,bbox('brand/'+f+'.png'))
