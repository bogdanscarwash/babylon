# Research seed — the tick compressor and the narrative vector space (BD, 2026-07-21)

> BD (verbatim intent): "if language models are essentially neural network powered
> compression machines for text input, and what babylon generates every tick is a
> text-based input that's highly structured and meaningful, can't we eventually fine
> tune a model that is task-designed to 'compress' each tick of the entire Babylon
> data, with some kind of narrative vector space and map things onto that?"

STATUS: **DEFERRED research seed** — post-1.0, activates with the AI-system phase
(fine-tuning is post-1.0 by standing BD ruling). Recorded now so the idea and its
groundings survive. Nothing here is scheduled; nothing here may touch the v1.0 trains.

## The claim, strengthened

Compression ≅ prediction: a model that compresses Babylon tick-text well has *learned
the engine's physics prior*. Babylon is an unusually strong candidate because:

- Tick output is narrow-distribution, highly structured, semantically dense text —
  exactly what a task-tuned small model compresses far better than a general one.
- **Determinism makes training data free and clean**: any (seed, defines) replays to
  byte-identical state sequences. Every campaign is a dataset; SessionRecorder
  (black-box replay) and `persistence/archival.py` parquet export are the extraction
  surfaces that already exist.

## The two payoffs

1. **Narrative vector space**: embed tick-states (or tick-deltas / windowed
   trajectories) such that distance = narrative similarity and trajectories = story
   arcs. Uses: narrator retrieval keyed by narrative STATE rather than keywords;
   campaign⇄history rhyming against the historical-event-atlas corpus ("this campaign
   is nearest the 1917 Petrograd trajectory") as a nearest-neighbor query; the
   "wind is blowing" trend digest as a decode from the trajectory tangent.
2. **Compression ratio as salience** (the deep one): surprisal IS newsworthiness. A
   tick the compressor finds expensive is a tick where the learned prior broke —
   a principled, non-hand-tuned salience/severity signal feeding chronicle salience,
   eventually grounding the derived-severity catalog in information theory instead of
   curation. Also the anomaly detector for playtesting (ticks that don't compress =
   emergent behavior worth eyes-on).

Secondary uses: learned codec for chronicle/replay compaction (save-file compression);
distillation target for a tick-summarizer that runs cheaper than the full narrator.

## Constitutional boundaries (all already ratified — the seed fits inside them)

- **AI observes, never adjudicates** (Amendment V/Y): the vector space and all its
  consumers are P-tier projections. Nothing derived from the model may feed physics;
  salience-from-surprisal drives PRESENTATION (chronicle ordering, autopause
  candidates) only via the same read-only lanes severity uses today.
- **Determinism**: embeddings are not byte-reproducible across model versions — never
  in the tick hash, excluded from verify stories exactly like `narrative/**`. The
  training DATA is deterministic; the trained artifact is a pinned, versioned blob.
- **Local-first**: train offline (dev box or rented GPU, out-of-band); ship weights
  via the ADR096 R2 signed-manifest lane like the narrator GGUFs; inference local.
- **Model pins**: base candidates are the shipped families (Llama 3.1 8B for
  generation; embeddinggemma-300m/768 as the embedding seam — a tick-tuned embedding
  head could stay 768-dim to drop into the existing pgvector schema unchanged).

## Existing seams it lands on (no new architecture required to start)

- pgvector Archive (768-dim, per-campaign embedding pin) — the vector store.
- SessionRecorder + parquet export — dataset extraction.
- `intelligence/rag/` chunking + PgVectorStore — ingestion machinery.
- Historical-event-atlas corpus (ai/_inbox/math/historical-event-atlas.md + the
  marxists.org atlas material) — the reference trajectories to rhyme against.
- Derived severity catalog (T1.1) — the hand-tuned baseline the surprisal signal
  would be evaluated AGAINST before ever supplementing it.

## Activation checklist (when the AI phase opens)

1. Dataset spec: canonical tick-text serialization (stable field order — the
   projection layer's deterministic renderings, not raw dicts).
2. Baseline: gzip/zstd + general-LLM perplexity on tick corpus → the compression
   floor any fine-tune must beat to matter.
3. Spike: embedding head fine-tune (768-dim) on tick windows; evaluate narrative-
   neighbor quality against hand-labeled "rhyming" episodes.
4. Surprisal-salience shadow: log per-tick compression cost alongside the derived
   severity tiers for N campaigns; correlate before proposing any consumer.
5. BD gate: any consumer beyond shadow logging is an Amendment-S-style ruling
   (observation vs feedback boundary).
