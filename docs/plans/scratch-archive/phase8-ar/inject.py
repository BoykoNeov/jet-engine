import sys
p, which = sys.argv[1], sys.argv[2]
t = open(p, encoding='utf-8', newline='').read()
M = {
 "I1": ("let ts: Vec<f64> = (0..80).map(|i| ta + (tb - ta) * i as f64 / 79.0).collect();",
        "let ts: Vec<f64> = (0..80).map(|i| ta + (tb - ta) * (i as f64 / 79.0)).collect();"),
 "I2": ("\"k_p\" => kp, \"C_opt\" => 1.0 / (4.0 * (kp * kp)),",
        "\"k_p\" => kp, \"C_opt\" => 1.0 / (4.0 * kp * kp),"),
 "I3": ("jobj! { \"phi\" => r(phi, 4), \"f\" => r6(f) }",
        "jobj! { \"phi\" => phi, \"f\" => r6(f) }"),
}
a, b = M[which]
assert t.count(a) == 1, which
open(p, 'w', encoding='utf-8', newline='').write(t.replace(a, b))
print("applied", which)
