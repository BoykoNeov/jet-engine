# Census split: ran<->refused flips; refusals with moved words; runs: worst relative change, worst hold miss.
import json, sys, re
def flat(x, p='', out=None):
    out = {} if out is None else out
    if isinstance(x, dict):
        for k, v in x.items(): flat(v, f'{p}.{k}', out)
    elif isinstance(x, list):
        for i, v in enumerate(x): flat(v, f'{p}.{i}', out)
    else: out[p] = x
    return out
def rows(p): return [l.rstrip('\n').split('\t') for l in open(p, encoding='utf-8')]
A, B = rows(sys.argv[1]), rows(sys.argv[2])
same = flip = refmoved = refwords = runmoved = 0
worst = (0, None, None); misses = []; refs = []
def holds(j):
    out = []
    for k, v in flat(j).items():
        if k.endswith('.hold_target') and v is not None:
            base = k[:-len('hold_target')]
            out.append((base, v, flat(j).get(base + 'hold_achieved'), flat(j).get(base + 'hold_capped')))
    return out
for (i, q, ra), (_, _, rb) in zip(A, B):
    if ra == rb: same += 1; continue
    ja, jb = json.loads(ra), json.loads(rb)
    if ja.get('ok') != jb.get('ok'):
        flip += 1; print(f'#{i} FLIP ok {ja.get("ok")} -> {jb.get("ok")}: {(ja.get("reason") or jb.get("reason"))[:200]}'); continue
    if not ja.get('ok'):
        # same verdict, moved words: digits only, or different words?
        strip = lambda s: re.sub(r'-?\d+(\.\d+)?(e-?\d+)?', '#', s or '')
        if strip(ja.get('reason')) == strip(jb.get('reason')): refmoved += 1; refs.append((i, ja.get('reason'), jb.get('reason')))
        else: refwords += 1; print(f'#{i} REFUSAL WORDS:\n   {ja.get("reason")[:300]}\n-> {jb.get("reason")[:300]}')
        continue
    runmoved += 1
    fa, fb = flat(ja), flat(jb)
    for k in fa.keys() & fb.keys():
        x, y = fa[k], fb[k]
        if isinstance(x, (int, float)) and isinstance(y, (int, float)) and not isinstance(x, bool) and x != y:
            if max(abs(x), abs(y)) > 1e-6:
                rel = abs(x - y) / max(abs(x), abs(y))
                if rel > worst[0]: worst = (rel, i, k)
    for (base, t, a, c) in holds(jb):
        if a is None or c: continue
        ta = dict((b, (tt, aa)) for b, tt, aa, cc in holds(ja)).get(base)
        misses.append((abs(a - t) / abs(t), abs(ta[1] - ta[0]) / abs(ta[0]) if ta else None, i, base))
print(f'{len(A)}: {same} identical; {flip} ran<->refused flips; {refmoved} refusals same words numbers moved; '
      f'{refwords} refusals with different words; {runmoved} runs with moved numbers')
print(f'runs: worst relative change {worst[0]:.3g} at #{worst[1]} {worst[2]}')
misses.sort(reverse=True)
print('worst uncapped hold misses (variant rel, base rel, #, end):')
for m in misses[:6]: print('  ', f'{m[0]:.3g}', f'{m[1]:.3g}' if m[1] is not None else '-', m[2], m[3])
for i, a, b in refs[:4]: print(f'  #{i}: {a[:170]}\n     -> {b[:170]}')
