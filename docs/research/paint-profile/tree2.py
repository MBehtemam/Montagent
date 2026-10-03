"""(element class x operation) exclusive breakdown of a `sample` main-thread tree."""
import re, sys, collections
exec(open(__file__.replace('tree2.py', 'tree.py')).read().split('total = nodes[0][1]')[0].replace('sys.argv[1]', 'sys.argv[1]'))
CLS = [('Canvas::text', 'text'), ('Canvas::raster', 'image'), ('Canvas::shape', 'rect/ellipse'),
       ('Painter::paint', 'frame (no element)'), ('encode_span|render_cancellable', 'render loop')]
OPS = [('SkBlurImageFilter::onFilterImage', 'blur filter (blur + shadow)'),
       ('SkMergeImageFilter::onFilterImage', 'shadow merge'),
       ('ColorFilterImageFilter', 'colour-matrix filter'),
       ('internalDrawDeviceWithFilter', 'filtered layer composite'),
       ('drawDevice', 'plain layer composite'),
       ('internalSaveLayer', 'layer open'),
       ('SkCanvas::clear|eraseColor|SkBitmapDevice::clear', 'clear'),
       ('montagent_text::|hb_|Shaper', 'text layout/shaping'),
       ('drawPath|draw_path', 'path draw'),
       ('drawImage|drawBitmap|draw_image', 'image draw'),
       ('decode|Codec', 'decode'),
       ('write', 'pipe write')]
selfc = [c for _, c, _ in nodes]; st = []
for i, (d, c, n) in enumerate(nodes):
    while st and nodes[st[-1]][0] >= d: st.pop()
    if st: selfc[st[-1]] -= c
    st.append(i)
out = collections.Counter(); st = []
for i, (d, c, n) in enumerate(nodes):
    while st and st[-1][0] >= d: st.pop()
    pc, po = st[-1][1] if st else (None, None)
    cl = next((l for p, l in CLS if re.search(p, n)), None) or pc
    op = next((l for p, l in OPS if re.search(p, n)), None) or po
    st.append((d, (cl, op)))
    out[(cl, op or 'other')] += selfc[i]
total = nodes[0][1]
bycl = collections.Counter()
for (cl, op), s in out.items(): bycl[cl] += s
for cl, s in bycl.most_common():
    print(f"{str(cl):22s} {100*s/total:5.1f}%")
    for (c2, op), s2 in out.most_common():
        if c2 == cl and s2/total > 0.002: print(f"    {op:32s} {100*s2/total:5.1f}%")
