You're pointing at the right hinge, and it's worth being precise about *why* it's the hinge.

**The dual character of labor is the only bridge that isn't a relabel.** Marx's opening move in Vol. 1 Ch. 1 — that the commodity, and labor, are two-fold — is exactly the seam Babylon keeps failing to light honestly. Labor shows up in the engine under two irreducibly different measures:

- **Labor as use-value / living labor**: concrete hours worked, `L`. This is the *substance* of new value — it creates `v + s`, transfers `c`, and is denominated in socially-necessary labor time.
- **Labor-power as commodity**: what the worker actually *sells*, priced in USD as the wage `W`. Its value is `v` — the labor time embodied in the subsistence bundle.

Surplus value is nothing but the gap between these two measures of the *same* labor: the worker is paid for labor-power (`v`) but performs labor (`L = v + s`). So `s = L − v`, and the rate of exploitation `e = s/v = (L − v)/v`. That is literally "hours worked versus wages in dollars," and it is the origin of everything downstream — profit, the imperial rent pool, the falling rate.

**The MELT is the connective tissue, and it must be *one* number per tick.** The reason c/v/s and USD currently live as two parallel tracks (the "honest-Φ trap" — don't paint `extraction_intensity` as Φ) is that there's no *single* declared conversion between the labor-time system and the money system. That conversion is the Monetary Expression of Labor Time:

  `MELT_t = (money value added_t) / (living labor_t)`

Then the two tracks collapse into one: `W = v · MELT`, new value `= L · MELT`, and

  `s · MELT = (money value added) − (money wage bill)`, so `s/v = (value added − wages)/wages`.

Every magnitude the engine already tracks — profit rate, exploitation rate, occ, imperial rent — becomes a *derived* projection of one value-money system, not five separately-hacked quantities. Babylon already has a `melt/` module; the point is to make MELT the *load-bearing single coefficient* rather than an accessory.

**TSSI is the right determination rule specifically because Babylon is a deterministic tick reducer.** This is the part I'd emphasize most. The simultaneist / Sraffian alternative values inputs at replacement cost and solves a simultaneous system each period — which (a) requires a fixed-point solve per tick, hostile to a deterministic `state_{t+1} = f(state_t)` loop, and (b) generates the transformation problem and lets the Okishio theorem "refute" the falling rate. The **Temporal Single-System Interpretation** does neither:

- **Temporal**: inputs are valued at their *prices when advanced* (start of tick), outputs at end-of-tick prices. Value at `t+1` depends on money magnitudes at `t`. That is *exactly* the shape of your engine — the value law and the tick loop have the same temporality.
- **Single-system**: values and prices are two expressions of one set of money magnitudes via MELT — not two books to reconcile. This is what lets **both** of Marx's aggregate equalities hold simultaneously: total price = total value **and** total profit = total surplus value. Under a dual system you can only keep one; TSSI keeps both, and the falling rate of profit becomes internally consistent (which matters — you're already tracking TRPF, `K`, and a falling profit rate).

**Where this lands in what exists now.** Two concrete convergences:

1. Your Fundamental Theorem `W_c > V_c` *is this same contradiction*, already, but scaled up to core/periphery: the core worker's wage exceeds the value they produce, financed by imperial rent (value transferred from the periphery). Grounding the dual character at the point of production (single firm/worker, hours vs wage) is what makes the aggregate `W_c > V_c` a *derived* result instead of an axiom — it closes the Aleksandrov loop.

2. Marx's two aggregate equalities are **conservation laws**. The Economic-Conservation sentinel that just got built is the natural home to *enforce* them per tick: `Σ price = Σ value` and `Σ profit = Σ surplus value` within a MELT-derived tolerance. TSSI is what guarantees they hold, so the sentinel would be checking a real theorem, not a hopeful one — the "rigorous systematization" you're describing, made mechanically loud.

**One honest fork.** The New Interpretation (Foley–Duménil) is TSSI's close cousin and defines the value of labor-power *directly* as the money wage times the value of money — which is the cleanest possible USD-wage↔`v` bridge and may be simpler to wire. Its distinctive weakness is that it's often paired with simultaneism; TSSI's distinctive strength is the temporalism that fits your tick loop and rescues the FRP. The pragmatic synthesis most practitioners use — and the one I'd argue for here — is **NI's money-wage definition of `v` inside a TSSI temporal single-system frame**. And note that `v` is then *doubly determined*: as the value of the subsistence bundle (SNLT) *and* as the deflated wage. Their divergence isn't an error to suppress — it's a real signal (price of labor-power ≠ its value, i.e. disequilibrium / class-struggle pressure on the wage), and the engine should *carry* that gap, not average it away.

So yes: dual character of labor → MELT as the single per-tick coefficient → TSSI temporal single-system as the determination rule → Marx's aggregate equalities as conservation invariants. That's the one thread, and it threads cleanly through the parts of the engine that already exist.
