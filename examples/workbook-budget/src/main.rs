//! Cookbook example: a simple budget workbook.
//!
//! Creates a workbook with two sheets (Income and Expenses), sets literal
//! values and SUM formulas, recalculates, and prints the results.
//!
//! Run:
//!   cargo run -p workbook-budget

use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).expect("valid A1 address")
}

fn set_num(wb: &mut Workbook, sheet: &str, cell: &str, n: f64) {
    wb.set(sheet, addr(cell), CellInput::Literal(Value::Number(n))).unwrap();
}

fn set_formula(wb: &mut Workbook, sheet: &str, cell: &str, formula: &str) {
    wb.set(sheet, addr(cell), CellInput::Formula(formula.to_string())).unwrap();
}

fn get_num(wb: &Workbook, sheet: &str, cell: &str) -> f64 {
    match wb.get(sheet, addr(cell)).unwrap().value() {
        Value::Number(n) => *n,
        v => panic!("expected number at {cell}, got {v:?}"),
    }
}

fn main() {
    // ── Build the workbook ───────────────────────────────────────────────────

    let mut wb = Workbook::new(EngineFlavor::Sheets);

    // Income sheet: monthly income sources
    wb.add_sheet(Worksheet::new("Income")).unwrap();
    set_num(&mut wb, "Income", "A1", 5000.0); // Salary
    set_num(&mut wb, "Income", "A2", 800.0);  // Freelance
    set_num(&mut wb, "Income", "A3", 200.0);  // Interest
    // A4: total income
    set_formula(&mut wb, "Income", "A4", "=SUM(A1:A3)");

    // Expenses sheet: monthly expense categories
    wb.add_sheet(Worksheet::new("Expenses")).unwrap();
    set_num(&mut wb, "Expenses", "A1", 1500.0); // Rent
    set_num(&mut wb, "Expenses", "A2", 400.0);  // Groceries
    set_num(&mut wb, "Expenses", "A3", 150.0);  // Utilities
    set_num(&mut wb, "Expenses", "A4", 300.0);  // Transport
    // A5: total expenses
    set_formula(&mut wb, "Expenses", "A5", "=SUM(A1:A4)");

    // Summary sheet: net = income - expenses
    wb.add_sheet(Worksheet::new("Summary")).unwrap();
    set_formula(&mut wb, "Summary", "A1", "=Income!A4");      // total income
    set_formula(&mut wb, "Summary", "A2", "=Expenses!A5");    // total expenses
    set_formula(&mut wb, "Summary", "A3", "=A1-A2");          // net

    // ── Recalculate ──────────────────────────────────────────────────────────

    // RecalcContext pins NOW/TODAY and the RNG; use a fixed instant for
    // deterministic output. Substitute the real UTC millisecond timestamp for a
    // live workbook.
    let ctx = RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0)
        .expect("valid IANA timezone");

    let changes = wb.recalc(&ctx);
    println!("Recalculated {} formula cell(s).\n", changes.len());

    // ── Print results ────────────────────────────────────────────────────────

    let total_income   = get_num(&wb, "Income",   "A4");
    let total_expenses = get_num(&wb, "Expenses", "A5");
    let net            = get_num(&wb, "Summary",  "A3");

    println!("Monthly Budget");
    println!("==============");
    println!("  Total income:    {:>10.2}", total_income);
    println!("  Total expenses:  {:>10.2}", total_expenses);
    println!("  Net:             {:>10.2}", net);

    if net >= 0.0 {
        println!("\nSurplus of {:.2}", net);
    } else {
        println!("\nDeficit of {:.2}", net.abs());
    }

    // ── Round-trip to JSON ───────────────────────────────────────────────────

    let json = wb.to_json().expect("serialization must succeed");
    println!("\nWorkbook JSON ({} bytes)", json.len());

    let wb2 = Workbook::from_json(json.as_bytes()).expect("round-trip must succeed");
    assert_eq!(wb, wb2, "round-trip produces identical workbook");
    println!("Round-trip check: OK");
}
