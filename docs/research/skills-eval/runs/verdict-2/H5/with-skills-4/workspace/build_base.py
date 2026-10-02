import json
P = json.load(open("h5.json"))
fv = P.get("fontVendor")
EASE_IO=[0.65,0,0.35,1]; POP=[0.34,1.56,0.64,1]
proj = {
 "frame":{"width":1080,"height":1350},"fps":25,"background":"#F5F0E6","duration":12000,"output":"deliverable.mp4",
 "fonts":{"bold":[{"file":"fonts/Inter-Bold.ttf"}],"regular":[{"file":"fonts/Inter-Regular.ttf"}]},
 "fontVendor":fv,
 "tracks":[
  {"name":"motif","layer":2,"elements":[
   {"id":"motif","type":"image","start":0,"end":10000,"source":"brand/mark.png","x":540,"y":600,"origin":"center","width":1240,"height":1240,"fit":"contain","opacity":0.07,
    "scale":[{"t":0,"v":[1.0,1.0]},{"t":9960,"v":[1.06,1.06],"ease":"linear"}]}]},
  {"name":"presenter","layer":10,"elements":[
   {"id":"presenter","type":"video","start":0,"end":10000,"source":"presenter/take-2.mp4","source_start":0,"source_end":9800,"overrun":"hold","volume":1.0,
    "x":540,"y":1758,"origin":"bottom-center","width":3072,"height":1728,"fit":"cover",
    "effects":[{"name":"chroma","color":"#00FF00","tolerance":0.2,"softness":0.08,"spill":0.9}]}]},
  {"name":"bar","layer":20,"elements":[
   {"id":"bar","type":"rect","start":600,"end":5400,"x":64,"y":760,"origin":"center-left","width":640,"height":150,"fill":"#101418","radius":8,
    "scale":[{"t":600,"v":[0.0,1.0]},{"t":920,"v":[1.0,1.0],"ease":EASE_IO},{"t":5120,"v":[1.0,1.0],"ease":"linear"},{"t":5360,"v":[0.0,1.0],"ease":EASE_IO}]}]},
  {"name":"bar-edge","layer":21,"elements":[
   {"id":"bar-edge","type":"rect","start":600,"end":5400,"x":64,"y":760,"origin":"center-left","width":12,"height":150,"fill":"#FF5A36"}]},
  {"name":"bar-mark","layer":22,"elements":[
   {"id":"bar-mark","type":"image","start":920,"end":5120,"source":"brand/mark-on-dark.png","x":104,"y":760,"origin":"center-left","width":96,"height":96,"fit":"contain"}]},
  {"name":"bar-name","layer":22,"elements":[
   {"id":"bar-name","type":"text","start":920,"end":5120,"x":224,"y":728,"origin":"center-left","width":420,"height":70,"font":"bold","size":56,"color":"#F5F0E6","runs":[{"text":"Montagent"}],"caption":False}]},
  {"name":"bar-line","layer":22,"elements":[
   {"id":"bar-line","type":"text","start":920,"end":5120,"x":226,"y":792,"origin":"center-left","width":420,"height":44,"font":"regular","size":32,"color":"#F5F0E6","runs":[{"text":"The video editor agents drive"}],"caption":False}]},
  {"name":"bar-cover","layer":23,"elements":[
   {"id":"bar-cover-on","type":"rect","start":920,"end":1320,"x":684,"y":760,"origin":"center-right","width":600,"height":130,"fill":"#101418","scale":[{"t":920,"v":[1.0,1.0]},{"t":1280,"v":[0.0,1.0],"ease":EASE_IO}]},
   {"id":"bar-cover-off","type":"rect","start":4880,"end":5120,"x":84,"y":760,"origin":"center-left","width":600,"height":130,"fill":"#101418","scale":[{"t":4880,"v":[0.0,1.0]},{"t":5080,"v":[1.0,1.0],"ease":EASE_IO}]}]},
  {"name":"endpanel","layer":50,"elements":[
   {"id":"endpanel","type":"rect","start":9720,"end":12000,"x":540,"y":[{"t":9720,"v":1350},{"t":10000,"v":0,"ease":EASE_IO}],"origin":"top-center","width":1080,"height":1350,"fill":"#101418"}]},
  {"name":"lockup","layer":51,"elements":[
   {"id":"lockup","type":"image","start":10000,"end":12000,"source":"brand/lockup-on-dark.png","x":540,"y":630,"origin":"center","width":760,"height":176,"fit":"contain",
    "scale":[{"t":10000,"v":[0.0,0.0]},{"t":10400,"v":[1.0,1.0],"ease":POP}]}]},
  {"name":"tagline","layer":52,"elements":[
   {"id":"tagline","type":"text","start":10200,"end":12000,"x":540,"y":790,"origin":"center","width":640,"height":53,"font":"regular","size":44,"color":"#F5F0E6","align":"center","runs":[{"text":"The video editor agents drive"}],"caption":False,
    "opacity":[{"t":10200,"v":0.0},{"t":10480,"v":1.0,"ease":"ease-out"}],"y":[{"t":10200,"v":814},{"t":10480,"v":790,"ease":[0.25,1,0.5,1]}]}]},
  {"name":"music","layer":0,"elements":[
   {"id":"bed","type":"audio","start":0,"end":12000,"source":"music/bed-120bpm.wav","source_start":0,"source_end":12000,"volume":0.5}]}
 ]}
json.dump(proj, open("h5.json","w"), indent=2)
