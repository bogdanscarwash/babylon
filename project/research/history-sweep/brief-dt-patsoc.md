# Design Brief: DT — The PatSoc Pipeline

**Theme:** National bolshevism, Strasserism, left-nationalist drift, the CLASS_ANALYSIS vs
NATIONAL_CHAUVINISM tag axis, and the mechanically-triggered faction flip ("You are the fascist who
thought he was a communist").

**Scope note on citations:** every claim carries the local corpus path it was extracted from.
Paths under `www.marxists.org/...` and the bare `archive/`, `history/`, `reference/`,
`subject/` paths are **primary-source documents** (party programs, congress minutes,
polemics, speeches, constitutions). Paths under `prolewiki/Exports/...` are a mix of
**primary-source library texts** (reprinted speeches, marked as such below) and **ProleWiki
encyclopedia summaries**, flagged explicitly wherever used since they report on events rather
than being the event's own record.

---

## 1. Historical Evidence

### 1.1 The fork itself: class analysis vs. the national question is a real, load-bearing axis

The tension the doctrine tree wants to model is not a game invention. At the Second RSDLP
Congress (1903), Lenin fought the Bund's demand for federated, nationality-organized structure
directly: *"Federation is harmful because it sanctions segregation and alienation, elevates
them to a principle, to a law"* (`archive/lenin/works/1903/2ndcong/13.htm`). The same Congress
saw the Polish Social-Democrats propose "cultural-national autonomy... only under another
name" — Lenin's specific complaint was that an ostensibly internationalist faction smuggled in
a national-particularist demand under euphemism (`archive/lenin/works/1903/2ndcong2/4.htm`).
This is the disguised-drift pattern the pipeline needs to model: a doctrine node can read as
pure class politics while functioning as national politics.

The 1920 Comintern's National and Colonial Question theses quantify the scale of the problem:
roughly 70% of world population belonged to oppressed/colonial-dependent nations
(`history/international/comintern/2nd-congress/ch04.htm`), and the Congress commission
deliberately amended "bourgeois-democratic movement" to "national-revolutionary movement" on
the floor — a procedurally-recorded case of redrawing the exact tag boundary between uncritical
nationalist support and class-conditioned support (same source). The Baku Congress of the
Peoples of the East (September 1920) shows the ratio this produced in practice: of ~1,926
classified delegates, 1,071 (55.6%) were Communists and 855 (44.4%) were non-Communist
sympathizers or nationalists (`history/international/comintern/baku/delegates.htm`) — even
orthodox Bolshevik agitators reached for civilizational/martial framing to mobilize this
audience, with Radek invoking "the warlike feelings which once inspired the peoples of the
East when these peoples, led by their great conquerors, advanced upon Europe"
(`history/international/comintern/baku/ch02.htm`).

### 1.2 Legitimate fusion vs. corrosive substitution — the African Blood Brotherhood as control case

The African Blood Brotherhood's 1922 program is the cleanest instance of CLASS_ANALYSIS and
NATIONAL_CHAUVINISM coexisting *stably* in one tag vector rather than one crowding out the other. It
fuses explicit class analysis (chattel-vs-wage-slavery framing, Third International alignment)
with race-first liberation goals, and pairs an open Northern Federation with a secret Southern
"Protective organization" modeled explicitly on Sinn Fein
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/usa/groups/abb/1922/0400-abb-program.pdf`).
It codified a *graded* alliance ladder — actual allies, potential allies whose consciousness
"must be awakened," and enemies — rather than a binary split (same source), and its own program
warns internally against Black-owned enterprise as a liberation substitute: *"sudden financial
collapse of such enterprises may break the whole morale of the Liberation Movement"* — an early
marker on the PatSoc drift gradient, distinct from the ABB's class-conscious mainline (same
source). By contrast, Cyril Briggs's 1918 pre-ABB "American Race Problem" essay argues pure
territorial-sovereignty separatism with no class-solidarity framing at all
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/history/usa/groups/abb/1918/0900-briggs-amraceproblem.pdf`)
— the "before" anchor showing the same author/organization moved *toward* CLASS_ANALYSIS once
Comintern-aligned. "Several thousand" UNIA members defected to the ABB's class-and-race
synthesis around 1921 (`history/usa/groups/abb/1921/1210-lorenzo-negrolib.pdf`). Garvey's own
compensation — $12,000/yr as UNIA president plus $10,000/yr as Black Star Line president,
against ordinary officer salaries of $3,000/yr — is a concrete contemporaneous figure for
elite-capture inside a nationalist-trunk organization (same source).

