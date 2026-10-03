"""Draw the T–s diagram (ideal vs real) from the data the Rust CLI writes.

    cargo run --release --manifest-path rust/Cargo.toml -- ts-diagram   # -> ts_diagram.json
    python plot_ts_diagram.py                                            # -> ts_diagram.png

(The full `cargo run --release` panel run writes `ts_diagram.json` too.)

This is the one piece of the project left in Python by decision (docs/plans/todo-rust-port.md
§ 9 item 2): matplotlib DRAWS here, and nothing else happens. It imports no engine code and
does no physics — every point, both 80-point isobar-shaped legs per cycle and the title come
from the JSON, computed by `rust/src/visuals.rs` (`ts_diagram`). What stays here is
presentation: colours, line styles, labels, layout. The calls are `main.py`'s `plot_ts_diagram`
call for call, in its order, so the PNG is the one that function drew (slice AR checked the two
renders byte for byte).

Overlay of the ideal cycle (vertical "isentropic" legs) and the real cycle (legs tilted right by
entropy generation) — the rung-2 payoff artifact. Each cycle is a closed Brayton loop: two work
legs (0->2->3 and 4->5->9) and two constant-pressure legs (combustion 3->4, heat rejection 9->0).
Stations 0 and 9 are STATIC (at ambient p0) while 2..5 are total — in pure total coordinates an
ideal nozzle conserves the totals, so 9 would collapse onto 5 and the heat-rejection leg vanish.
"""
import json
import sys

import matplotlib

matplotlib.use("Agg")  # headless: render to a file, never pop a window
import matplotlib.pyplot as plt  # noqa: E402


def main(src="ts_diagram.json", out="ts_diagram.png"):
    sys.stdout.reconfigure(encoding="utf-8")  # the closing line prints an en dash
    with open(src, encoding="utf-8") as fh:
        data = json.load(fh)

    fig, ax = plt.subplots(figsize=(8.5, 6.5))

    def draw(cyc, color, ls, lw, label, alpha):
        # Work legs (would be vertical if isentropic).
        for leg in cyc["work_legs"]:
            ax.plot(leg["s"], leg["T"], color=color, ls=ls, lw=lw, alpha=alpha, zorder=2)
        # Combustion 3->4 and heat rejection 9->0: the cp*ln(T) isobar shape plus the linear
        # residual that lands the curve on the true endpoint (the burner's pressure-loss entropy).
        for iso in cyc["isobars"]:
            ax.plot(iso["s"], iso["T"], color=color, ls=ls, lw=lw, alpha=alpha, zorder=2)
        ax.plot([], [], color=color, ls=ls, lw=lw, label=label)  # legend proxy
        for pt in cyc["points"]:
            ax.scatter([pt["s"]], [pt["T"]], color=color, s=28, zorder=3, alpha=alpha)

    draw(data["ideal"], "tab:blue", "--", 1.8, "ideal (isentropic legs)", 0.7)
    draw(data["real"], "tab:red", "-", 2.2, "real (legs tilt right)", 1.0)

    # Label the real-cycle stations (the ones that moved).
    for pt in data["real"]["points"]:
        ax.annotate(f"  {pt['label']}", (pt["s"], pt["T"]), fontsize=11, fontweight="bold", va="center")

    ax.set_xlabel("entropy  s − s0  [J/(kg·K)]")
    ax.set_ylabel("temperature  T  [K]")
    ax.set_title(data["title"])
    ax.legend(loc="upper left")
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
    fig.savefig(out, dpi=120)
    plt.close(fig)
    print(f"T–s diagram (ideal vs real) written to {out}")


if __name__ == "__main__":
    main(*sys.argv[1:3])
