use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

#[derive(PartialEq)]
enum UnitCategory {
    Length,
    Mass,
    Time,
    Temperature,
    Pressure,
    Energy,
    Power,
    Force,
    Speed,
    Area,
    Volume,
    Magnetism,
    Information,
}

fn get_unit(name: &str) -> Option<(UnitCategory, f64)> {
    match name {
        // Length (base: m)
        "m"          => Some((UnitCategory::Length, 1.0)),
        "km"         => Some((UnitCategory::Length, 1000.0)),
        "mi"         => Some((UnitCategory::Length, 1609.344)),
        "nmi" | "Nmi" => Some((UnitCategory::Length, 1852.0)),
        "ft"         => Some((UnitCategory::Length, 0.3048)),
        "in"         => Some((UnitCategory::Length, 0.0254)),
        "yd"         => Some((UnitCategory::Length, 0.9144)),
        "cm"         => Some((UnitCategory::Length, 0.01)),
        "mm"         => Some((UnitCategory::Length, 0.001)),
        "Å" | "Ang"  => Some((UnitCategory::Length, 1e-10)),
        "ly"         => Some((UnitCategory::Length, 9.4607304725808e15)),
        "pica"       => Some((UnitCategory::Length, 0.00423333333)),
        "pt"         => Some((UnitCategory::Length, 0.000352778)),
        "ell"        => Some((UnitCategory::Length, 1.143)),
        "parsec"     => Some((UnitCategory::Length, 3.08568e16)),
        "survey_mi"  => Some((UnitCategory::Length, 1609.3472)),

        // Mass (base: kg)
        "kg"     => Some((UnitCategory::Mass, 1.0)),
        "g"      => Some((UnitCategory::Mass, 0.001)),
        "lbm"    => Some((UnitCategory::Mass, 0.45359237)),
        "oz"     => Some((UnitCategory::Mass, 0.028349523125)),
        "mg"     => Some((UnitCategory::Mass, 1e-6)),
        "grain"  => Some((UnitCategory::Mass, 6.479891e-5)),
        "cwt"    => Some((UnitCategory::Mass, 45.359237)),
        "uk_cwt" => Some((UnitCategory::Mass, 50.80234544)),
        "ton"    => Some((UnitCategory::Mass, 907.18474)),
        "uk_ton" => Some((UnitCategory::Mass, 1016.0469088)),
        "stone"  => Some((UnitCategory::Mass, 6.35029318)),
        "slug"   => Some((UnitCategory::Mass, 14.59390294)),

        // Time (base: s)
        "s" | "sec" => Some((UnitCategory::Time, 1.0)),
        "mn" | "min" => Some((UnitCategory::Time, 60.0)),
        "hr"        => Some((UnitCategory::Time, 3600.0)),
        "day"       => Some((UnitCategory::Time, 86400.0)),
        "yr"        => Some((UnitCategory::Time, 31557600.0)),

        // Temperature (special handling, factor unused)
        "C" | "cel"  => Some((UnitCategory::Temperature, 0.0)),
        "F" | "fah"  => Some((UnitCategory::Temperature, 0.0)),
        "K" | "kel"  => Some((UnitCategory::Temperature, 0.0)),
        "Rank"       => Some((UnitCategory::Temperature, 0.0)),

        // Pressure (base: Pa)
        "Pa"   => Some((UnitCategory::Pressure, 1.0)),
        "atm"  => Some((UnitCategory::Pressure, 101325.0)),
        "mmHg" => Some((UnitCategory::Pressure, 133.322387415)),
        "psi"  => Some((UnitCategory::Pressure, 6894.757293168)),
        "Torr" => Some((UnitCategory::Pressure, 133.322387415)),

        // Energy (base: J)
        "J"     => Some((UnitCategory::Energy, 1.0)),
        "kJ"    => Some((UnitCategory::Energy, 1000.0)),
        "cal"   => Some((UnitCategory::Energy, 4.184)),
        "kcal"  => Some((UnitCategory::Energy, 4184.0)),
        "eV"    => Some((UnitCategory::Energy, 1.602176634e-19)),
        "BTU"   => Some((UnitCategory::Energy, 1055.05585262)),
        "Wh"    => Some((UnitCategory::Energy, 3600.0)),
        "kWh"   => Some((UnitCategory::Energy, 3600000.0)),
        "HPh"   => Some((UnitCategory::Energy, 2684519.5368856)),
        "ft-lb" => Some((UnitCategory::Energy, 1.3558179483314)),

        // Power (base: W)
        "W"  => Some((UnitCategory::Power, 1.0)),
        "kW" => Some((UnitCategory::Power, 1000.0)),
        "HP" => Some((UnitCategory::Power, 745.69987158227)),
        "PS" => Some((UnitCategory::Power, 735.49875)),

        // Force (base: N)
        "N"    => Some((UnitCategory::Force, 1.0)),
        "lbf"  => Some((UnitCategory::Force, 4.4482216152605)),
        "dyn"  => Some((UnitCategory::Force, 1e-5)),
        "pond" => Some((UnitCategory::Force, 0.00980665)),

        // Speed (base: m/s)
        "m/s"   => Some((UnitCategory::Speed, 1.0)),
        "m/h"   => Some((UnitCategory::Speed, 0.00027778)),
        "kn"    => Some((UnitCategory::Speed, 0.51444)),
        "mph"   => Some((UnitCategory::Speed, 0.44704)),
        "admkn" => Some((UnitCategory::Speed, 0.514773)),

        // Area (base: m²)
        "m2"   => Some((UnitCategory::Area, 1.0)),
        "km2"  => Some((UnitCategory::Area, 1e6)),
        "ft2"  => Some((UnitCategory::Area, 0.09290304)),
        "in2"  => Some((UnitCategory::Area, 0.00064516)),
        "yd2"  => Some((UnitCategory::Area, 0.83612736)),
        "mi2"  => Some((UnitCategory::Area, 2589988.110336)),
        "ha"   => Some((UnitCategory::Area, 10000.0)),
        "ar"   => Some((UnitCategory::Area, 100.0)),
        "acre" => Some((UnitCategory::Area, 4046.8564224)),

        // Volume (base: L)
        "l" | "L" | "lt" => Some((UnitCategory::Volume, 1.0)),
        "ml" | "mL"      => Some((UnitCategory::Volume, 0.001)),
        "m3"             => Some((UnitCategory::Volume, 1000.0)),
        "km3"            => Some((UnitCategory::Volume, 1e12)),
        "ft3"            => Some((UnitCategory::Volume, 28.316846592)),
        "in3"            => Some((UnitCategory::Volume, 0.016387064)),
        "yd3"            => Some((UnitCategory::Volume, 764.554857984)),
        "mi3"            => Some((UnitCategory::Volume, 4_168_181_825.440_579_4)),
        "gal"            => Some((UnitCategory::Volume, 3.785411784)),
        "qt"             => Some((UnitCategory::Volume, 0.946352946)),
        "cup"            => Some((UnitCategory::Volume, 0.2365882365)),
        "fl_oz"          => Some((UnitCategory::Volume, 0.0295735295625)),
        "tsp"            => Some((UnitCategory::Volume, 0.00492892159375)),
        "tbs"            => Some((UnitCategory::Volume, 0.01478676478125)),
        "uk_pt"          => Some((UnitCategory::Volume, 0.56826125)),
        "uk_qt"          => Some((UnitCategory::Volume, 1.1365225)),
        "uk_gal"         => Some((UnitCategory::Volume, 4.54609)),
        "ang3"           => Some((UnitCategory::Volume, 1e-30)),

        // Magnetism (base: T)
        "T"  => Some((UnitCategory::Magnetism, 1.0)),
        "ga" => Some((UnitCategory::Magnetism, 0.0001)),

        // Information (base: bit)
        "bit"   => Some((UnitCategory::Information, 1.0)),
        "byte"  => Some((UnitCategory::Information, 8.0)),
        "kbit"  => Some((UnitCategory::Information, 1000.0)),
        "kbyte" => Some((UnitCategory::Information, 8000.0)),
        "Mbit"  => Some((UnitCategory::Information, 1e6)),
        "Mbyte" => Some((UnitCategory::Information, 8e6)),
        "Gbit"  => Some((UnitCategory::Information, 1e9)),
        "Gbyte" => Some((UnitCategory::Information, 8e9)),
        "Tbit"  => Some((UnitCategory::Information, 1e12)),
        "Tbyte" => Some((UnitCategory::Information, 8e12)),

        _ => None,
    }
}