The distinction the game needs is echoed decades later, sympathetically, by Stokely
Carmichael: *"racism comes first, far more important than exploitation"* to Black Power-era
organizers (`prolewiki/Exports/Library/Library_Stokely Speaks_ Black Power Back to
Pan-Africanism.txt` — primary-source speech reprint). This is a legitimate crossroads the tag
vector should represent as contestable, not an automatic villain-flag. Harry Haywood's 1930
CPUSA/Comintern polemic gives the internal naming precedent for the axis itself: he attacks
"race question" framing as the opportunist/chauvinist deviation and defends the October 1928
ECCI resolution's "national question" framing as the corrected line
(`history/usa/parties/cpusa/1930-haywood-againstbourgeoisliberaldistortions.pdf`).

### 1.3 The literal historical PatSoc pipeline: Hamburg National Bolshevism, 1919–1921

This is the strongest single case in the corpus for the pipeline's actual mechanism. Wolffheim
and Laufenberg's Hamburg "National Bolshevism" bridged class-council language into
national-unity language *smoothly*, through shared vocabulary rather than a discrete break:
*"The councils system groups together all the workers... behind the class interests which are
the interests of socialism and the nation. The factory councils will become an element of
national unity"*
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/subject/germany-1918-23/dauve-authier/appendix3.htm`).
The drift required no new theoretical apparatus, only reframing existing vocabulary — direct
support for a **zero-cost** doctrine node. Initial support was real and locally competitive:
"several thousand Hamburg workers" vs "a few hundred" for the orthodox KPD locally (same
source), but the current collapsed within roughly 2–3 years, consistent with the broader
AAUD/KAPD organizational collapse curve documented elsewhere in the same crisis (AAUD peaked at
~150,000 members winter 1920–21 against KAPD's ~40,000, then "lost almost all of its members
after 1923" — `.../dauve-authier/04.htm`). Wolffheim himself later died in a concentration
camp — the pipeline's terminal node offered no safe harbor even in fascist terms.

The 1923 Ruhr crisis (French occupation) produced an adjacent case: a "Group of German
Communist Army Officers" (KPD-linked) adopted Spenglerian national bolshevism mid-crisis,
redefining the council system as "a Prussian notion based on concepts of an elite, solidarity
and mutual responsibility" — and the KPD held actual fraternization meetings with Nazis during
this period, in one of which "a Nazi orator rendered homage to the communists, but ironically
advised them to rid themselves of the Jews" (`.../dauve-authier/ch15.htm`). No cadre-count or
duration figure survives for this specific officer group in the corpus — flag as a data gap.

### 1.4 The doctrinal content of the terminal node: Spengler and the Feder/DAP precursor

Spengler's "Prussian Socialism" supplies the literal terminal tag-vector content: *"Not Marx's
theory, but Frederick William's Prussian practice... trains the individual in his duty to the
whole"* — hierarchy/duty/blood displacing class analysis while retaining socialist vocabulary
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/subject/fascism/blick/ch08.htm`). The
Communist Manifesto itself pre-names this family of deviations — Feudal Socialism,
Petit-Bourgeois Socialism, "German, or 'True,' Socialism" — ideologies that borrow socialist
language while defending pre-capitalist or petit-bourgeois property relations (same source),
giving the pipeline's intermediate nodes a period-accurate taxonomy rather than invented names.
The recurrence is long: the "blame finance capital, spare industrial capital" move recurs from
Luther (1530) to Feder (1920), ~390 years, independent of theoretical investment (same source)
— support for treating this move as a standing, near-zero-cost node.

