# Troll-Farm bot evaluation

Benchmark of the bot against a reference boss over many reproducible games,
with plots that explain *where* games are won or lost. This page is laid out
so a **new version's plots sit next to the baseline** for direct comparison.

## How to run

```bash
# Baseline (the version you compare against) — already generated:
python3 eval.py 1000 --seed 1 --jobs 16 --label baseline

# A new version: rebuild the bot, copy it into the referee dir, then:
cargo build -p trollfarm --release
cp ../../target/release/trollfarm assets/Troll-Farm/trollfarm
python3 eval.py 1000 --seed 1 --jobs 16 --label candidate
```

Each run writes to `eval_out/<label>/`:

- `00_overview.png` — all 12 panels in one image
- `01..12_*.png` — the individual panels (referenced below)
- `results.json` — every game (seed, scores, map, final composition)
- `summary.md` — the headline table

Because `--seed` is a base seed, **`baseline` and `candidate` play the exact
same 1000 maps**, so the comparison is apples-to-apples. Use the same
`--seed` and game count for both. The two columns below are
`eval_out/baseline/…` and `eval_out/candidate/…`; the candidate column is
blank until you run with `--label candidate`.

> How the data is collected: each game runs the referee jar with `-l` (replay
> log, no web server) — deterministic per seed, ~0.14 s/game. Scores and the
> map come from the replay; the per-turn score **trajectory** and **final
> shack composition** (both players) come from our bot's `[INV]` stderr line
> in `src/bot/play.rs` — keep that line.

## Headline numbers

Open **`eval_out/baseline/summary.md`** and **`eval_out/candidate/summary.md`**
side by side. Current baseline (current bot vs gold-X, 1000 games):

| metric | value |
|---|---|
| win rate | **7.7%** (77 W / 918 L / 5 D) |
| avg margin | **−18.7** (std 14.1) |
| avg score (us vs opp) | 99.2 vs 117.9 |
| leftover fruit (us vs opp) | **13.1 vs 0.0** |
| wood (us vs opp) | 21.5 vs 29.4 → **85.8 vs 117.7 pts** |
| avg game length | 173 turns |

**The loss is almost entirely a wood-production gap:** gold-X converts
everything into wood (0 leftover fruit), we leave ~13 fruit scoring 1 pt
instead of ~4 pt as wood, and we chop less wood overall. A better version
should push *our wood up* and *our leftover fruit down*.

---

## Panels

One brief line per panel saying what it shows, then the plot. The right column
fills in once you run a `candidate` benchmark.

### 1. Per-game scores
Our score (y) vs opponent (x); points above the diagonal are wins.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/01_score_scatter.png) | ![](eval_out/candidate/01_score_scatter.png) |

### 2. Margin distribution
Histogram of the score margin (us − opp); the mean is marked.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/02_margin_hist.png) | ![](eval_out/candidate/02_margin_hist.png) |

### 3. Margin CDF
Cumulative margin curve — fraction of games within any margin, and the loss rate where it crosses 0.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/03_margin_cdf.png) | ![](eval_out/candidate/03_margin_cdf.png) |

### 4. Win rate by map size
Win % for each map size (16×8 … 22×11); dashed line is 50%.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/04_winrate_by_mapsize.png) | ![](eval_out/candidate/04_winrate_by_mapsize.png) |

### 5. Margin vs shack distance
Margin against the Manhattan distance between the two shacks (close vs far bases).

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/05_margin_vs_shackdist.png) | ![](eval_out/candidate/05_margin_vs_shackdist.png) |

### 6. Margin vs water count
Margin against the number of water cells (water speeds up adjacent trees).

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/06_margin_vs_water.png) | ![](eval_out/candidate/06_margin_vs_water.png) |

### 7. Game length
Histogram of how many turns games last (max 300); games end early when trees run out.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/07_game_length.png) | ![](eval_out/candidate/07_game_length.png) |

### 8. Margin vs game length
Margin against game length — whether we lose more in short or long games.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/08_margin_vs_length.png) | ![](eval_out/candidate/08_margin_vs_length.png) |

### 9. Wasted fruit at game end
Leftover fruit in the shack at game end (1 pt each, unconverted) — us vs opp.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/09_wasted_fruit.png) | ![](eval_out/candidate/09_wasted_fruit.png) |

### 10. Score composition
Average points from fruit vs wood, for us and opp — where the score comes from.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/10_score_composition.png) | ![](eval_out/candidate/10_score_composition.png) |

### 11. Mean score trajectory
Mean score over game progress (0–100%), us vs opp, with a 25–75 percentile band.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/11_score_trajectory.png) | ![](eval_out/candidate/11_score_trajectory.png) |

### 12. Mean margin trajectory
Mean margin over game progress — shows *when* the gap opens during a game.

| Baseline | Candidate |
|---|---|
| ![](eval_out/baseline/12_margin_trajectory.png) | ![](eval_out/candidate/12_margin_trajectory.png) |
