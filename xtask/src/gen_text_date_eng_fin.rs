use crate::types::{Platform, TestCase};

fn tc(desc: &str, formula: &str, expected_type: &str, category: &str) -> TestCase {
    TestCase::new(desc, formula, "", category, expected_type)
}

pub fn generate_text(_platform: Platform) -> Vec<TestCase> {
    vec![
        // LEN
        tc("LEN basic string", r#"LEN("hello")"#, "number", "text"),
        tc("LEN empty string", r#"LEN("")"#, "number", "text"),
        tc("LEN with spaces", r#"LEN("  hi  ")"#, "number", "text"),
        tc("LEN number coercion", "LEN(123)", "number", "text"),
        // LEFT
        tc("LEFT basic", r#"LEFT("hello",3)"#, "string", "text"),
        tc("LEFT zero chars", r#"LEFT("hello",0)"#, "string", "text"),
        tc("LEFT entire string", r#"LEFT("hello",5)"#, "string", "text"),
        tc("LEFT beyond length", r#"LEFT("hello",99)"#, "string", "text"),
        tc("LEFT default", r#"LEFT("hello")"#, "string", "text"),
        // RIGHT
        tc("RIGHT basic", r#"RIGHT("hello",3)"#, "string", "text"),
        tc("RIGHT default", r#"RIGHT("hello")"#, "string", "text"),
        tc("RIGHT zero chars", r#"RIGHT("hello",0)"#, "string", "text"),
        // MID — BUG-18: start=0
        tc("MID start=0 (BUG-18)", r#"MID("hello",0,3)"#, "string", "text"),
        tc("MID start=1 normal", r#"MID("hello",1,3)"#, "string", "text"),
        tc("MID middle", r#"MID("hello",2,2)"#, "string", "text"),
        tc("MID past end", r#"MID("hello",4,10)"#, "string", "text"),
        // UPPER / LOWER / PROPER
        tc("UPPER basic", r#"UPPER("hello")"#, "string", "text"),
        tc("LOWER basic", r#"LOWER("HELLO")"#, "string", "text"),
        tc("PROPER basic", r#"PROPER("hello world")"#, "string", "text"),
        tc("PROPER mixed case", r#"PROPER("hElLo")"#, "string", "text"),
        // TRIM / CLEAN
        tc("TRIM leading trailing", r#"TRIM("  hello  ")"#, "string", "text"),
        tc("TRIM internal spaces", r#"TRIM("hello   world")"#, "string", "text"),
        tc("CLEAN no change", r#"CLEAN("hello")"#, "string", "text"),
        // SUBSTITUTE
        tc("SUBSTITUTE basic", r#"SUBSTITUTE("hello world","world","Rust")"#, "string", "text"),
        tc("SUBSTITUTE instance", r#"SUBSTITUTE("aabaa","a","x",2)"#, "string", "text"),
        tc("SUBSTITUTE no match", r#"SUBSTITUTE("hello","z","x")"#, "string", "text"),
        // REPLACE
        tc("REPLACE basic", r#"REPLACE("hello",2,3,"XX")"#, "string", "text"),
        tc("REPLACE at start", r#"REPLACE("hello",1,2,"XX")"#, "string", "text"),
        // FIND
        tc("FIND basic", r#"FIND("l","hello")"#, "number", "text"),
        tc("FIND with start", r#"FIND("l","hello",4)"#, "number", "text"),
        tc("FIND not found", r#"FIND("z","hello")"#, "error", "text"),
        tc("FIND case-sensitive", r#"FIND("H","hello")"#, "error", "text"),
        // SEARCH
        tc("SEARCH case-insensitive", r#"SEARCH("H","hello")"#, "number", "text"),
        tc("SEARCH wildcard", r#"SEARCH("h?llo","hello")"#, "number", "text"),
        tc("SEARCH not found", r#"SEARCH("z","hello")"#, "error", "text"),
        // TEXT — BUG-16: time formats
        tc("TEXT date format", r#"TEXT(45000,"YYYY-MM-DD")"#, "string", "text"),
        tc("TEXT currency format", r#"TEXT(1234.5,"$#,##0.00")"#, "string", "text"),
        tc("TEXT time hh:mm:ss (BUG-16)", r#"TEXT(0.5,"hh:mm:ss")"#, "string", "text"),
        tc("TEXT time h:mm AM/PM (BUG-16)", r#"TEXT(0.5,"h:mm AM/PM")"#, "string", "text"),
        tc("TEXT time hh:mm (BUG-16)", r#"TEXT(0.75,"hh:mm")"#, "string", "text"),
        tc("TEXT integer format", r#"TEXT(42,"0")"#, "string", "text"),
        tc("TEXT percent format", r#"TEXT(0.25,"0%")"#, "string", "text"),
        // VALUE — BUG-17: formatted strings
        tc("VALUE plain number", r#"VALUE("123")"#, "number", "text"),
        tc("VALUE with spaces (BUG-17)", r#"VALUE("  42  ")"#, "number", "text"),
        tc("VALUE comma-formatted (BUG-17)", r#"VALUE("1,234.56")"#, "number", "text"),
        tc("VALUE percent (BUG-17)", r#"VALUE("12%")"#, "number", "text"),
        tc("VALUE dollar (BUG-17)", r#"VALUE("$42")"#, "number", "text"),
        tc("VALUE invalid text", r#"VALUE("abc")"#, "error", "text"),
        // CONCATENATE / TEXTJOIN / JOIN
        tc("CONCATENATE two strings", r#"CONCATENATE("hello"," ","world")"#, "string", "text"),
        tc("CONCATENATE with number", r#"CONCATENATE("val=",42)"#, "string", "text"),
        tc("TEXTJOIN basic", r#"TEXTJOIN(",",TRUE,"a","b","c")"#, "string", "text"),
        tc("TEXTJOIN ignore empty", r#"TEXTJOIN(",",TRUE,"a","","c")"#, "string", "text"),
        tc("TEXTJOIN keep empty", r#"TEXTJOIN(",",FALSE,"a","","c")"#, "string", "text"),
        tc("JOIN basic", r#"JOIN("-","a","b","c")"#, "string", "text"),
        // SPLIT
        tc("SPLIT basic", r#"SPLIT("a,b,c",",")"#, "string", "text"),
        tc("SPLIT by each char", r#"SPLIT("a,b;c",",;")"#, "string", "text"),
        // REPT
        tc("REPT basic", r#"REPT("ab",3)"#, "string", "text"),
        tc("REPT zero times", r#"REPT("ab",0)"#, "string", "text"),
        // CHAR / CODE / UNICODE / UNICHAR
        tc("CHAR basic", "CHAR(65)", "string", "text"),
        tc("CHAR newline", "CHAR(10)", "string", "text"),
        tc("CODE basic", r#"CODE("A")"#, "number", "text"),
        tc("CODE first char", r#"CODE("ABC")"#, "number", "text"),
        tc("UNICODE basic", r#"UNICODE("A")"#, "number", "text"),
        tc("UNICHAR basic", "UNICHAR(65)", "string", "text"),
        tc("UNICHAR emoji range", "UNICHAR(128514)", "string", "text"),
        // EXACT
        tc("EXACT same", r#"EXACT("hello","hello")"#, "boolean", "text"),
        tc("EXACT case different", r#"EXACT("hello","Hello")"#, "boolean", "text"),
        tc("EXACT different", r#"EXACT("abc","xyz")"#, "boolean", "text"),
        // T
        tc("T with text", r#"T("hello")"#, "string", "text"),
        tc("T with number", "T(42)", "string", "text"),
        // FIXED
        tc("FIXED basic", "FIXED(1234.567,2)", "string", "text"),
        tc("FIXED no commas", "FIXED(1234.567,2,TRUE)", "string", "text"),
        tc("FIXED zero decimals", "FIXED(1234.567,0)", "string", "text"),
        // DOLLAR
        tc("DOLLAR basic", "DOLLAR(1234.5)", "string", "text"),
        tc("DOLLAR with decimals", "DOLLAR(1234.5,3)", "string", "text"),
        tc("DOLLAR negative", "DOLLAR(-99.5)", "string", "text"),
        // ARABIC / ROMAN
        tc("ARABIC basic", r#"ARABIC("XIV")"#, "number", "text"),
        tc("ARABIC zero", r#"ARABIC("")"#, "number", "text"),
        tc("ROMAN basic", "ROMAN(14)", "string", "text"),
        tc("ROMAN large", "ROMAN(1999)", "string", "text"),
        // ASC
        tc("ASC half-width passthrough", r#"ASC("abc")"#, "string", "text"),
        // REGEXMATCH / REGEXEXTRACT / REGEXREPLACE
        tc("REGEXMATCH basic true", r#"REGEXMATCH("hello123","[0-9]+")"#, "boolean", "text"),
        tc("REGEXMATCH no match", r#"REGEXMATCH("hello","[0-9]+")"#, "boolean", "text"),
        tc("REGEXEXTRACT basic", r#"REGEXEXTRACT("hello123","[0-9]+")"#, "string", "text"),
        tc("REGEXEXTRACT no match", r#"REGEXEXTRACT("hello","[0-9]+")"#, "error", "text"),
        tc("REGEXREPLACE basic", r#"REGEXREPLACE("hello123","[0-9]+","NUM")"#, "string", "text"),
        tc("REGEXREPLACE no match", r#"REGEXREPLACE("hello","[0-9]+","NUM")"#, "string", "text"),
        // LEFTB / RIGHTB / LENB / MIDB / FINDB / REPLACEB / SEARCHB
        tc("LEFTB ASCII", r#"LEFTB("hello",3)"#, "string", "text"),
        tc("RIGHTB ASCII", r#"RIGHTB("hello",3)"#, "string", "text"),
        tc("LENB ASCII", r#"LENB("hello")"#, "number", "text"),
        tc("MIDB ASCII", r#"MIDB("hello",2,3)"#, "string", "text"),
        tc("FINDB basic", r#"FINDB("l","hello")"#, "number", "text"),
        tc("REPLACEB basic", r#"REPLACEB("hello",2,3,"XX")"#, "string", "text"),
        tc("SEARCHB basic", r#"SEARCHB("L","hello")"#, "number", "text"),
        // LEN additional
        tc("LEN unicode", r#"LEN("cafe\u{301}")"#, "number", "text"),
        tc("LEN single char", r#"LEN("a")"#, "number", "text"),
        // LEFT additional
        tc("LEFT empty string", r#"LEFT("",3)"#, "string", "text"),
        tc("LEFT one char", r#"LEFT("hello",1)"#, "string", "text"),
        // RIGHT additional
        tc("RIGHT one char", r#"RIGHT("hello",1)"#, "string", "text"),
        tc("RIGHT empty string", r#"RIGHT("",3)"#, "string", "text"),
        // MID additional
        tc("MID start beyond end", r#"MID("hello",10,3)"#, "string", "text"),
        tc("MID zero length", r#"MID("hello",2,0)"#, "string", "text"),
        // UPPER / LOWER / PROPER additional
        tc("UPPER numbers unchanged", r#"UPPER("hello123")"#, "string", "text"),
        tc("LOWER numbers unchanged", r#"LOWER("HELLO123")"#, "string", "text"),
        tc("PROPER hyphenated", r#"PROPER("hello-world")"#, "string", "text"),
        // TRIM additional
        tc("TRIM only spaces", r#"TRIM("   ")"#, "string", "text"),
        tc("TRIM no change", r#"TRIM("hello")"#, "string", "text"),
        // SUBSTITUTE additional
        tc("SUBSTITUTE all instances", r#"SUBSTITUTE("banana","a","x")"#, "string", "text"),
        tc("SUBSTITUTE first instance", r#"SUBSTITUTE("banana","a","x",1)"#, "string", "text"),
        tc("SUBSTITUTE empty replacement", r#"SUBSTITUTE("hello","l","")"#, "string", "text"),
        // REPLACE additional
        tc("REPLACE zero length", r#"REPLACE("hello",3,0,"XX")"#, "string", "text"),
        tc("REPLACE end of string", r#"REPLACE("hello",5,1,"Z")"#, "string", "text"),
        // FIND additional
        tc("FIND start of string", r#"FIND("h","hello")"#, "number", "text"),
        tc("FIND empty needle", r#"FIND("","hello")"#, "number", "text"),
        // SEARCH additional
        tc("SEARCH star wildcard", r#"SEARCH("h*o","hello")"#, "number", "text"),
        tc("SEARCH with start pos", r#"SEARCH("l","hello",4)"#, "number", "text"),
        // TEXT additional
        tc("TEXT number format comma", "TEXT(1234567,\"#,##0\")", "string", "text"),
        tc("TEXT scientific", r#"TEXT(0.000012345,"0.00E+00")"#, "string", "text"),
        tc("TEXT zero", r#"TEXT(0,"0.00")"#, "string", "text"),
        tc("TEXT negative", r#"TEXT(-42,"0.00")"#, "string", "text"),
        // VALUE additional
        tc("VALUE negative", r#"VALUE("-42")"#, "number", "text"),
        tc("VALUE decimal", r#"VALUE("3.14")"#, "number", "text"),
        tc("VALUE scientific", r#"VALUE("1E3")"#, "number", "text"),
        // CONCATENATE additional
        tc("CONCATENATE single arg", r#"CONCATENATE("hello")"#, "string", "text"),
        tc("CONCATENATE empty strings", r#"CONCATENATE("","","")"#, "string", "text"),
        // TEXTJOIN additional
        tc("TEXTJOIN empty delimiter", r#"TEXTJOIN("",TRUE,"a","b","c")"#, "string", "text"),
        tc("TEXTJOIN numbers", r#"TEXTJOIN(",",TRUE,1,2,3)"#, "string", "text"),
        // JOIN additional
        tc("JOIN single value", r#"JOIN(",","a")"#, "string", "text"),
        tc("JOIN numbers", r#"JOIN("+",1,2,3)"#, "string", "text"),
        // SPLIT additional
        tc("SPLIT space delimiter", r#"SPLIT("hello world"," ")"#, "string", "text"),
        tc("SPLIT empty parts removed", r#"SPLIT("a,,b",",")"#, "string", "text"),
        // REPT additional
        tc("REPT single char", r#"REPT("x",5)"#, "string", "text"),
        tc("REPT empty string", r#"REPT("",10)"#, "string", "text"),
        // CHAR additional
        tc("CHAR space", "CHAR(32)", "string", "text"),
        tc("CHAR Z", "CHAR(90)", "string", "text"),
        tc("CHAR lowercase a", "CHAR(97)", "string", "text"),
        // CODE additional
        tc("CODE space", r#"CODE(" ")"#, "number", "text"),
        tc("CODE lowercase", r#"CODE("a")"#, "number", "text"),
        // EXACT additional
        tc("EXACT empty strings", r#"EXACT("","")"#, "boolean", "text"),
        tc("EXACT empty vs non-empty", r#"EXACT("","a")"#, "boolean", "text"),
        // T additional
        tc("T empty string", r#"T("")"#, "string", "text"),
        tc("T boolean", "T(TRUE)", "string", "text"),
        // FIXED additional
        tc("FIXED negative number", "FIXED(-1234.567,2)", "string", "text"),
        tc("FIXED large decimals", "FIXED(1.23456,5)", "string", "text"),
        // DOLLAR additional
        tc("DOLLAR zero decimals", "DOLLAR(1234.5,0)", "string", "text"),
        tc("DOLLAR large number", "DOLLAR(1234567.89)", "string", "text"),
        // ARABIC / ROMAN additional
        tc("ARABIC complex", r#"ARABIC("MMXIX")"#, "number", "text"),
        tc("ROMAN zero", "ROMAN(0)", "string", "text"),
        tc("ROMAN one", "ROMAN(1)", "string", "text"),
        // REGEXMATCH additional
        tc("REGEXMATCH anchored start", r#"REGEXMATCH("hello","^hel")"#, "boolean", "text"),
        tc("REGEXMATCH anchored end", r#"REGEXMATCH("hello","lo$")"#, "boolean", "text"),
        // REGEXEXTRACT additional
        tc("REGEXEXTRACT group", r#"REGEXEXTRACT("hello123world","([0-9]+)")"#, "string", "text"),
        // REGEXREPLACE additional
        tc("REGEXREPLACE all digits", r#"REGEXREPLACE("a1b2c3","[0-9]","X")"#, "string", "text"),
        // LENB additional
        tc("LENB empty string", r#"LENB("")"#, "number", "text"),
        // LEFTB additional
        tc("LEFTB zero bytes", r#"LEFTB("hello",0)"#, "string", "text"),
        // RIGHTB additional
        tc("RIGHTB zero bytes", r#"RIGHTB("hello",0)"#, "string", "text"),
        // MIDB additional
        tc("MIDB start 1", r#"MIDB("hello",1,5)"#, "string", "text"),
        // FINDB additional
        tc("FINDB not found", r#"FINDB("z","hello")"#, "error", "text"),
        // REPLACEB additional
        tc("REPLACEB at start", r#"REPLACEB("hello",1,2,"XX")"#, "string", "text"),
        // SEARCHB additional
        tc("SEARCHB not found", r#"SEARCHB("z","hello")"#, "error", "text"),
        // UNICODE / UNICHAR additional
        tc("UNICODE space", r#"UNICODE(" ")"#, "number", "text"),
        tc("UNICHAR null char", "UNICHAR(0)", "error", "text"),
        // LEN edge cases
        tc("LEN tab character", "LEN(\"\t\")", "number", "text"),
        tc("LEN 10 chars", r#"LEN("0123456789")"#, "number", "text"),
        // LEFT edge cases
        tc("LEFT unicode", r#"LEFT("hello",2)"#, "string", "text"),
        tc("LEFT negative clamp", r#"LEFT("hello",0)"#, "string", "text"),
        // RIGHT edge cases
        tc("RIGHT full string", r#"RIGHT("abc",3)"#, "string", "text"),
        // MID edge cases
        tc("MID entire string", r#"MID("hello",1,5)"#, "string", "text"),
        tc("MID single char", r#"MID("hello",3,1)"#, "string", "text"),
        tc("MID start equals length", r#"MID("hello",5,1)"#, "string", "text"),
        // SUBSTITUTE edge cases
        tc("SUBSTITUTE replace with longer", r#"SUBSTITUTE("ab","b","bc")"#, "string", "text"),
        tc("SUBSTITUTE instance 3", r#"SUBSTITUTE("aaaaaa","aa","X",3)"#, "string", "text"),
        // FIND edge cases
        tc("FIND substring", r#"FIND("ell","hello")"#, "number", "text"),
        tc("FIND from position 2", r#"FIND("e","hello",2)"#, "error", "text"),
        // SEARCH edge cases
        tc("SEARCH tilde escape", r#"SEARCH("~?","hello?")"#, "number", "text"),
        // TEXT edge cases
        tc("TEXT large number", r#"TEXT(1000000,"0")"#, "string", "text"),
        tc("TEXT fraction", r#"TEXT(0.1,"0.000")"#, "string", "text"),
        tc("TEXT negative currency", r#"TEXT(-1234.5,"$#,##0.00")"#, "string", "text"),
        // VALUE edge cases
        tc("VALUE zero", r#"VALUE("0")"#, "number", "text"),
        tc("VALUE leading zeros", r#"VALUE("007")"#, "number", "text"),
        // CONCATENATE edge cases
        tc("CONCATENATE four args", r#"CONCATENATE("a","b","c","d")"#, "string", "text"),
        // TEXTJOIN edge cases
        tc("TEXTJOIN with number values", r#"TEXTJOIN("|",TRUE,1,2,3)"#, "string", "text"),
        tc("TEXTJOIN single value", r#"TEXTJOIN(",",TRUE,"only")"#, "string", "text"),
        // REPT edge cases
        tc("REPT long repetition", r#"REPT("x",10)"#, "string", "text"),
        // CHAR edge cases
        tc("CHAR out of range low", "CHAR(0)", "error", "text"),
        tc("CHAR out of range high", "CHAR(256)", "string", "text"),
        // EXACT edge cases
        tc("EXACT numbers as text", r#"EXACT("42","42")"#, "boolean", "text"),
        tc("EXACT trailing space", r#"EXACT("hello","hello ")"#, "boolean", "text"),
        // FIXED edge cases
        tc("FIXED default decimals", "FIXED(1234.5)", "string", "text"),
        tc("FIXED negative decimals", "FIXED(1234.567,-2)", "string", "text"),
        // DOLLAR edge cases
        tc("DOLLAR negative decimals", "DOLLAR(1234.567,-2)", "string", "text"),
        // ARABIC edge cases
        tc("ARABIC single I", r#"ARABIC("I")"#, "number", "text"),
        tc("ARABIC lowercase", r#"ARABIC("xiv")"#, "number", "text"),
        // ROMAN edge cases
        tc("ROMAN 4", "ROMAN(4)", "string", "text"),
        tc("ROMAN 9", "ROMAN(9)", "string", "text"),
        tc("ROMAN 40", "ROMAN(40)", "string", "text"),
        tc("ROMAN 400", "ROMAN(400)", "string", "text"),
        // REGEXMATCH edge cases
        tc("REGEXMATCH empty string", r#"REGEXMATCH("",".*")"#, "boolean", "text"),
        tc("REGEXMATCH dot matches any", r#"REGEXMATCH("a",".+")"#, "boolean", "text"),
        // REGEXEXTRACT edge cases
        tc("REGEXEXTRACT first word", r#"REGEXEXTRACT("hello world","\\w+")"#, "string", "text"),
        // REGEXREPLACE edge cases
        tc("REGEXREPLACE empty match insert", r#"REGEXREPLACE("abc","b","[b]")"#, "string", "text"),
        // JOIN edge cases
        tc("JOIN empty delimiter", r#"JOIN("","a","b","c")"#, "string", "text"),
        // SPLIT edge cases
        tc("SPLIT no match", r#"SPLIT("hello",",")"#, "string", "text"),
        // T edge cases
        tc("T with error propagation", "T(1/0)", "error", "text"),
    ]
}

pub fn generate_date(_platform: Platform) -> Vec<TestCase> {
    vec![
        // DATE
        tc("DATE basic", "DATE(2024,1,15)", "number", "date"),
        tc("DATE month overflow", "DATE(2024,13,1)", "number", "date"),
        tc("DATE day overflow", "DATE(2024,1,32)", "number", "date"),
        tc("DATE negative day", "DATE(2024,2,0)", "number", "date"),
        tc("DATE year 1900", "DATE(1900,1,1)", "number", "date"),
        tc("DATE year 2000", "DATE(2000,6,15)", "number", "date"),
        // DATEVALUE
        tc("DATEVALUE ISO format", r#"DATEVALUE("2024-01-15")"#, "number", "date"),
        tc("DATEVALUE US format", r#"DATEVALUE("1/15/2024")"#, "number", "date"),
        tc("DATEVALUE invalid", r#"DATEVALUE("not a date")"#, "error", "date"),
        // TIMEVALUE
        tc("TIMEVALUE basic", r#"TIMEVALUE("12:00:00")"#, "number", "date"),
        tc("TIMEVALUE PM", r#"TIMEVALUE("6:30 PM")"#, "number", "date"),
        tc("TIMEVALUE midnight", r#"TIMEVALUE("00:00:00")"#, "number", "date"),
        // YEAR / MONTH / DAY
        tc("YEAR basic", "YEAR(45000)", "number", "date"),
        tc("YEAR from DATE", "YEAR(DATE(2024,6,15))", "number", "date"),
        tc("MONTH basic", "MONTH(45000)", "number", "date"),
        tc("MONTH from DATE", "MONTH(DATE(2024,6,15))", "number", "date"),
        tc("DAY basic", "DAY(45000)", "number", "date"),
        tc("DAY from DATE", "DAY(DATE(2024,6,15))", "number", "date"),
        // HOUR / MINUTE / SECOND
        tc("HOUR noon", "HOUR(0.5)", "number", "date"),
        tc("HOUR midnight", "HOUR(0)", "number", "date"),
        tc("MINUTE basic", "MINUTE(0.5208333)", "number", "date"),
        tc("SECOND basic", "SECOND(0.5)", "number", "date"),
        // WEEKDAY
        tc("WEEKDAY default type", "WEEKDAY(DATE(2024,1,15))", "number", "date"),
        tc("WEEKDAY type 2", "WEEKDAY(DATE(2024,1,15),2)", "number", "date"),
        tc("WEEKDAY type 3", "WEEKDAY(DATE(2024,1,15),3)", "number", "date"),
        // WEEKNUM
        tc("WEEKNUM basic", "WEEKNUM(DATE(2024,1,15))", "number", "date"),
        tc("WEEKNUM type 2", "WEEKNUM(DATE(2024,1,15),2)", "number", "date"),
        // ISOWEEKNUM
        tc("ISOWEEKNUM basic", "ISOWEEKNUM(DATE(2024,1,15))", "number", "date"),
        tc("ISOWEEKNUM year boundary", "ISOWEEKNUM(DATE(2024,12,31))", "number", "date"),
        // DAYS
        tc("DAYS basic", "DAYS(DATE(2024,12,31),DATE(2024,1,1))", "number", "date"),
        tc("DAYS negative", "DAYS(DATE(2024,1,1),DATE(2024,12,31))", "number", "date"),
        // DAYS360
        tc("DAYS360 US method", "DAYS360(DATE(2024,1,1),DATE(2024,12,31))", "number", "date"),
        tc("DAYS360 European method", "DAYS360(DATE(2024,1,1),DATE(2024,12,31),TRUE)", "number", "date"),
        // DATEDIF
        tc("DATEDIF years", r#"DATEDIF(DATE(2020,1,1),DATE(2024,6,15),"Y")"#, "number", "date"),
        tc("DATEDIF months", r#"DATEDIF(DATE(2020,1,1),DATE(2024,6,15),"M")"#, "number", "date"),
        tc("DATEDIF days", r#"DATEDIF(DATE(2020,1,1),DATE(2024,6,15),"D")"#, "number", "date"),
        tc("DATEDIF YM", r#"DATEDIF(DATE(2020,1,1),DATE(2024,6,15),"YM")"#, "number", "date"),
        tc("DATEDIF MD", r#"DATEDIF(DATE(2020,1,1),DATE(2024,6,15),"MD")"#, "number", "date"),
        tc("DATEDIF YD", r#"DATEDIF(DATE(2020,1,1),DATE(2024,6,15),"YD")"#, "number", "date"),
        // EDATE
        tc("EDATE forward", "EDATE(DATE(2024,1,31),1)", "number", "date"),
        tc("EDATE backward", "EDATE(DATE(2024,3,31),-1)", "number", "date"),
        tc("EDATE 12 months", "EDATE(DATE(2024,1,1),12)", "number", "date"),
        // EOMONTH
        tc("EOMONTH current", "EOMONTH(DATE(2024,1,15),0)", "number", "date"),
        tc("EOMONTH next", "EOMONTH(DATE(2024,1,15),1)", "number", "date"),
        tc("EOMONTH prev", "EOMONTH(DATE(2024,1,15),-1)", "number", "date"),
        // NETWORKDAYS
        tc("NETWORKDAYS basic", "NETWORKDAYS(DATE(2024,1,1),DATE(2024,1,31))", "number", "date"),
        tc("NETWORKDAYS same day", "NETWORKDAYS(DATE(2024,1,15),DATE(2024,1,15))", "number", "date"),
        // NETWORKDAYS.INTL
        tc("NETWORKDAYS.INTL default", "NETWORKDAYS.INTL(DATE(2024,1,1),DATE(2024,1,31),1)", "number", "date"),
        tc("NETWORKDAYS.INTL Fri-Sat weekend", "NETWORKDAYS.INTL(DATE(2024,1,1),DATE(2024,1,31),7)", "number", "date"),
        // WORKDAY
        tc("WORKDAY forward", "WORKDAY(DATE(2024,1,1),10)", "number", "date"),
        tc("WORKDAY backward", "WORKDAY(DATE(2024,1,31),-10)", "number", "date"),
        // WORKDAY.INTL
        tc("WORKDAY.INTL default weekend", "WORKDAY.INTL(DATE(2024,1,1),10,1)", "number", "date"),
        tc("WORKDAY.INTL Fri-Sat weekend", "WORKDAY.INTL(DATE(2024,1,1),10,7)", "number", "date"),
        // YEARFRAC
        tc("YEARFRAC basis 0", "YEARFRAC(DATE(2024,1,1),DATE(2024,12,31),0)", "number", "date"),
        tc("YEARFRAC basis 1", "YEARFRAC(DATE(2024,1,1),DATE(2024,12,31),1)", "number", "date"),
        tc("YEARFRAC basis 3", "YEARFRAC(DATE(2024,1,1),DATE(2024,12,31),3)", "number", "date"),
        // TIME
        tc("TIME basic", "TIME(12,0,0)", "number", "date"),
        tc("TIME midnight", "TIME(0,0,0)", "number", "date"),
        tc("TIME noon", "TIME(12,30,0)", "number", "date"),
        // EPOCHTODATE
        tc("EPOCHTODATE seconds", "EPOCHTODATE(0)", "number", "date"),
        tc("EPOCHTODATE milliseconds", "EPOCHTODATE(1000,1)", "number", "date"),
    ]
}

pub fn generate_engineering(_platform: Platform) -> Vec<TestCase> {
    vec![
        // ERF / ERFC
        tc("ERF single arg", "ERF(1)", "number", "engineering"),
        tc("ERF two args", "ERF(0,1)", "number", "engineering"),
        tc("ERF.PRECISE basic", "ERF.PRECISE(1)", "number", "engineering"),
        tc("ERFC basic", "ERFC(1)", "number", "engineering"),
        tc("ERFC.PRECISE basic", "ERFC.PRECISE(1)", "number", "engineering"),
        tc("ERF zero", "ERF(0)", "number", "engineering"),
        tc("ERFC zero", "ERFC(0)", "number", "engineering"),
        // BIN2DEC / BIN2HEX / BIN2OCT
        tc("BIN2DEC zero", r#"BIN2DEC("0")"#, "number", "engineering"),
        tc("BIN2DEC positive", r#"BIN2DEC("1010")"#, "number", "engineering"),
        tc("BIN2DEC negative MSB set", r#"BIN2DEC("1111111110")"#, "number", "engineering"),
        tc("BIN2HEX basic", r#"BIN2HEX("1010")"#, "string", "engineering"),
        tc("BIN2HEX with places", r#"BIN2HEX("1010",4)"#, "string", "engineering"),
        tc("BIN2OCT basic", r#"BIN2OCT("1010")"#, "string", "engineering"),
        // DEC2BIN / DEC2HEX / DEC2OCT
        tc("DEC2BIN zero", "DEC2BIN(0)", "string", "engineering"),
        tc("DEC2BIN positive", "DEC2BIN(10)", "string", "engineering"),
        tc("DEC2BIN negative", "DEC2BIN(-2)", "string", "engineering"),
        tc("DEC2BIN with places", "DEC2BIN(10,8)", "string", "engineering"),
        tc("DEC2HEX positive", "DEC2HEX(255)", "string", "engineering"),
        tc("DEC2HEX negative", "DEC2HEX(-1)", "string", "engineering"),
        tc("DEC2OCT positive", "DEC2OCT(8)", "string", "engineering"),
        // HEX2BIN / HEX2DEC / HEX2OCT
        tc("HEX2BIN basic", r#"HEX2BIN("A")"#, "string", "engineering"),
        tc("HEX2BIN with places", r#"HEX2BIN("A",8)"#, "string", "engineering"),
        tc("HEX2DEC positive", r#"HEX2DEC("FF")"#, "number", "engineering"),
        tc("HEX2DEC negative", r#"HEX2DEC("FFFFFFFFFF")"#, "number", "engineering"),
        tc("HEX2OCT basic", r#"HEX2OCT("F")"#, "string", "engineering"),
        // OCT2BIN / OCT2DEC / OCT2HEX
        tc("OCT2BIN basic", r#"OCT2BIN("7")"#, "string", "engineering"),
        tc("OCT2DEC basic", r#"OCT2DEC("17")"#, "number", "engineering"),
        tc("OCT2HEX basic", r#"OCT2HEX("17")"#, "string", "engineering"),
        // COMPLEX / IM functions
        tc("COMPLEX real only", "COMPLEX(3,0)", "string", "engineering"),
        tc("COMPLEX imaginary only", "COMPLEX(0,2)", "string", "engineering"),
        tc("COMPLEX both parts", "COMPLEX(3,4)", "string", "engineering"),
        tc("COMPLEX with j suffix", r#"COMPLEX(3,4,"j")"#, "string", "engineering"),
        tc("IMABS basic", r#"IMABS("3+4i")"#, "number", "engineering"),
        tc("IMABS pure real", r#"IMABS("5")"#, "number", "engineering"),
        tc("IMARGUMENT basic", r#"IMARGUMENT("3+4i")"#, "number", "engineering"),
        tc("IMCONJUGATE basic", r#"IMCONJUGATE("3+4i")"#, "string", "engineering"),
        tc("IMCONJUGATE pure real", r#"IMCONJUGATE("5")"#, "string", "engineering"),
        tc("IMREAL basic", r#"IMREAL("3+4i")"#, "number", "engineering"),
        tc("IMAGINARY basic", r#"IMAGINARY("3+4i")"#, "number", "engineering"),
        tc("IMSUM two", r#"IMSUM("1+2i","3+4i")"#, "string", "engineering"),
        tc("IMSUB basic", r#"IMSUB("3+4i","1+2i")"#, "string", "engineering"),
        tc("IMPRODUCT basic", r#"IMPRODUCT("3+4i","1+2i")"#, "string", "engineering"),
        tc("IMDIV basic", r#"IMDIV("3+4i","1+2i")"#, "string", "engineering"),
        tc("IMSQRT basic", r#"IMSQRT("-1")"#, "string", "engineering"),
        tc("IMPOWER basic", r#"IMPOWER("3+4i",2)"#, "string", "engineering"),
        tc("IMEXP basic", r#"IMEXP("1")"#, "string", "engineering"),
        tc("IMLN basic", r#"IMLN("1")"#, "string", "engineering"),
        tc("IMLOG10 basic", r#"IMLOG10("10")"#, "string", "engineering"),
        tc("IMLOG2 basic", r#"IMLOG2("2")"#, "string", "engineering"),
        tc("IMSIN basic", r#"IMSIN("1+i")"#, "string", "engineering"),
        tc("IMCOS basic", r#"IMCOS("1+i")"#, "string", "engineering"),
        tc("IMTAN basic", r#"IMTAN("1+i")"#, "string", "engineering"),
        tc("IMCOT basic", r#"IMCOT("1+i")"#, "string", "engineering"),
        tc("IMCSC basic", r#"IMCSC("1+i")"#, "string", "engineering"),
        tc("IMSEC basic", r#"IMSEC("1+i")"#, "string", "engineering"),
        tc("IMSINH basic", r#"IMSINH("1+i")"#, "string", "engineering"),
        tc("IMCOSH basic", r#"IMCOSH("1+i")"#, "string", "engineering"),
        tc("IMCSCH basic", r#"IMCSCH("1+i")"#, "string", "engineering"),
        tc("IMSECH basic", r#"IMSECH("1+i")"#, "string", "engineering"),
        // BITAND / BITOR / BITXOR / BITLSHIFT / BITRSHIFT
        tc("BITAND basic", "BITAND(12,10)", "number", "engineering"),
        tc("BITAND zero", "BITAND(0,255)", "number", "engineering"),
        tc("BITOR basic", "BITOR(12,10)", "number", "engineering"),
        tc("BITXOR basic", "BITXOR(12,10)", "number", "engineering"),
        tc("BITXOR same", "BITXOR(5,5)", "number", "engineering"),
        tc("BITLSHIFT basic", "BITLSHIFT(1,4)", "number", "engineering"),
        tc("BITLSHIFT by 0", "BITLSHIFT(7,0)", "number", "engineering"),
        tc("BITRSHIFT basic", "BITRSHIFT(16,2)", "number", "engineering"),
        tc("BITRSHIFT by 0", "BITRSHIFT(7,0)", "number", "engineering"),
        // DELTA / GESTEP
        tc("DELTA equal", "DELTA(5,5)", "number", "engineering"),
        tc("DELTA not equal", "DELTA(5,4)", "number", "engineering"),
        tc("DELTA default second arg", "DELTA(0)", "number", "engineering"),
        tc("GESTEP above step", "GESTEP(5,3)", "number", "engineering"),
        tc("GESTEP below step", "GESTEP(2,3)", "number", "engineering"),
        tc("GESTEP equal to step", "GESTEP(3,3)", "number", "engineering"),
        tc("GESTEP default step", "GESTEP(5)", "number", "engineering"),
    ]
}

pub fn generate_financial(_platform: Platform) -> Vec<TestCase> {
    vec![
        // PV
        tc("PV basic loan", "PV(0.05/12,60,-200)", "number", "financial"),
        tc("PV with fv", "PV(0.05/12,60,-200,1000)", "number", "financial"),
        tc("PV annuity due", "PV(0.05/12,60,-200,0,1)", "number", "financial"),
        tc("PV zero rate", "PV(0,60,-200)", "number", "financial"),
        // FV
        tc("FV basic savings", "FV(0.05/12,60,-200)", "number", "financial"),
        tc("FV with pv", "FV(0.05/12,60,-200,-1000)", "number", "financial"),
        tc("FV annuity due", "FV(0.05/12,60,-200,0,1)", "number", "financial"),
        tc("FV zero rate", "FV(0,12,-100)", "number", "financial"),
        // PMT
        tc("PMT basic loan", "PMT(0.05/12,60,10000)", "number", "financial"),
        tc("PMT with fv", "PMT(0.05/12,60,10000,1000)", "number", "financial"),
        tc("PMT annuity due", "PMT(0.05/12,60,10000,0,1)", "number", "financial"),
        tc("PMT zero rate", "PMT(0,12,1200)", "number", "financial"),
        // NPER
        tc("NPER basic", "NPER(0.05/12,-200,10000)", "number", "financial"),
        tc("NPER with fv", "NPER(0.05/12,-200,10000,1000)", "number", "financial"),
        tc("NPER zero rate", "NPER(0,-200,10000)", "number", "financial"),
        // RATE
        tc("RATE basic", "RATE(60,-200,10000)", "number", "financial"),
        tc("RATE with fv", "RATE(60,-200,10000,1000)", "number", "financial"),
        // NPV
        tc("NPV basic", "NPV(0.1,-1000,300,400,500)", "number", "financial"),
        tc("NPV single cash flow", "NPV(0.1,1000)", "number", "financial"),
        // IRR
        tc("IRR basic", "IRR({-1000,300,400,500})", "number", "financial"),
        tc("IRR with guess", "IRR({-1000,300,400,500},0.1)", "number", "financial"),
        // XNPV
        tc("XNPV basic", "XNPV(0.1,{-1000,300,700},{DATE(2024,1,1),DATE(2024,6,1),DATE(2024,12,1)})", "number", "financial"),
        // XIRR
        tc("XIRR basic", "XIRR({-1000,300,700},{DATE(2024,1,1),DATE(2024,6,1),DATE(2024,12,1)})", "number", "financial"),
        // MIRR
        tc("MIRR basic", "MIRR({-1000,300,400,500},0.1,0.12)", "number", "financial"),
        // SLN
        tc("SLN basic", "SLN(10000,1000,5)", "number", "financial"),
        tc("SLN zero salvage", "SLN(10000,0,10)", "number", "financial"),
        // SYD
        tc("SYD year 1", "SYD(10000,1000,5,1)", "number", "financial"),
        tc("SYD last year", "SYD(10000,1000,5,5)", "number", "financial"),
        // DB
        tc("DB year 1", "DB(1000000,100000,6,1,7)", "number", "financial"),
        tc("DB mid year", "DB(1000000,100000,6,3)", "number", "financial"),
        // DDB
        tc("DDB year 1", "DDB(10000,1000,5,1)", "number", "financial"),
        tc("DDB with factor", "DDB(10000,1000,5,1,1.5)", "number", "financial"),
        // VDB
        tc("VDB basic", "VDB(10000,1000,5,0,1)", "number", "financial"),
        tc("VDB partial period", "VDB(10000,1000,5,0.5,1)", "number", "financial"),
        // IPMT / PPMT
        tc("IPMT period 1", "IPMT(0.05/12,1,60,10000)", "number", "financial"),
        tc("IPMT last period", "IPMT(0.05/12,60,60,10000)", "number", "financial"),
        tc("PPMT period 1", "PPMT(0.05/12,1,60,10000)", "number", "financial"),
        // ISPMT
        tc("ISPMT basic", "ISPMT(0.05/12,1,60,10000)", "number", "financial"),
        // CUMIPMT / CUMPRINC
        tc("CUMIPMT year 1", "CUMIPMT(0.05/12,60,10000,1,12,0)", "number", "financial"),
        tc("CUMPRINC year 1", "CUMPRINC(0.05/12,60,10000,1,12,0)", "number", "financial"),
        // EFFECT / NOMINAL
        tc("EFFECT basic", "EFFECT(0.1,4)", "number", "financial"),
        tc("EFFECT monthly", "EFFECT(0.12,12)", "number", "financial"),
        tc("NOMINAL basic", "NOMINAL(0.1,4)", "number", "financial"),
        // DOLLARDE / DOLLARFR
        tc("DOLLARDE basic", "DOLLARDE(1.02,16)", "number", "financial"),
        tc("DOLLARFR basic", "DOLLARFR(1.125,16)", "number", "financial"),
        // AMORLINC
        tc("AMORLINC basic", "AMORLINC(2400,DATE(2008,8,19),DATE(2008,12,31),300,1,0.15,1)", "number", "financial"),
        // PDURATION
        tc("PDURATION basic", "PDURATION(0.05,1000,2000)", "number", "financial"),
        // RRI
        tc("RRI basic", "RRI(10,1000,2000)", "number", "financial"),
        // DURATION / MDURATION
        tc("DURATION basic", "DURATION(DATE(2018,1,15),DATE(2023,1,15),0.05,0.06,2,0)", "number", "financial"),
        tc("MDURATION basic", "MDURATION(DATE(2018,1,15),DATE(2023,1,15),0.05,0.06,2,0)", "number", "financial"),
        // FVSCHEDULE
        tc("FVSCHEDULE basic", "FVSCHEDULE(1000,{0.05,0.06,0.07})", "number", "financial"),
        // MIRR corner case
        tc("MIRR all negative impossible", "MIRR({-100,-50,-30},0.1,0.12)", "error", "financial"),
        // Bond functions
        tc("ACCRINT basic", "ACCRINT(DATE(2023,1,1),DATE(2023,7,1),DATE(2023,10,1),0.05,1000,2,0)", "number", "financial"),
        tc("ACCRINTM basic", "ACCRINTM(DATE(2023,1,1),DATE(2023,10,1),0.05,1000,0)", "number", "financial"),
        tc("DISC basic", "DISC(DATE(2024,1,1),DATE(2024,12,31),97,100,0)", "number", "financial"),
        tc("INTRATE basic", "INTRATE(DATE(2024,1,1),DATE(2024,12,31),1000,1100,0)", "number", "financial"),
        tc("RECEIVED basic", "RECEIVED(DATE(2024,1,1),DATE(2024,12,31),1000,0.05,0)", "number", "financial"),
        tc("PRICE basic", "PRICE(DATE(2024,1,1),DATE(2029,1,1),0.05,0.06,100,2,0)", "number", "financial"),
        tc("PRICEDISC basic", "PRICEDISC(DATE(2024,1,1),DATE(2024,12,31),0.05,100,0)", "number", "financial"),
        tc("PRICEMAT basic", "PRICEMAT(DATE(2024,1,1),DATE(2025,1,1),DATE(2023,1,1),0.05,0.06,0)", "number", "financial"),
        tc("YIELD basic", "YIELD(DATE(2024,1,1),DATE(2029,1,1),0.05,95,100,2,0)", "number", "financial"),
        tc("YIELDDISC basic", "YIELDDISC(DATE(2024,1,1),DATE(2024,12,31),95,100,0)", "number", "financial"),
        tc("YIELDMAT basic", "YIELDMAT(DATE(2024,1,1),DATE(2025,1,1),DATE(2023,1,1),0.05,95,0)", "number", "financial"),
        tc("TBILLEQ basic", "TBILLEQ(DATE(2024,1,1),DATE(2024,7,1),0.05)", "number", "financial"),
        tc("TBILLPRICE basic", "TBILLPRICE(DATE(2024,1,1),DATE(2024,7,1),0.05)", "number", "financial"),
        tc("TBILLYIELD basic", "TBILLYIELD(DATE(2024,1,1),DATE(2024,7,1),98)", "number", "financial"),
        tc("COUPDAYBS basic", "COUPDAYBS(DATE(2024,1,15),DATE(2026,1,15),2,0)", "number", "financial"),
        tc("COUPDAYS basic", "COUPDAYS(DATE(2024,1,15),DATE(2026,1,15),2,0)", "number", "financial"),
        tc("COUPDAYSNC basic", "COUPDAYSNC(DATE(2024,1,15),DATE(2026,1,15),2,0)", "number", "financial"),
        tc("COUPNCD basic", "COUPNCD(DATE(2024,1,15),DATE(2026,1,15),2,0)", "number", "financial"),
        tc("COUPNUM basic", "COUPNUM(DATE(2024,1,15),DATE(2026,1,15),2,0)", "number", "financial"),
        tc("COUPPCD basic", "COUPPCD(DATE(2024,1,15),DATE(2026,1,15),2,0)", "number", "financial"),
    ]
}
