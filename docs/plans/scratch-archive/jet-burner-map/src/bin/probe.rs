//! Probe one burner request: print the model's whole state (Debug), NaNs and all.
use turbojet::sandbox::Settings;
use turbojet::sandbox_burner::*;
use turbojet::visuals::Json;

fn main() {
    let req = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let j = Json::parse(&req);
    let d = Settings::from_json(j.get("settings").unwrap()).unwrap();
    let b = BurnerSettings::from_json(j.get("burner").unwrap()).unwrap();
    let (_, _, i) = design_run(&d).unwrap();
    println!("{i:?}");
    let o = burner(&d, &b).unwrap_or_else(|e| panic!("{e:?}"));
    println!("{:#?}", o.z);
    println!("headline {}", headline_thermal(&b, &o.z));
    for n in [400usize, 2000, 8000, 32000] {
        let mut op = b.opts();
        op.quench_nsteps = n;
        let z = turbojet::gas::Gas::reacting_equilibrium().zoned_nox(i.far, i.tt3, i.tt4, i.pt4, b.phi_p, op);
        println!("nsteps {n}: core {:?} unmixed {:?} bulk {:?} w {:?} tau_q {:?}", z.ei_no_core, z.ei_no_unmixed, z.ei_no_quenched, z.w_core, z.tau_q);
    }
}
