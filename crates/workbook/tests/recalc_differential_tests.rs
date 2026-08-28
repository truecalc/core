//! A randomized differential harness for `incremental ≡ full`.
//!
//! ## Why this file exists
//!
//! `recalc_incremental` promises the same grid a fresh `recalc` produces. Two
//! shipped defects (#926, #930) violated that promise while every
//! value-asserting suite in the repo stayed green, because each of those suites
//! asserts a hand-chosen cell of a hand-chosen shape: they can only catch the
//! divergences somebody already thought of. The instrument that does catch this
//! class is a generator — build a random workbook, apply a random edit, and
//! compare the *whole* grid (canonical JSON) against a full recalc on a clone.
//!
//! ## What it generates
//!
//! Three shapes. `Plain` discriminates from the other two outright — it never
//! generates an array literal at all. `SpillHeavy` and `ConditionalArray` both
//! generate arrays, but at different rates (`extra` differs between them) and
//! diverge further in `apply_edit`, whose footprint-changes-mid-recalc edit
//! only `ConditionalArray` applies. That `extra` gap is load-bearing: giving
//! both shapes the same `extra` made their `build()` draw from the identical
//! distribution, so for a given seed the two shapes produced byte-identical
//! workbooks and differed only via that one `apply_edit` arm — the harness
//! looked like it ran three independent generators when it only ran two.
//!
//!  * [`Shape::Plain`] — two sheets, cross-sheet references, defined names and
//!    name **retargeting**, ranges, `TODAY`/`RAND`, `clear`, and
//!    formula↔literal edits.
//!  * [`Shape::SpillHeavy`] — the above plus array anchors that shrink, grow,
//!    collapse to a scalar, block and unblock.
//!  * [`Shape::ConditionalArray`] — anchors whose **footprint changes mid
//!    recalc** (`=IF(cond,{1;2;3},{1;2})`), including cells that read inside
//!    their own spill footprint. This is the sharpest of the three: it is the
//!    only one that reliably reproduces the self-referential-spill divergence
//!    class.
//!
//! ## Scale
//!
//! The committed run is deliberately small enough to sit in `cargo test`.
//! `TRUECALC_DIFF_SEEDS` raises the per-shape seed count and
//! `TRUECALC_DIFF_SEED_BASE` shifts the seed window, so the same code runs at
//! the thousands-of-seeds scale used to qualify a change:
//!
//! ```text
//! TRUECALC_DIFF_SEEDS=4000 cargo test --release -p truecalc-workbook \
//!     --test recalc_differential_tests
//! ```
//!
//! ## Reading a failure
//!
//! A failure prints the shape, the seed, the edit index, and both grids. Re-run
//! that one seed with `TRUECALC_DIFF_SEED_BASE=<seed> TRUECALC_DIFF_SEEDS=1`.

use std::collections::BTreeSet;

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

// ---------------------------------------------------------------------------
// Deterministic RNG (SplitMix64) — no dev-dependency, and a seed reproduces a
// workbook exactly.
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xDEAD_BEEF_CAFE_F00D)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `0..n`. `n` must be non-zero.
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }

    /// [`pick`](Self::pick) for the `&'static str` tables, which want the
    /// string itself rather than a reference to the table slot.
    fn pick_str(&mut self, xs: &[&'static str]) -> &'static str {
        xs[self.below(xs.len())]
    }
}

// ---------------------------------------------------------------------------
// Workbook geometry. Small enough that a divergence is readable, large enough
// that a spill has somewhere to go.
// ---------------------------------------------------------------------------

const SHEETS: [&str; 2] = ["S", "T"];
const ROWS: u32 = 7;
const COLS: u32 = 4;

/// Named-range targets the generator defines and retargets between. A mix of
/// single cells and ranges, on both sheets, so name indirection is exercised in
/// both of its shapes.
const NAME_REFS: [&str; 6] = ["S!A1", "S!B2", "S!A1:A3", "T!A1", "T!A1:B2", "S!C1:C3"];

fn addr(row: u32, col: u32) -> Address {
    Address::new(row, col).expect("in-bounds address")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

/// A later context, so `TODAY()` genuinely moves between the two recalcs a
/// volatile cell is compared across.
fn ctx_later() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000 + 86_400_000, "Etc/GMT", 0).expect("valid tz")
}

