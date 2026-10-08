/// `test_p5`'s spacing formula at `ds = 0.005`: `ds·(hi - lo)/ds_star`.
const SPACING: f64 = 0.005 * (0.024 - 0.016) / 0.005938;

impl Flat {
    fn oi(&mut self, p: &str, x: Option<i64>) {
        match x {
            Some(x) => self.put(p, format!("i:{x}")),
            None => self.none(p),
        }
    }
    fn kind(&mut self, p: &str, k: Option<staircase_law::Kind>) {
        self.os(p, k.map(|k| k.as_str()));
    }
}

fn edge(o: &mut Flat, p: &str, e: &EdgeRead) {
    o.keys(p, 19);
    o.f(&format!("{p}.tau_f"), e.tau_f);
    o.f(&format!("{p}.r"), e.r);
    o.f(&format!("{p}.ds"), e.ds);
    o.of(&format!("{p}.h"), e.h);
    o.of(&format!("{p}.kappa"), e.kappa);
    o.b(&format!("{p}.kappa_pure"), e.kappa_pure);
    o.of(&format!("{p}.F"), e.f);
    o.of(&format!("{p}.g"), e.g);
    o.len(&format!("{p}.summands"), e.summands.len());
    for (k, &(s, v)) in e.summands.iter().enumerate() {
        o.len(&format!("{p}.summands.{k}"), 2);
        o.f(&format!("{p}.summands.{k}.0"), s);
        o.f(&format!("{p}.summands.{k}.1"), v);
    }
    o.of(&format!("{p}.s_bind"), e.s_bind);
    o.of(&format!("{p}.edge"), e.edge);
    o.b(&format!("{p}.at_edge"), e.at_edge);
    o.i(&format!("{p}.n_ride"), e.n_ride);
    o.i(&format!("{p}.n_scored"), e.n_scored);
    o.i(&format!("{p}.n_slope_excluded"), e.n_slope_excluded);
    o.b(&format!("{p}.window_open"), e.window_open);
    o.b(&format!("{p}.riding4_valid"), e.riding4_valid);
    o.b(&format!("{p}.edge_on_grid"), e.edge_on_grid);
    o.oi(&format!("{p}.edge_index"), e.edge_index);
}

fn classified(o: &mut Flat, p: &str, c: &Classified) {
    o.keys(p, 17);
    o.f(&format!("{p}.tau_lo"), c.tau_lo);
    o.f(&format!("{p}.tau_hi"), c.tau_hi);
    o.fs(&format!("{p}.entered"), &c.entered);
    o.fs(&format!("{p}.left"), &c.left);
    o.b(&format!("{p}.set_changed"), c.set_changed);
    o.b(&format!("{p}.argmin_moved"), c.argmin_moved);
    o.b(&format!("{p}.edge_moved"), c.edge_moved);
    o.of(&format!("{p}.h_lo"), c.h_lo);
    o.of(&format!("{p}.h_hi"), c.h_hi);
    o.of(&format!("{p}.h_common_lo"), c.h_common_lo);
    o.of(&format!("{p}.h_common_hi"), c.h_common_hi);
    o.of(&format!("{p}.d_full"), c.d_full);
    o.of(&format!("{p}.d_smooth"), c.d_smooth);
    o.of(&format!("{p}.d_membership"), c.d_membership);
    o.b(&format!("{p}.sign_change"), c.sign_change);
    o.b(&format!("{p}.sign_change_common"), c.sign_change_common);
    o.kind(&format!("{p}.kind"), c.kind);
}

fn ladder(o: &mut Flat, p: &str, s: &StaircaseScan) {
    o.keys(p, 22);
    o.f(&format!("{p}.lo"), s.lo);
    o.f(&format!("{p}.hi"), s.hi);
    o.i(&format!("{p}.n"), s.n);
    o.len(&format!("{p}.points"), s.points.len());
    for (k, x) in s.points.iter().enumerate() {
        edge(o, &format!("{p}.points.{k}"), x);
    }
    o.len(&format!("{p}.pairs"), s.pairs.len());
    for (k, x) in s.pairs.iter().enumerate() {
        classified(o, &format!("{p}.pairs.{k}"), x);
    }
    o.len(&format!("{p}.changes"), s.changes.len());
    for (k, x) in s.changes.iter().enumerate() {
        classified(o, &format!("{p}.changes.{k}"), x);
    }
    o.i(&format!("{p}.n_at_edge"), s.n_at_edge);
    o.i(&format!("{p}.n_points"), s.n_points);
    o.b(&format!("{p}.all_at_edge"), s.all_at_edge);
    o.i(&format!("{p}.n_sign_changes"), s.n_sign_changes);
    o.i(&format!("{p}.n_crossings"), s.n_crossings);
    o.i(&format!("{p}.n_jumps"), s.n_jumps);
    o.b(&format!("{p}.exact_zero_when_set_equal"), s.exact_zero_when_set_equal);
    o.b(&format!("{p}.nonzero_when_set_differs"), s.nonzero_when_set_differs);
    o.i(&format!("{p}.n_argmin_only"), s.n_argmin_only);
    o.i(&format!("{p}.n_set_only"), s.n_set_only);
    o.i(&format!("{p}.n_edge_moves"), s.n_edge_moves);
    o.b(&format!("{p}.all_on_grid"), s.all_on_grid);
    o.b(&format!("{p}.edge_monotone"), s.edge_monotone);
    o.len(&format!("{p}.edge_indices"), s.edge_indices.len());
    for (k, &x) in s.edge_indices.iter().enumerate() {
        o.oi(&format!("{p}.edge_indices.{k}"), x);
    }
    o.b(&format!("{p}.all_open"), s.all_open);
    o.b(&format!("{p}.all_kappa_pure"), s.all_kappa_pure);
}