fn is_temperature(name: &str) -> bool {
    matches!(name, "C" | "cel" | "F" | "fah" | "K" | "kel" | "Rank")
}

fn convert_temperature(value: f64, from: &str, to: &str) -> f64 {
    let celsius = match from {
        "C" | "cel" => value,
        "F" | "fah" => (value - 32.0) * 5.0 / 9.0,
        "K" | "kel" => value - 273.15,
        "Rank"      => (value - 491.67) * 5.0 / 9.0,
        _ => unreachable!(),
    };
    match to {
        "C" | "cel" => celsius,
        "F" | "fah" => celsius * 9.0 / 5.0 + 32.0,
        "K" | "kel" => celsius + 273.15,
        "Rank"      => (celsius + 273.15) * 9.0 / 5.0,
        _ => unreachable!(),
    }
}

fn coerce_to_number(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => Some(*n),
        Value::Bool(b)   => Some(if *b { 1.0 } else { 0.0 }),
        Value::Date(n)   => Some(*n),
        _ => None,
    }
}

pub fn convert_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }

    let value = match coerce_to_number(&args[0]) {
        Some(n) => n,
        None => return Value::Error(ErrorKind::Value),
    };

    let from = match &args[1] {
        Value::Text(s)  => s.clone(),
        Value::Error(_) => return args[1].clone(),
        _ => return Value::Error(ErrorKind::Value),
    };

    let to = match &args[2] {
        Value::Text(s)  => s.clone(),
        Value::Error(_) => return args[2].clone(),
        _ => return Value::Error(ErrorKind::Value),
    };

    if is_temperature(&from) || is_temperature(&to) {
        if !is_temperature(&from) || !is_temperature(&to) {
            return Value::Error(ErrorKind::NA);
        }
        let result = convert_temperature(value, &from, &to);
        if result.is_finite() {
            Value::Number(result)
        } else {
            Value::Error(ErrorKind::Num)
        }
    } else {
        let (from_cat, from_factor) = match get_unit(&from) {
            Some(u) => u,
            None => return Value::Error(ErrorKind::NA),
        };
        let (to_cat, to_factor) = match get_unit(&to) {
            Some(u) => u,
            None => return Value::Error(ErrorKind::NA),
        };
        if from_cat != to_cat {
            return Value::Error(ErrorKind::NA);
        }
        let result = value * from_factor / to_factor;
        if result.is_finite() {
            Value::Number(result)
        } else {
            Value::Error(ErrorKind::Num)
        }
    }
}

#[cfg(test)]
mod tests;
