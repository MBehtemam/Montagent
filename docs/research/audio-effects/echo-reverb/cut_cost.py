#!/usr/bin/env python3
"""PROTOTYPE: what cutting the tail at `end` costs on the narration. Usage: python3 -I cut_cost.py [FFMPEG]"""
import importlib.util,json
import pathlib,sys
spec=importlib.util.spec_from_file_location('p',pathlib.Path(__file__).resolve().parent/'prototype_echo_reverb.py'); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
import numpy as np
ff=sys.argv[1] if len(sys.argv)>1 else "ffmpeg"; m.use(ff)
src=["-i",str(m.FIX/"narration.flac")]
E=m.SPEECH_END_MS*48
out={}
for kind,p,t in (("echo",m.ECHO,1500),("reverb",m.REVERB,1500)):
    x=m.pcm(ff,[*src,"-filter_complex",m.element_graph(kind,p,m.SPEECH_END_MS,t,"0:a","el"),"-map","[el]"])
    c=x[:E]; tail=x[E:]
    db=lambda v:20*np.log10(max(float(v),1e-12))
    # energy per 100ms window past end
    w=[round(db(np.sqrt((tail[i:i+4800]**2).mean())),1) for i in range(0,len(tail),4800)][:10]
    body=np.sqrt((c**2).mean())
    out[kind]={"element_rms_dbfs":round(db(body),1),"last_10ms_rms_dbfs_at_cut":round(db(np.sqrt((c[-480:]**2).mean())),1),
     "last_sample_abs":float(np.abs(c[-1]).max()),"tail_rms_per_100ms_dbfs":w,
     "tail_energy_share_of_element":float((tail**2).sum()/(x**2).sum()), "tail_first_100ms_re_element_rms_db":round(w[0]-db(body),1)}
print(json.dumps(out,indent=1))
