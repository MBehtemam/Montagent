import json,sys
d=json.load(open(sys.argv[1]))
POP=[0.34,1.56,0.64,1]
CLOSE=9680
for t in d['tracks']:
  if t['name'] in('captions','captions-bg'):
    t['elements']=[e for e in t['elements'] if e['start']<6700]
  if t['name'] in('payoff','payoff-bg'):
    for e in t['elements']:
      if e['end']>9600: e['end']=CLOSE
      s=e['start']
      e['scale']=[{"t":s,"v":[0.86,0.86]},{"t":s+280,"v":[1.0,1.0],"ease":POP}]
      if e['type']=='rect':
        e['stroke']='#FF5A36'; e['stroke_width']=6
  for e in t['elements']:
    if e['id']=='payoff-02':
      r=e['runs']; txt=[x['text'] for x in r]
      i=txt.index('nothing'); r[i+1]['text']='\n'
      j=txt.index('else'); r[j+1]['text']=' '
      e['width']=710
    if e['id']=='payoff-bg-02': e['width']=788
    if e['id'] in('presenter','halo'):
      e['opacity']=[{"t":CLOSE,"v":1.0},{"t":9960,"v":0.0,"ease":"ease-in"}]
    if e['id']=='lockup':
      E=[0.65,0,0.35,1]
      e['y']=[{"t":CLOSE,"v":209},{"t":10480,"v":675,"ease":E}]
      e['scale']=[{"t":CLOSE,"v":[0.45,0.45]},{"t":10480,"v":[1.0,1.0],"ease":E}]
    if e['id']=='bed':
      e['volume']=[k for k in e['volume'] if not (500<k['t']<1300)]
json.dump(d,open(sys.argv[2],'w'),indent=1)
