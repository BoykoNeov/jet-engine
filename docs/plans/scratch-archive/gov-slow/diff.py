# Compare two census files (i<TAB>req<TAB>reply): identical / differing replies, max relative change over numbers,
# outcome flips (ran <-> refused, refusal words), structure changes. Usage: diff.py A B
import json, sys
def flat(x, p='', out=None):
    out = {} if out is None else out
    if isinstance(x, dict):
        for k, v in x.items(): flat(v, f'{p}.{k}', out)
    elif isinstance(x, list):
        for i, v in enumerate(x): flat(v, f'{p}.{i}', out)
    else: out[p] = x
    return out
A = [l.rstrip('\n').split('\t') for l in open(sys.argv[1], encoding='utf-8')]
B = [l.rstrip('\n').split('\t') for l in open(sys.argv[2], encoding='utf-8')]
same = diff = flips = shape = 0; worst = (0, None, None); worst_abs_small = (0, None, None)
for (i, qa, ra), (_, qb, rb) in zip(A, B):
    assert qa == qb
    if ra == rb: same += 1; continue
    diff += 1
    if ra == 'PANIC' or rb == 'PANIC': flips += 1; print('PANIC flip', i); continue
    ja, jb = json.loads(ra), json.loads(rb)
    if ja.get('ok') != jb.get('ok') or ja.get('reason') != jb.get('reason'):
        flips += 1; print(f'#{i} OUTCOME: {ra[:160]}\n      -> {rb[:160]}'); continue
    fa, fb = flat(ja), flat(jb)
    if fa.keys() != fb.keys():
        shape += 1; print(f'#{i} SHAPE: {len(fa)} vs {len(fb)} keys; only A {list(fa.keys()-fb.keys())[:4]} only B {list(fb.keys()-fa.keys())[:4]}'); continue
    for k in fa:
        x, y = fa[k], fb[k]
        if x == y: continue
        if isinstance(x, (int, float)) and isinstance(y, (int, float)) and not isinstance(x, bool):
            rel = abs(x - y) / max(abs(x), abs(y), 1e-300)
            if max(abs(x), abs(y)) > 1e-6:
                if rel > worst[0]: worst = (rel, i, k)
            elif abs(x - y) > worst_abs_small[0]: worst_abs_small = (abs(x - y), i, k)
        else:
            print(f'#{i} TEXT {k}: {str(x)[:100]} -> {str(y)[:100]}')
print(f'{len(A)} requests: {same} identical, {diff} differ ({flips} outcome flips, {shape} shape changes)')
print(f'worst relative change {worst[0]:.3g} at #{worst[1]} {worst[2]}; worst absolute change on |x|<1e-6: {worst_abs_small[0]:.3g} at #{worst_abs_small[1]} {worst_abs_small[2]}')