fn lattice(o: &mut Flat, p: &str, c: &LatticeCount) {
    o.keys(p, 14);
    o.f(&format!("{p}.lo"), c.lo);
    o.f(&format!("{p}.hi"), c.hi);
    o.f(&format!("{p}.ds"), c.ds);
    o.f(&format!("{p}.r"), c.r);
    o.of(&format!("{p}.edge_lo"), c.edge_lo);
    o.of(&format!("{p}.edge_hi"), c.edge_hi);
    o.oi(&format!("{p}.index_lo"), c.index_lo);
    o.oi(&format!("{p}.index_hi"), c.index_hi);
    o.oi(&format!("{p}.n_jumps"), c.n_jumps);
    o.of(&format!("{p}.ds_star"), c.ds_star);
    o.of(&format!("{p}.slope_s_star"), c.slope_s_star);
    o.b(&format!("{p}.on_grid"), c.on_grid);
    o.len(&format!("{p}.at_edge"), 2);
    o.b(&format!("{p}.at_edge.0"), c.at_edge.0);
    o.b(&format!("{p}.at_edge.1"), c.at_edge.1);
    o.os(&format!("{p}.void"), c.void.as_deref());
}

fn number(o: &mut Flat, p: &str, n: &StaircaseNumber) {
    match n {
        StaircaseNumber::V2 { void, tau_lo, tau_hi } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.tau_lo"), *tau_lo);
            o.f(&format!("{p}.tau_hi"), *tau_hi);
            o.none(&format!("{p}.rise"));
            o.none(&format!("{p}.dg_dtau"));
        }
        StaircaseNumber::V5 { void, tau_lo, tau_hi, edge_moved } => {
            o.keys(p, 6);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.tau_lo"), *tau_lo);
            o.f(&format!("{p}.tau_hi"), *tau_hi);
            o.b(&format!("{p}.edge_moved"), *edge_moved);
            o.none(&format!("{p}.rise"));
            o.none(&format!("{p}.dg_dtau"));
        }
        StaircaseNumber::Ok(x) => {
            o.keys(p, 15);
            o.f(&format!("{p}.tau_lo"), x.tau_lo);
            o.f(&format!("{p}.tau_hi"), x.tau_hi);
            o.f(&format!("{p}.ds"), x.ds);
            o.f(&format!("{p}.r"), x.r);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.rise"), x.rise);
            o.f(&format!("{p}.dg_dtau"), x.dg_dtau);
            o.f(&format!("{p}.dtau"), x.dtau);
            o.of(&format!("{p}.spacing"), x.spacing);
            o.of(&format!("{p}.tread"), x.tread);
            o.of(&format!("{p}.lam"), x.lam);
            o.fs(&format!("{p}.entered"), &x.entered);
            o.fs(&format!("{p}.left"), &x.left);
            o.f(&format!("{p}.d_membership"), x.d_membership);
            o.f(&format!("{p}.d_smooth"), x.d_smooth);
        }
    }
}

fn root(o: &mut Flat, p: &str, r: &RootClass) {
    match r {
        RootClass::Void { void, r, ds } => {
            o.keys(p, 5);
            o.s(&format!("{p}.void"), void);
            o.f(&format!("{p}.r"), *r);
            o.f(&format!("{p}.ds"), *ds);
            o.none(&format!("{p}.kind"));
            o.none(&format!("{p}.root_exists"));
        }
        RootClass::Ok(x) => {
            o.keys(p, 21);
            o.f(&format!("{p}.r"), x.r);
            o.f(&format!("{p}.ds"), x.ds);
            o.none(&format!("{p}.void"));
            o.f(&format!("{p}.lo"), x.lo);
            o.f(&format!("{p}.hi"), x.hi);
            o.f(&format!("{p}.mid"), x.mid);
            o.f(&format!("{p}.width"), x.width);
            o.kind(&format!("{p}.kind"), x.kind);
            o.b(&format!("{p}.root_exists"), x.root_exists);
            o.b(&format!("{p}.set_changed"), x.set_changed);
            o.b(&format!("{p}.argmin_moved"), x.argmin_moved);
            o.b(&format!("{p}.edge_moved"), x.edge_moved);
            o.fs(&format!("{p}.entered"), &x.entered);
            o.fs(&format!("{p}.left"), &x.left);
            o.of(&format!("{p}.d_full"), x.d_full);
            o.of(&format!("{p}.d_smooth"), x.d_smooth);
            o.of(&format!("{p}.d_membership"), x.d_membership);
            o.of(&format!("{p}.h_lo"), x.h_lo);
            o.of(&format!("{p}.h_hi"), x.h_hi);
            o.of(&format!("{p}.h_common_lo"), x.h_common_lo);
            o.of(&format!("{p}.h_common_hi"), x.h_common_hi);
        }
    }
}