The actual founding document of Nazism gives the pipeline's real first step. The German
Workers' Party (DAP) Vienna programme, August 1918, splits capital into "disintegrating finance
capital" versus "the highly desirable productive national capital" (`.../blick/ch11.htm`) —
the exact tag-vector substitution the pipeline should model as its entry node: capital-fraction
framing that reads as economically radical replaces class framing, without ever being labeled
nationalist.

### 1.5 Strasserism proper — Otto Strasser's break, the Stennes revolt, and material capture

Otto Strasser broke with Hitler in 1930; the Stennes SA revolt of 1931 (the "Black Front")
accused the NSDAP leadership of straying from "revolutionary National Socialism," charging that
"the revolutionary force of the SA has been saturated with bourgeois liberal tendencies"
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/subject/fascism/blick/ch20.htm`). This is
the mirror-image PatSoc case for calibration: dissidents *inside* a fascist movement using the
identical "betrayed the revolution to capital" rhetorical shape, running in the opposite
direction. Hitler's SA-discipline decree came 28 March 1931; Stennes rebelled and was expelled
within about one week (same source) — a concrete days-to-a-week latency for how fast a
leadership purges an internal ideological deviation once it becomes open revolt. Big-capital
patronage directly conditioned the purge: Schacht made cooperation with Hitler contingent on
sacrificing the Strasser brothers, and the Rhenish-Westphalian coal syndicate had been paying
Gregor Strasser RM 10,000/month from spring 1931 specifically to maintain an internal
counterweight to Goebbels/Goering (same source) — a real cadre-patronage figure for "buying
off" a PatSoc faction.

### 1.6 Liquidationism via nationalist framing — Browder and the CPUSA

Earl Browder's "national unity" framing was the rhetorical vehicle for dissolving the CPUSA
into the Communist Political Association in May 1944: *"in the interests of national unity...
to make still greater contributions toward winning the war"*
(`.../history/usa/parties/cpusa/1944/05/0520-cpusa-convminutes.pdf`). The successor body's
founding charter states the substitution baldly: it "carries forward the traditions of
Washington, Jefferson, Paine, Jackson, and Lincoln" (`.../1944/05/0522-cpa-constitution.pdf`)
— national-patriotic lineage replacing class-vanguard language, plus renunciation of party
form ("a non-party organization of Americans"). This mild, non-fascist ancestor shows
CLASS_ANALYSIS erosion via nationalist-unity framing does not require fascist content to be
damaging; it can present as patriotic reformism and still hollow out the vanguard.

### 1.7 The trap that blinds, not just constrains — KPD's "social fascism" line and the Red Referendum

The doctrine also degrades *perception*, not only available actions. The KPD's "social
fascism" line (SPD as the "main enemy") masked the growing NSDAP threat right up to the 1930
Nazi election breakthrough: *"the KPD paid scarcely any attention to the monster that was
daily growing stronger... within their midst"* (`.../subject/fascism/blick/ch18.htm`). In
August 1931 the KPD tactically bloc'd with the NSDAP and DNVP on the "Red Referendum" to
dissolve the SPD-led Prussian Landtag: 9,000,000 votes, 37% of the Prussian electoral roll —
short of the majority needed (`.../blick/ch20.htm`). This is "the fascist who thought he was a
communist" running in reverse — an orthodox party fraternizing tactically with actual
fascists under "united front from below" logic, without ever relabeling its own tags.

### 1.8 Reversibility, and recurrence into the present

The drift is not one-directional or irreversible. PRRWO adopted a "divided nation" thesis on
Puerto Ricans in the U.S. in December 1970 and formally repudiated it at its July 1972
Congress — roughly 19 months — after intensive study of Stalin's national-question texts *and*
an organizational crisis (a split two months before the Congress)
(`history/erol/ncm-1/prrwo-history.htm`). Correction required both theoretical investment and
crisis pressure together, not theory alone.

The pattern also recurs in the present, per **ProleWiki encyclopedia entries** (not primary
sources, flagged as such): "MAGA Communism" is explicitly compared to Strasserism by its own
adherents for "mutual opposition to finance capital and 'globalism'... pro-rural, pro-small
business, ultranationalist attitude" (`prolewiki/Exports/Main/MAGACommunism.txt`, Aug 2022);
ProleWiki's purge of "patsocs" that same period led directly to the founding of InfraWiki
roughly two months later, October 2022 (`prolewiki/Exports/Main/Infrared.txt`), and the same
current runs an explicit entryist "CPUSA 2036" strategy targeting capture of an existing
legitimate vanguard party rather than founding a new fringe org (same source). A named
2013–2018 US organization, the Traditionalist Worker Party, fused worker-appeal rhetoric with
explicit Strasserism before its five-year dissolution
(`prolewiki/Exports/Main/Traditionalist Worker Party.txt`). Contemporary Hoxhaist doctrine
explicitly names and rejects the whole family: *"We are fully opposed to 'National Communism',
'Patriotic Socialism', 'National Bolshevism'..."*
(`prolewiki/Exports/Essays/Essay_What is Hoxhaism.txt`), confirming these are recognized,
nameable deviation categories in the tradition the game models, not an invented taxonomy.

---

## 2. Design Recommendations

**Assumption flagged:** the historical timescales below are converted assuming the standard
"1 tick ≈ 1 week" convention used elsewhere in this design series (52 ticks/year). No source
in this corpus states the Babylon engine's actual tick-to-calendar mapping; confirm against
`GameDefines`/`SimulationConfig` before hardcoding and adjust the derived ticks proportionally
if the real mapping differs.

### 2.1 Pipeline structure — five nodes, near-zero TL cost

Recommend modeling the PatSoc Pipeline as a linear DAG chain matching the historical sequence
found in §1.3–1.5, each node a specific, nameable substitution rather than a generic
"nationalism +1" slider:

1. **Capital-Fraction Splitting** (DAP 1918 precursor, §1.4): introduces a `national_capital` /
   `finance_capital` distinction that reads as economically radical while dropping
   class-relation analysis of *productive* capital. **TL cost: 0** — grounded in the
   Luther-to-Feder ~390-year recurrence showing this move needs no fresh theoretical
   investment (`.../blick/ch08.htm`). No historical number exists for a cost beyond this
   qualitative point; `TL_COST_NODE1 = 0` is design-law-grounded, not playtest-calibrated.
2. **Council/Vocabulary Fusion** (Hamburg, §1.3): NATIONAL_CHAUVINISM tag rises while CLASS_ANALYSIS is
   *not yet* zeroed — both present, doctrine reads as legitimate fusion (matches the ABB
   control case, §1.2). This node must NOT auto-flag, since ABB shows the same co-occurrence
   can be stable for years. **Recommended check:** does the node retain an explicit
   class-relation clause (ABB's chattel-vs-wage-slavery framing survives; Hamburg's "elements
   of national unity" does not). Model as a boolean `class_analysis_load_bearing` flag set at
   node-purchase time, not a continuous score — see §3.
3. **Capital-Fraction → Elite Capture** (Garvey/UNIA, §1.2): triggers when leader compensation
   exceeds ordinary-cadre compensation by the historically observed ratio. **Derivation:**
   Garvey drew $22,000/yr ($12,000 UNIA + $10,000 Black Star Line) against ordinary officer pay
   of $3,000/yr = **7.33×** (`history/usa/groups/abb/1921/1210-lorenzo-negrolib.pdf`). Recommend
   `PATSOC_ELITE_CAPTURE_RATIO = 7.0` (rounded down from the single historical data point —
   flag single-source, needs playtest confirmation).
4. **Fraternization / Tactical Bloc** (KPD-Nazi 1923 meetings + 1931 Red Referendum, §1.3/1.7):
   an explicit alliance-action with a fascist-tagged faction. **Derivation:** the Red
   Referendum drew 37% of the Prussian electoral roll against a >50% requirement
   (`.../blick/ch20.htm`). Recommend `PATSOC_BLOC_FAILURE_THRESHOLD = 0.50` (simple majority)
   with the historical ~0.37 outcome as a mid-range playtest reference — one data point from
   one electoral system, not universal.
5. **Faction Flip / Strasserism terminal** (Otto Strasser/Stennes, §1.5): triggers the
   mechanical faction-flip ending. **Derivation:** Hitler's discipline decree (28 March 1931)
   to Stennes's expulsion took about one week (`.../ch20.htm`). Recommend
   `PATSOC_FLIP_RESOLUTION_TICKS = 1` (one weekly tick) — a genuinely fast, near-instantaneous
   event, unlike the multi-year Congress rectification cadences documented for the other three
   trunks elsewhere in this research program.

### 2.2 Trigger formula — AND-conditions, not a single scalar

Historically the trap did not fire on a meter crossing a line; it fired on *concrete actions*
(fraternization meetings, referendum blocs, salary disclosures). Recommend the mechanical
trigger for the terminal faction-flip require **all** of:

- `CLASS_ANALYSIS_tag < ε` (node 2's `class_analysis_load_bearing` flag is false)
- `NATIONALISM_tag > θ_high` (post node-1/2 accumulation)
- an explicit fraternization/bloc action taken this tick (node 4) — **not** merely doctrinal
  drift with no corresponding action, matching the KPD case where the trap required an actual
  referendum bloc, not just rhetoric
- `elite_capture_ratio > PATSOC_ELITE_CAPTURE_RATIO` OR an external-patron subsidy event fired

No single source gives a combined numeric formula; this AND-structure is a design inference
from the pattern across §1.3–1.7, not a sourced coefficient, and needs playtest tuning of ε
and θ_high.

### 2.3 External patron subsidy coefficient

**Derivation:** RM 10,000/month paid to Gregor Strasser from spring 1931 by the
Rhenish-Westphalian coal syndicate, specifically to sustain an internal counterweight faction
(`.../blick/ch20.htm`). No modern-currency conversion is defensible from this single 1931
Reichsmark figure without an inflation model this brief lacks — model instead as a **discrete
event type** ("external patronage offer"): an NPC capital-faction offers a PatSoc-tagged
faction a resource subsidy in exchange for a doctrine concession, conditioned on purging the
faction's most class-analysis-retaining members (Schacht's condition was explicitly
"sacrificing the Strasser brothers"). Magnitude needs playtest calibration.

### 2.4 Reversal / rectification path

**Derivation:** PRRWO's divided-nation-theory repudiation took ~19 months (December 1970
adoption to July 1972 Congress) and required both theoretical-labor investment and an
organizational crisis (a split two months prior) (`history/erol/ncm-1/prrwo-history.htm`).
Recommend `PATSOC_REVERSAL_LATENCY_TICKS ≈ 82` (19 months × ~4.33 ticks/month) as a candidate
cooldown for reversing an *early-stage* PatSoc node (the terminal faction-flip, per §2.1, is
treated as non-reversible once fired), conditional on both TL investment AND a triggering
crisis — reversal attempted without a co-occurring crisis should have sharply reduced success
probability, per the historical requirement of both factors together. No source gives a
"crisis absent" probability — flag as needing playtest.

### 2.5 United-front tolerance band and node-naming

At Baku (1920), a body that was 44.4% non-Communist/nationalist delegates
(`history/international/comintern/baku/delegates.htm`) was treated as a legitimate united
front, not a PatSoc drift instance. Recommend the tag-vector scoring NOT fire any
PatSoc-pipeline warning purely from NATIONAL_CHAUVINISM-tag presence up to roughly this band
(`PATSOC_SAFE_NATIONALISM_SHARE ≈ 0.44`) provided the `class_analysis_load_bearing` flag
(§2.1 node 2) remains true — a single historical reference point, order-of-magnitude only.
Separately, recommend naming pipeline nodes/flavor text from the Manifesto's own taxonomy —
Feudal, Petit-Bourgeois, and "German/'True'" Socialism — plus the modern self-applied names
"National Bolshevism" and "Patriotic Socialism," both explicitly named and rejected by
contemporary Hoxhaist doctrine (`prolewiki/Exports/Essays/Essay_What is Hoxhaism.txt`), rather
than inventing new terminology.

---

## 3. Tensions and Open Questions

**1. Vocabulary alone cannot distinguish legitimate fusion from drift.** The ABB (§1.2, stable
for years) and Hamburg National Bolshevism (§1.3, collapsed in ~2-3 years) use structurally
similar language — "class interests" and "national unity" appear in both. The CCP's own 1963
critique of Soviet aid to India names the actual test: material benefit, not vocabulary —
"they often take an attitude of great-power chauvinism and national egoism... under
[internationalist] language" (`subject/china/documents/polemic/neocolon.htm`). This argues the
tag-vector scoring needs a "who benefits materially" check, much harder to compute
mechanically than a keyword or tag-weight scan — flagged as the core design risk of this
mechanic, not a solved problem. Recommend prototyping the `class_analysis_load_bearing` flag
(§2.1) as a first pass, expecting refinement, since ABB vs. Hamburg were distinguishable only
in hindsight, over years.

**2. The drift is symmetric, not one-directional.** §1.7 (KPD Red Referendum) shows the
identical "fraternize with the nationalist-right for tactical advantage" move made by an
*orthodox* Communist party with no doctrine change at all — a fascist-adjacent action without
a fascist-tagged doctrine. The pipeline may need a second, non-doctrinal entry ramp: a purely
tactical bloc action carrying reputational cost independent of the tag vector. Whether this
belongs inside the DT PatSoc Pipeline or as a separate OODA/diplomatic-action risk is an open
architecture question this brief cannot resolve from the evidence alone.

**3. The five-node chain length is a design inference, not a sourced number.** No document in
this corpus specifies "five stages"; the mapping in §2.1 (capital-splitting → vocabulary
fusion → elite capture → fraternization → flip) is this brief's synthesis of the historical
sequence across four national cases (Germany twice, 1918-23 and 1930-31; the US CPUSA 1944;
the ABB as counter-example). The *existence* of a multi-step, low-cost sequence is well
grounded (§1.3, §1.4); the specific count of five is not, and should be treated as a
placeholder pending game-design fit.

**4. Several requested numbers have no historical figure, and the ones that exist are single
data points.** The corpus has no data on the size/lifespan of the 1923 "Group of German
Communist Army Officers" (§1.3, needed for a minimum viable cell size), a modern-currency
conversion for the RM 10,000/month patronage figure (§2.3), or a reversal-success probability
absent a co-occurring crisis (§2.4) — all marked "needs playtest calibration," not given
fabricated numbers. The Baku 44.4% figure and the Garvey 7.33× ratio (§2.5, §2.1) are each the
*only* quantified instance the corpus yielded for their mechanic; treat both as
order-of-magnitude anchors pending confirmation from additional cases, not validated
constants.

**6. Reversibility contradicts a strict "no optimal path" reading of the terminal node.** §2.1
treats the faction-flip as effectively non-reversible once fired (matching Wolffheim's fate and
the absence of any "return to class politics" case in the corpus), while §2.4 shows an
*early-stage* national-question error (PRRWO) is fully reversible with sufficient investment.
The design needs an explicit answer for where along the five-node chain reversibility stops —
this brief recommends "before node 4 (fraternization action)," the node with a real historical
action attached, but this is an inference, not a sourced cutoff.
