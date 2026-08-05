#![no_main]
use libfuzzer_sys::fuzz_target;
use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Workbook, Worksheet};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut wb = Workbook::new(EngineFlavor::Sheets);
        let _ = wb.add_sheet(Worksheet::new("S".to_string()));
        if let Some(addr) = Address::from_a1("A1") {
            let _ = wb.set("S", addr, CellInput::Formula(s.to_string()));
            if let Some(ctx) = RecalcContext::new(0, "UTC", 0) {
                wb.recalc(&ctx);
            }
        }
    }
});
