#![no_main]
use libfuzzer_sys::fuzz_target;
use truecalc_workbook::Workbook;

fuzz_target!(|data: &[u8]| {
    let _ = Workbook::from_json(data);
});