/// The cells a canonical `Sheet!A1` / `Sheet!A1:B2` reference covers — what a
/// caller must report as `edited` when a name is retargeted onto or off it.
fn cells_of_ref(r: &str) -> Vec<(String, Address)> {
    let (sheet, a1) = r.split_once('!').expect("canonical ref carries a sheet");
    let (start, end) = match a1.split_once(':') {
        Some((s, e)) => (
            Address::from_a1(s).expect("valid A1"),
            Address::from_a1(e).expect("valid A1"),
        ),
        None => {
            let a = Address::from_a1(a1).expect("valid A1");
            (a, a)
        }
    };
    let mut out = Vec::new();
    for row in start.row..=end.row {
        for col in start.column..=end.column {
            out.push((sheet.to_owned(), addr(row, col)));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Formula generation
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Plain,
    SpillHeavy,
    ConditionalArray,
}

impl Shape {
    fn name(self) -> &'static str {
        match self {
            Shape::Plain => "plain",
            Shape::SpillHeavy => "spill-heavy",
            Shape::ConditionalArray => "conditional-array",
        }
    }
}

/// A reference string to some cell in the workbook — same sheet (bare) or the
/// other sheet (qualified), so cross-sheet edges appear.
fn ref_text(rng: &mut Rng, own_sheet: &str) -> String {
    let sheet = rng.pick_str(&SHEETS);
    let a = addr(
        1 + rng.below(ROWS as usize) as u32,
        1 + rng.below(COLS as usize) as u32,
    );
    if sheet == own_sheet {
        a.to_a1()
    } else {
        format!("{sheet}!{}", a.to_a1())
    }
}

/// A range reference string. Deliberately allowed to span rows a spill can
/// reach, so a range reader can straddle a footprint that grows or shrinks.
fn range_text(rng: &mut Rng, own_sheet: &str) -> String {
    let sheet = rng.pick_str(&SHEETS);
    let r1 = 1 + rng.below(ROWS as usize) as u32;
    let r2 = 1 + rng.below(ROWS as usize) as u32;
    let c1 = 1 + rng.below(COLS as usize) as u32;
    let c2 = 1 + rng.below(COLS as usize) as u32;
    let start = addr(r1.min(r2), c1.min(c2));
    let end = addr(r1.max(r2), c1.max(c2));
    let body = format!("{}:{}", start.to_a1(), end.to_a1());
    if sheet == own_sheet {
        body
    } else {
        format!("{sheet}!{body}")
    }
}

/// One formula, drawn from the pool the shape calls for.
fn formula(rng: &mut Rng, shape: Shape, own_sheet: &str, names: &[String]) -> String {
    // The common pool: every shape gets ordinary arithmetic, ranges, names and
    // volatiles, so the array-focused shapes are still realistic workbooks.
    let common = 10usize;
    let extra = match shape {
        Shape::Plain => 0,
        // Narrower than `ConditionalArray`'s: `SpillHeavy` only ever draws
        // arms 10/11 (plain array anchors), never the conditional-footprint
        // arms 12/13, which `ConditionalArray` alone should generate. Giving
        // both shapes `extra = 4` made the two draw from the same
        // distribution, so `build()` produced byte-identical workbooks for a
        // given seed and the shapes diverged only via `apply_edit`'s arm 6.
        Shape::SpillHeavy => 2,
        Shape::ConditionalArray => 4,
    };
    match rng.below(common + extra) {
        0 => format!("={}+{}", ref_text(rng, own_sheet), ref_text(rng, own_sheet)),
        1 => format!("={}*2", ref_text(rng, own_sheet)),
        2 => format!("=SUM({})", range_text(rng, own_sheet)),
        3 => format!("=COUNT({})", range_text(rng, own_sheet)),
        4 => format!(
            "=IF({}>2,{},{})",
            ref_text(rng, own_sheet),
            ref_text(rng, own_sheet),
            rng.below(9)
        ),
        5 => format!("={}+1", rng.pick(names)),
        6 => format!("=SUM({})", rng.pick(names)),
        7 => format!("={}-{}", ref_text(rng, own_sheet), rng.below(5)),
        8 => "=TODAY()".to_owned(),
        9 => format!("=RAND()*0+{}", ref_text(rng, own_sheet)),
        // Spill-heavy extras: plain array anchors of varying footprint.
        10 => array_literal(rng),
        11 => format!("={}+{}", array_literal_body(rng), ref_text(rng, own_sheet)),
        // Conditional-array extras: the footprint depends on a value computed
        // during the same recalc, so it can change mid-recalc.
        12 => format!(
            "=IF({}>{},{},{})",
            ref_text(rng, own_sheet),
            rng.below(5),
            array_literal_body(rng),
            array_literal_body(rng)
        ),
        _ => format!(
            "=IF({}>{},{},{})",
            ref_text(rng, own_sheet),
            rng.below(5),
            array_literal_body(rng),
            rng.below(9)
        ),
    }
}

/// The `{...}` body of an array literal, 1..=3 elements, vertical or horizontal.
fn array_literal_body(rng: &mut Rng) -> String {
    let n = 1 + rng.below(3);
    let sep = if rng.below(2) == 0 { ";" } else { "," };
    let elems: Vec<String> = (0..n)
        .map(|i| ((i + 1) * (1 + rng.below(4))).to_string())
        .collect();
    format!("{{{}}}", elems.join(sep))
}

fn array_literal(rng: &mut Rng) -> String {
    format!("={}", array_literal_body(rng))
}

// ---------------------------------------------------------------------------
// Workbook construction
// ---------------------------------------------------------------------------

fn build(rng: &mut Rng, shape: Shape) -> (Workbook, Vec<String>) {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in SHEETS {
        wb.add_sheet(Worksheet::new(s)).expect("fresh sheet name");
    }

    let names: Vec<String> = vec!["NAMEA".to_owned(), "NAMEB".to_owned()];
    for n in &names {
        wb.define_name(n, rng.pick_str(&NAME_REFS))
            .expect("valid ref");
    }

    for sheet in SHEETS {
        for row in 1..=ROWS {
            for col in 1..=COLS {
                match rng.below(10) {
                    // A sparse grid: some cells stay unauthored, which is the
                    // population the range seeding rule is about.
                    0..=2 => {}
                    3..=5 => {
                        wb.set(
                            sheet,
                            addr(row, col),
                            CellInput::Literal(Value::Number(rng.below(10) as f64)),
                        )
                        .expect("literal");
                    }
                    _ => {
                        let f = formula(rng, shape, sheet, &names);
                        // A generated formula is always syntactically valid;
                        // `set` still validates, so surface a generator bug.
                        wb.set(sheet, addr(row, col), CellInput::Formula(f.clone()))
                            .unwrap_or_else(|e| panic!("generated formula {f} rejected: {e:?}"));
                    }
                }
            }
        }
    }
    (wb, names)
}

// ---------------------------------------------------------------------------
// Edits
// ---------------------------------------------------------------------------

/// Applies one random edit and returns the cells to report as `edited`.
fn apply_edit(
    rng: &mut Rng,
    wb: &mut Workbook,
    shape: Shape,
    names: &[String],
) -> Vec<(String, Address)> {
    let sheet = rng.pick_str(&SHEETS);
    let a = addr(
        1 + rng.below(ROWS as usize) as u32,
        1 + rng.below(COLS as usize) as u32,
    );
    match rng.below(10) {
        0 | 1 => {
            wb.set(
                sheet,
                a,
                CellInput::Literal(Value::Number(rng.below(20) as f64)),
            )
            .expect("literal");
            vec![(sheet.to_owned(), a)]
        }
        2 => {
            wb.clear(sheet, a);
            vec![(sheet.to_owned(), a)]
        }
        3 => {
            // Retarget a name: the caller's contract is to report the name's
            // old *and* new target cells.
            let name = rng.pick(names).clone();
            let old = wb
                .name(&name)
                .map(|n| n.r#ref.clone())
                .expect("the name exists");
            let new = rng.pick_str(&NAME_REFS).to_owned();
            wb.redefine_name(&name, &new).expect("valid ref");
            let mut edited = cells_of_ref(&old);
            edited.extend(cells_of_ref(&new));
            edited
        }
        4 | 5 if shape != Shape::Plain => {
            wb.set(sheet, a, CellInput::Formula(array_literal(rng)))
                .expect("array literal");
            vec![(sheet.to_owned(), a)]
        }
        6 if shape == Shape::ConditionalArray => {
            let f = format!(
                "=IF({}>{},{},{})",
                ref_text(rng, sheet),
                rng.below(5),
                array_literal_body(rng),
                array_literal_body(rng)
            );
            wb.set(sheet, a, CellInput::Formula(f)).expect("formula");
            vec![(sheet.to_owned(), a)]
        }
        _ => {
            let f = formula(rng, shape, sheet, names);
            wb.set(sheet, a, CellInput::Formula(f.clone()))
                .unwrap_or_else(|e| panic!("generated formula {f} rejected: {e:?}"));
            vec![(sheet.to_owned(), a)]
        }
    }
}

// ---------------------------------------------------------------------------
// The differential itself
// ---------------------------------------------------------------------------

/// Runs one seed: build, full-recalc, then a short edit script, comparing
/// `incremental` against a fresh `full` on a clone after every edit.
///
/// Returns `Err` with a readable report at the first divergence.
fn run_seed(shape: Shape, seed: u64) -> Result<(), String> {
    let mut rng = Rng::new(seed);
    let (mut live, names) = build(&mut rng, shape);
    live.recalc(&ctx());

    for step in 0..6 {
        let edited = apply_edit(&mut rng, &mut live, shape, &names);
        // Alternate the context so `TODAY()` moves under an incremental recalc
        // exactly as it does under a full one.
        let c = if step % 3 == 2 { ctx_later() } else { ctx() };

        let mut full = live.clone();
        full.recalc(&c);
        live.recalc_incremental(&c, &edited);

        let got = live.to_json().expect("serializable");
        let want = full.to_json().expect("serializable");
        if got != want {
            let touched: Vec<String> = edited
                .iter()
                .map(|(s, a)| format!("{s}!{}", a.to_a1()))
                .collect();
            return Err(format!(
                "shape={} seed={seed} step={step} edited={touched:?}\n\
                 incremental: {got}\n\
                 full:        {want}",
                shape.name()
            ));
        }
    }
    Ok(())
}

/// Per-shape seed count, overridable for a qualification run.
fn seed_count() -> u64 {
    std::env::var("TRUECALC_DIFF_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

fn seed_base() -> u64 {
    std::env::var("TRUECALC_DIFF_SEED_BASE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn sweep(shape: Shape) {
    let base = seed_base();
    let n = seed_count();
    let mut failures = 0usize;
    let mut reports: Vec<String> = Vec::new();
    for seed in base..base + n {
        if let Err(report) = run_seed(shape, seed) {
            failures += 1;
            // Printed for every divergence, not just the reported ones: a seed
            // is the whole reproduction, and a new discriminating seed belongs
            // in `pinned_regression_seeds` below.
            println!("  {} divergence at seed {seed}", shape.name());
            if reports.len() < 3 {
                reports.push(report);
            }
        }
    }
    println!("{}: {failures}/{n} divergences", shape.name());
    assert!(
        failures == 0,
        "{failures} divergence(s) over {n} seeds from {base} on shape {}:\n\n{}",
        shape.name(),
        reports.join("\n\n")
    );
}

#[test]
fn plain_workbooks_agree() {
    sweep(Shape::Plain);
}

#[test]
fn spill_heavy_workbooks_agree() {
    sweep(Shape::SpillHeavy);
}

/// The sharpest of the three: arrays whose footprint depends on a value
/// computed during the same recalc, so an anchor can spill onto a cell that is
/// read by a formula which has already been evaluated this pass — including a
/// cell that reads inside its **own** footprint.
#[test]
fn conditional_array_workbooks_agree() {
    sweep(Shape::ConditionalArray);
}

/// Seeds that are known to discriminate, run every time regardless of the seed
/// budget.
///
/// The sweeps above default to a few hundred seeds so they fit in an ordinary
/// `cargo test`. Every defect this harness has caught so far diverges on the
/// order of one workbook in a thousand (or rarer), which that budget does not
/// reach. A seed is a complete reproduction, so pinning the ones that
/// discriminate turns a sweep that has to get lucky into a guard that does
/// not.
///
/// `SpillHeavy`'s seeds in each group are tied to its `extra` weight in
/// [`formula`]: changing that weight changes every `SpillHeavy` workbook this
/// harness generates from a given seed, so a seed pinned before a weight
/// change can stop discriminating afterward without the underlying code
/// changing at all. `Plain` and `ConditionalArray` seeds are not affected by
/// `SpillHeavy`'s weight.
///
/// Add to this list whenever a sweep at scale finds a new divergence — the
/// failure output prints the seed for exactly that purpose.
#[test]
fn pinned_regression_seeds() {
    // Group 1 — the name branch of spill seeding. These diverge if a
    // `Precedent::Name` stops being put through its target's rule.
    const NAME_SEEDING: [(Shape, u64); 5] = [
        (Shape::Plain, 1219),
        (Shape::SpillHeavy, 2602),
        (Shape::SpillHeavy, 2835),
        (Shape::ConditionalArray, 723),
        (Shape::ConditionalArray, 6279),
    ];
    // Group 2 — the broad "does this range hold a non-authored cell" fallback
    // of range seeding. These diverge if that fallback is dropped and a range
    // reader is seeded only when it overlaps a spill rectangle that is on the
    // grid right now.
    const RANGE_SEEDING: [(Shape, u64); 5] = [
        (Shape::Plain, 4504),
        (Shape::SpillHeavy, 2176),
        (Shape::SpillHeavy, 4025),
        (Shape::ConditionalArray, 366),
        (Shape::ConditionalArray, 6837),
    ];
    // Group 3 — the widen loop's rewind to the pre-recalc grid. These diverge
    // if a second widen attempt continues from the first attempt's output
    // instead of restarting from the grid the recalc began with: a cell that
    // reads inside its own spill footprint then lands on the opposite phase
    // from the full recalc.
    //
    // These three no longer diverge under the broad range fallback above
    // (issue #949): broadening range seeding back out pulls a
    // self-referential-footprint reader into the *first* widen pass more
    // often, which is exactly the situation the rewind exists for, so the
    // second pass rarely runs at all against this generator any more. A sweep
    // of 100,000 seeds per shape with the rewind disabled found no divergence
    // either. Kept as a passive regression check — they still pass, and would
    // matter again if range seeding narrows in the future — but they are not
    // load-bearing evidence for the rewind today; see PR #948's review.
    const WIDEN_REWIND: [(Shape, u64); 3] = [
        (Shape::SpillHeavy, 4494),
        (Shape::SpillHeavy, 6916),
        (Shape::ConditionalArray, 6657),
    ];
    // Group 4 — a spill footprint retired by a *previous* recalc rather than
    // the edit under test (issue #949). These diverge if range seeding falls
    // back to "overlaps a spill rectangle on the grid now" without also
    // catching a non-authored cell the current grid can no longer explain.
    // See also the named reproduction,
    // `a_range_reader_sees_a_footprint_a_previous_recalc_retired`.
    const PREVIOUS_RECALC_VACATED_FOOTPRINT: [(Shape, u64); 1] = [(Shape::SpillHeavy, 4520)];

    let mut failures: Vec<String> = Vec::new();
    for (group, seeds) in [
        ("name seeding", &NAME_SEEDING[..]),
        ("range seeding", &RANGE_SEEDING[..]),
        ("widen-loop rewind", &WIDEN_REWIND[..]),
        (
            "previous-recalc vacated footprint",
            &PREVIOUS_RECALC_VACATED_FOOTPRINT[..],
        ),
    ] {
        for &(shape, seed) in seeds {
            if let Err(report) = run_seed(shape, seed) {
                failures.push(format!("[{group}] {report}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} pinned seed(s) diverged:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

/// A fixed, non-random instance of the self-referential-spill shape: `C1`'s own
/// array spills into `C2`, and `C1` reads `C2`. Kept alongside the sweeps
/// because a named shape is what a future reader debugs against, and because it
/// runs in microseconds regardless of the seed budget.
#[test]
fn a_cell_reading_inside_its_own_spill_footprint_matches_a_full_recalc() {
    let mut live = Workbook::new(EngineFlavor::Sheets);
    live.add_sheet(Worksheet::new("S")).unwrap();
    live.set("S", addr(5, 2), CellInput::Formula("={8;4}".into()))
        .unwrap(); // B5 spills B5:B6
    live.set("S", addr(1, 3), CellInput::Formula("=C2+B5".into()))
        .unwrap(); // C1's array spills into C2 — it reads its own footprint
    live.set("S", addr(5, 5), CellInput::Formula("=IF(D6>2,C2,3)".into()))
        .unwrap();
    live.set("S", addr(4, 1), CellInput::Formula("=E5+1".into()))
        .unwrap();
    live.recalc(&ctx());

    live.set("S", addr(5, 2), CellInput::Formula("={9;5}".into()))
        .unwrap();
    let mut full = live.clone();
    full.recalc(&ctx());
    live.recalc_incremental(&ctx(), &[("S".to_owned(), addr(5, 2))]);

    assert_eq!(
        live.to_json().unwrap(),
        full.to_json().unwrap(),
        "a cell reading inside its own spill footprint must converge to the \
         full-recalc grid"
    );
}

/// A fixed, non-random instance of the class no edit-local reasoning can
/// recover from: a spill footprint retired by a *previous* recalc, not the
/// edit under test.
///
/// `B4`'s array spills onto `B6` while the grid settles; `T!D5` sums a range
/// covering `B6` and stores a value computed against that spill. By the time
/// the grid is settled, `B4` no longer holds an array, and `B4` was never an
/// edited cell — the edit below only ever touches `B3`. A rule that reasons
/// from the edit's own cells about which footprint could have vacated has
/// nothing to find `B4` from, so `D5` keeps a stale value (issue #925
/// follow-up).
#[test]
fn a_range_reader_sees_a_footprint_a_previous_recalc_retired() {
    let mut live = Workbook::new(EngineFlavor::Sheets);
    live.add_sheet(Worksheet::new("S")).unwrap();
    live.add_sheet(Worksheet::new("T")).unwrap();
    live.set("S", addr(6, 1), CellInput::Literal(Value::Number(4.0)))
        .unwrap(); // A6
    live.set("S", addr(3, 4), CellInput::Literal(Value::Number(9.0)))
        .unwrap(); // D3
    live.set("S", addr(3, 2), CellInput::Formula("=IF(B6>2,4,8)".into()))
        .unwrap(); // B3
    live.set(
        "S",
        addr(4, 2),
        CellInput::Formula("=IF(B3>4,{1;8;12},4)".into()),
    )
    .unwrap(); // B4
    live.set("T", addr(5, 4), CellInput::Formula("=SUM(S!A6:B6)".into()))
        .unwrap(); // D5

    // Two full recalcs to settle: the first pass computes B3 against B6
    // vacant, spills B4 onto B6, and evaluates D5 against that spill; the
    // second pass then sees B3 = 4 and collapses B4 to a scalar, vacating B6.
    live.recalc(&ctx());
    live.recalc(&ctx());

    live.set("S", addr(3, 2), CellInput::Formula("={3,6,6}".into()))
        .unwrap(); // B3
    let mut full = live.clone();
    full.recalc(&ctx());
    live.recalc_incremental(&ctx(), &[("S".to_owned(), addr(3, 2))]);

    assert_eq!(
        live.to_json().unwrap(),
        full.to_json().unwrap(),
        "a range reader must not keep a value computed against a spill \
         footprint a previous recalc retired, even though the reader was \
         never itself edited"
    );
}

/// The harness must be able to *see* a divergence, not merely fail to produce
/// one: a workbook whose grid is deliberately left stale must compare unequal.
/// Without this, a generator that silently produced empty workbooks would look
/// like a passing suite.
#[test]
fn the_comparison_detects_a_stale_grid() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr(2, 1), CellInput::Formula("=A1+1".into()))
        .unwrap();
    wb.recalc(&ctx());

    // Edit, but never recalc: the clone's full recalc must disagree.
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(9.0)))
        .unwrap();
    let mut full = wb.clone();
    full.recalc(&ctx());
    assert_ne!(
        wb.to_json().unwrap(),
        full.to_json().unwrap(),
        "the differential comparison must be able to observe a stale grid"
    );
}

/// Every generated shape must actually produce the constructs it claims to, or
/// the sweeps above are green for the wrong reason. Checks the corpus, not one
/// workbook: any individual seed may legitimately miss a construct.
#[test]
fn the_generator_produces_the_constructs_it_claims() {
    for shape in [Shape::Plain, Shape::SpillHeavy, Shape::ConditionalArray] {
        let mut seen_formula = false;
        let mut seen_literal = false;
        let mut seen_unauthored = false;
        let mut seen_range = false;
        let mut seen_name_use = false;
        let mut seen_cross_sheet = false;
        let mut seen_volatile = false;
        let mut seen_array = shape == Shape::Plain; // not claimed for Plain
        let mut anchors: BTreeSet<String> = BTreeSet::new();

        for seed in 0..40u64 {
            let mut rng = Rng::new(seed);
            let (mut wb, _) = build(&mut rng, shape);
            wb.recalc(&ctx());
            for sheet in SHEETS {
                for row in 1..=ROWS {
                    for col in 1..=COLS {
                        let Some(cell) = wb.get(sheet, addr(row, col)) else {
                            seen_unauthored = true;
                            continue;
                        };
                        match cell.formula() {
                            None => seen_literal = true,
                            Some(f) => {
                                seen_formula = true;
                                if f.contains(':') {
                                    seen_range = true;
                                }
                                if f.contains("NAMEA") || f.contains("NAMEB") {
                                    seen_name_use = true;
                                }
                                if f.contains("S!") || f.contains("T!") {
                                    seen_cross_sheet = true;
                                }
                                if f.contains("TODAY") || f.contains("RAND") {
                                    seen_volatile = true;
                                }
                                if f.contains('{') {
                                    seen_array = true;
                                }
                            }
                        }
                        if matches!(cell.value(), Value::Array(_)) {
                            anchors.insert(format!("{sheet}!{}", addr(row, col).to_a1()));
                        }
                    }
                }
            }
        }

        let s = shape.name();
        assert!(seen_formula, "{s}: no formula cells generated");
        assert!(seen_literal, "{s}: no literal cells generated");
        assert!(
            seen_unauthored,
            "{s}: no unauthored cells — ranges are dense"
        );
        assert!(seen_range, "{s}: no range references generated");
        assert!(seen_name_use, "{s}: no name references generated");
        assert!(seen_cross_sheet, "{s}: no cross-sheet references generated");
        assert!(seen_volatile, "{s}: no volatile cells generated");
        assert!(seen_array, "{s}: no array literals generated");
        if shape != Shape::Plain {
            assert!(
                !anchors.is_empty(),
                "{s}: no array actually spilled anywhere in the corpus"
            );
        }
    }
}
