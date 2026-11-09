import json, sys
def flat(x, p='', out=None):
    out = {} if out is None else out
    if isinstance(x, dict):
        for k, v in x.items(): flat(v, f'{p}.{k}', out)
    elif isinstance(x, list):
        for i, v in enumerate(x): flat(v, f'{p}.{i}', out)
    else: out[p] = x
    return out
rows = lambda p: [l.rstrip('\n').split('\t') for l in open(p, encoding='utf-8')]
A, B = rows(sys.argv[1]), rows(sys.argv[2])
held = []; dT = []; other = (0, None, None); thrust = (0, None)
for (i, q, ra), (_, _, rb) in zip(A, B):
    if ra == rb: continue
    ja, jb = json.loads(ra), json.loads(rb)
    if not ja.get('ok'): continue
    fa, fb = flat(ja), flat(jb)
    for end in ('start', 'end'):
        t = fa.get(f'.{end}.hold_target'); a0 = fa.get(f'.{end}.hold_achieved'); a1 = fb.get(f'.{end}.hold_achieved')
        if t is not None and a0 is not None:
            held.append((abs(a1 - a0) / abs(t), i, end, fa.get(f'.{end}.hold_capped'), json.loads(q)['controls']['hold']))
        T0, T1 = fa.get(f'.{end}.Tt4'), fb.get(f'.{end}.Tt4')
        if T0 is not None: dT.append((abs(T1 - T0), i, end))
    for k in fa.keys() & fb.keys():
        x, y = fa[k], fb[k]
        if isinstance(x, (int, float)) and not isinstance(x, bool) and isinstance(y, (int, float)) and x != y:
            if 'settled' in k or max(abs(x), abs(y)) < 1e-3: continue
            rel = abs(x - y) / max(abs(x), abs(y))
            if rel > other[0]: other = (rel, i, k)
held.sort(reverse=True); dT.sort(reverse=True)
print('largest change in the held quantity, relative to its target (rel, #, end, capped, hold):')
for h in held[:5]: print('  ', f'{h[0]:.3g}', *h[1:])
print('largest change in an end\'s burner temperature, K:', [(f'{d[0]:.3g}', d[1], d[2]) for d in dT[:5]])
print(f'worst relative change in any other number (|x|>=1e-3, not "settled"): {other[0]:.3g} at #{other[1]} {other[2]}')
