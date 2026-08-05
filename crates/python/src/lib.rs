//! Python bindings for `truecalc-core`.
//!
//! # Value mapping
//!
//! Spreadsheet values map onto native Python types wherever the mapping is
//! lossless, and onto a small set of wrapper classes where it is not:
//!
//! | Engine value | Python |
//! |---|---|
//! | `Number` | `float` |
//! | `Text` | `str` |
//! | `Bool` | `bool` |
//! | `Empty` | `None` |
//! | `Array` | `list` (nested for 2-D) |
//! | `Date` | [`Date`] — carries the serial; `float` would erase the distinction |
//! | `Zoned` | [`Zoned`] — RFC-9557 string plus accessors |
//! | `Error`/`ErrorMsg` | [`Error`] — **returned, not raised**; see below |
//! | `Sparkline` | [`Sparkline`] |
//!
//! # Errors are values
//!
//! `=1/0` evaluates to `#DIV/0!`. That is a *result*, not a failure: spreadsheet
//! formulas routinely branch on it (`IFERROR`, `ISNA`), and a `SUM` over a range
//! containing one propagates it. Raising by default would therefore diverge from
//! both the Rust and JS surfaces and make conformance untestable from Python.
//!
//! So [`Error`] is returned like any other value. Callers who genuinely want
//! exception control flow opt in per call with `raise_on_error=True`, which
//! raises [`FormulaError`].
//!
//! Note that this is separate from *malformed input*: a formula that does not
//! parse raises `ValueError` regardless, because there is no spreadsheet value
//! to represent it.

use std::collections::HashMap;

use pyo3::exceptions::{PyValueError, PyZeroDivisionError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyList, PyString};

use truecalc_core::types::{SparklineValue, Value};
use truecalc_core::{Engine as CoreEngine, Registry};

// ─── Exception ───────────────────────────────────────────────────────────────

pyo3::create_exception!(
    _truecalc,
    FormulaError,
    pyo3::exceptions::PyException,
    "Raised when a formula evaluates to a spreadsheet error and the caller \
     passed `raise_on_error=True`."
);

// ─── Wrapper value types ─────────────────────────────────────────────────────

/// A spreadsheet error value, e.g. `#DIV/0!`.
///
/// Returned rather than raised — see the module docs. Truthiness is `False`, so
/// `if not result:` catches errors alongside empty cells, which is the check
/// most callers actually want.
#[pyclass(module = "truecalc.core", frozen)]
pub struct Error {
    /// The spreadsheet error code, e.g. `"#DIV/0!"`.
    #[pyo3(get)]
    code: String,
    /// A human-readable diagnostic when the engine produced one, else `None`.
    #[pyo3(get)]
    message: Option<String>,
}

#[pymethods]
impl Error {
    fn __repr__(&self) -> String {
        match &self.message {
            Some(m) => format!("Error({:?}, message={:?})", self.code, m),
            None => format!("Error({:?})", self.code),
        }
    }

    fn __str__(&self) -> String {
        self.code.clone()
    }

    fn __bool__(&self) -> bool {
        false
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        // Compare by code, matching the engine's own equality: two errors of the
        // same kind are equal whatever diagnostic they carry.
        if let Ok(other) = other.extract::<PyRef<'_, Error>>() {
            return self.code == other.code;
        }
        if let Ok(s) = other.extract::<String>() {
            return self.code == s;
        }
        false
    }

    fn __hash__(&self) -> u64 {
        let mut h: u64 = 1469598103934665603;
        for b in self.code.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(1099511628211);
        }
        h
    }
}

/// A spreadsheet serial date.
///
/// Kept distinct from `float` because the engine distinguishes them: `ISDATE`
/// is `True` here and `False` for a plain number carrying the same value.
/// Day 0 is 1899-12-30 under the `sheets` flavor.
#[pyclass(module = "truecalc.core", frozen)]
pub struct Date {
    /// The underlying serial number.
    #[pyo3(get)]
    serial: f64,
}

#[pymethods]
impl Date {
    fn __repr__(&self) -> String {
        format!("Date({})", self.serial)
    }

    fn __float__(&self) -> f64 {
        self.serial
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<PyRef<'_, Date>>() {
            return self.serial == other.serial;
        }
        // A bare float is deliberately NOT equal to a Date: the engine treats
        // them as different types, and silently equating them here would hide
        // exactly the distinction this class exists to preserve.
        false
    }

    /// Convert to a `datetime.datetime`, interpreting the serial under the
    /// given epoch (default 1899-12-30, the `sheets` flavor).
    #[pyo3(signature = (epoch = None))]
    fn to_datetime<'py>(
        &self,
        py: Python<'py>,
        epoch: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let datetime = py.import("datetime")?;
        let base = match epoch {
            Some(e) => e.clone(),
            None => datetime
                .getattr("datetime")?
                .call1((1899, 12, 30))?,
        };
        let delta = datetime
            .getattr("timedelta")?
            .call1((self.serial,))?;
        base.call_method1("__add__", (delta,))
    }
}

/// A zone-aware instant, carried as its canonical RFC-9557 string.
#[pyclass(module = "truecalc.core", frozen)]
pub struct Zoned {
    /// The RFC-9557 representation, e.g.
    /// `2026-07-14T11:00:00+02:00[Europe/Berlin]`.
    #[pyo3(get)]
    value: String,
}

#[pymethods]
impl Zoned {
    fn __repr__(&self) -> String {
        format!("Zoned({:?})", self.value)
    }

    fn __str__(&self) -> String {
        self.value.clone()
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other) = other.extract::<PyRef<'_, Zoned>>() {
            return self.value == other.value;
        }
        if let Ok(s) = other.extract::<String>() {
            return self.value == s;
        }
        false
    }
}

/// A parsed `SPARKLINE` render spec.
#[pyclass(module = "truecalc.core", frozen)]
pub struct Sparkline {
    /// `"line"` (the default), `"bar"`, `"column"` or `"winloss"`.
    #[pyo3(get)]
    charttype: String,
    /// The points to plot, row-major.
    #[pyo3(get)]
    data: Py<PyList>,
    /// Remaining option key/value pairs, keys lower-cased, order preserved.
    #[pyo3(get)]
    options: Py<PyList>,
}

#[pymethods]
impl Sparkline {
    fn __repr__(&self) -> String {
        format!("Sparkline(charttype={:?})", self.charttype)
    }
}

// ─── Value conversion ────────────────────────────────────────────────────────

fn sparkline_value_to_py<'py>(py: Python<'py>, v: &SparklineValue) -> PyResult<Bound<'py, PyAny>> {
    Ok(match v {
        SparklineValue::Number(n) => PyFloat::new(py, *n).into_any(),
        SparklineValue::Text(s) => PyString::new(py, s).into_any(),
        SparklineValue::Bool(b) => PyBool::new(py, *b).to_owned().into_any(),
        SparklineValue::Blank => py.None().into_bound(py),
    })
}

/// Map an engine [`Value`] onto its Python representation.
fn value_to_py<'py>(py: Python<'py>, value: Value) -> PyResult<Bound<'py, PyAny>> {
    Ok(match value {
        Value::Number(n) => PyFloat::new(py, n).into_any(),
        Value::Text(s) => PyString::new(py, &s).into_any(),
        Value::Bool(b) => PyBool::new(py, b).to_owned().into_any(),
        Value::Empty => py.None().into_bound(py),
        Value::Date(n) => Date { serial: n }.into_pyobject(py)?.into_any(),
        Value::Zoned(z) => Zoned { value: z.to_rfc9557() }.into_pyobject(py)?.into_any(),
        Value::Error(kind) => Error { code: kind.to_string(), message: None }
            .into_pyobject(py)?
            .into_any(),
        Value::ErrorMsg(kind, msg) => Error { code: kind.to_string(), message: Some(msg) }
            .into_pyobject(py)?
            .into_any(),
        Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(value_to_py(py, item)?)?;
            }
            list.into_any()
        }
        Value::Sparkline(spec) => {
            let data = PyList::empty(py);
            for v in spec.data.iter() {
                data.append(sparkline_value_to_py(py, v)?)?;
            }
            let options = PyList::empty(py);
            for (k, v) in spec.options.iter() {
                options.append((k.clone(), sparkline_value_to_py(py, v)?))?;
            }
            Sparkline {
                charttype: spec.chart_type.as_str().to_string(),
                data: data.unbind(),
                options: options.unbind(),
            }
            .into_pyobject(py)?
            .into_any()
        }
    })
}

/// Map a Python object supplied as a variable onto an engine [`Value`].
///
/// `bool` is checked before the numeric extraction because `bool` is a subclass
/// of `int` in Python — without the ordering, `True` would silently arrive as
/// the number 1 and change `IF`/`AND` semantics.
fn py_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Empty);
    }
    if let Ok(b) = obj.cast::<PyBool>() {
        return Ok(Value::Bool(b.is_true()));
    }
    if let Ok(n) = obj.extract::<f64>() {
        if !n.is_finite() {
            // The engine's Number invariant forbids NaN/inf; surface it as the
            // spreadsheet error rather than constructing an invalid Value.
            return Ok(Value::Error(truecalc_core::ErrorKind::Num));
        }
        return Ok(Value::Number(n));
    }
    if let Ok(s) = obj.extract::<String>() {
        return Ok(Value::Text(s));
    }
    if let Ok(e) = obj.extract::<PyRef<'_, Error>>() {
        for kind in truecalc_core::ErrorKind::LITERAL_KINDS {
            if kind.to_string() == e.code {
                return Ok(Value::Error(kind));
            }
        }
        return Ok(Value::Error(truecalc_core::ErrorKind::Value));
    }
    if let Ok(d) = obj.extract::<PyRef<'_, Date>>() {
        return Ok(Value::Date(d.serial));
    }
    Err(PyValueError::new_err(format!(
        "unsupported variable type {}: expected float, int, str, bool, None, Date or Error",
        obj.get_type().name()?
    )))
}

fn variables_from_py(vars: Option<&Bound<'_, PyDict>>) -> PyResult<HashMap<String, Value>> {
    let mut out = HashMap::new();
    if let Some(dict) = vars {
        for (k, v) in dict.iter() {
            let key: String = k.extract().map_err(|_| {
                PyValueError::new_err("variable names must be strings, e.g. \"A1\"")
            })?;
            out.insert(key, py_to_value(&v)?);
        }
    }
    Ok(out)
}

/// Convert a returned error value into a raised exception, when the caller
/// asked for that. `#DIV/0!` maps onto Python's own `ZeroDivisionError` so it
/// composes with ordinary `except` blocks; everything else raises
/// [`FormulaError`].
fn raise_for_error(value: &Bound<'_, PyAny>) -> PyResult<()> {
    if let Ok(err) = value.extract::<PyRef<'_, Error>>() {
        let detail = match &err.message {
            Some(m) => format!("{}: {}", err.code, m),
            None => err.code.clone(),
        };
        return Err(if err.code == "#DIV/0!" {
            PyZeroDivisionError::new_err(detail)
        } else {
            FormulaError::new_err(detail)
        });
    }
    Ok(())
}

// ─── Engine ──────────────────────────────────────────────────────────────────

/// A formula engine locked to one conformance target.
///
/// The flavor is required and immutable — construct with [`Engine::sheets`] or
/// [`Engine::excel`] and reuse the instance.
#[pyclass(module = "truecalc.core", frozen)]
pub struct Engine {
    inner: CoreEngine,
    flavor: &'static str,
}

#[pymethods]
impl Engine {
    /// An engine targeting Google Sheets conformance.
    #[staticmethod]
    fn sheets() -> Self {
        Engine { inner: CoreEngine::sheets(), flavor: "sheets" }
    }

    /// An engine targeting Excel conformance.
    ///
    /// Parse and validate only — evaluation returns `#UNSUPPORTED!` pending
    /// the Excel evaluation work.
    #[staticmethod]
    fn excel() -> Self {
        Engine { inner: CoreEngine::excel(), flavor: "excel" }
    }

    /// The conformance target this engine is locked to.
    #[getter]
    fn flavor(&self) -> &str {
        self.flavor
    }

    /// Evaluate `formula`, resolving references against `variables`.
    ///
    /// Returns a spreadsheet error as an [`Error`] value. Pass
    /// `raise_on_error=True` to raise instead.
    ///
    /// Raises `ValueError` if the formula does not parse — that is malformed
    /// input rather than a spreadsheet result.
    #[pyo3(signature = (formula, variables = None, *, raise_on_error = false))]
    fn evaluate<'py>(
        &self,
        py: Python<'py>,
        formula: &str,
        variables: Option<&Bound<'py, PyDict>>,
        raise_on_error: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let vars = variables_from_py(variables)?;
        if let Err(e) = self.inner.validate(formula) {
            return Err(PyValueError::new_err(format!(
                "could not parse formula: {}",
                e
            )));
        }
        let result = py.detach(|| self.inner.evaluate(formula, &vars));
        let obj = value_to_py(py, result)?;
        if raise_on_error {
            raise_for_error(&obj)?;
        }
        Ok(obj)
    }

    /// Check whether `formula` parses, without evaluating it.
    ///
    /// Returns `None` when valid, else the parse error message.
    fn validate(&self, formula: &str) -> Option<String> {
        self.inner.validate(formula).err().map(|e| e.to_string())
    }

    /// Shift every relative reference in `formula` by `d_row`/`d_col`, as
    /// filling a formula down or across a sheet would.
    fn translate(&self, formula: &str, d_row: i64, d_col: i64) -> PyResult<String> {
        self.inner
            .translate_formula(formula, d_row, d_col)
            .map_err(|e| PyValueError::new_err(format!("could not parse formula: {}", e)))
    }

    /// Rewrite every reference to sheet `old` so it points at `new`.
    fn rename_sheet_refs(&self, formula: &str, old: &str, new: &str) -> PyResult<String> {
        self.inner
            .rename_sheet_refs(formula, old, new)
            .map_err(|e| PyValueError::new_err(format!("could not parse formula: {}", e)))
    }

    fn __repr__(&self) -> String {
        format!("Engine.{}()", self.flavor)
    }
}

// ─── Module-level helpers ────────────────────────────────────────────────────

/// Evaluate a formula against Google Sheets conformance.
///
/// Convenience wrapper over `Engine.sheets().evaluate(...)`. Constructing an
/// engine is cheap but not free — prefer reusing one [`Engine`] in a loop.
#[pyfunction]
#[pyo3(signature = (formula, variables = None, *, raise_on_error = false))]
fn evaluate<'py>(
    py: Python<'py>,
    formula: &str,
    variables: Option<&Bound<'py, PyDict>>,
    raise_on_error: bool,
) -> PyResult<Bound<'py, PyAny>> {
    Engine::sheets().evaluate(py, formula, variables, raise_on_error)
}

/// Check whether a formula parses. Returns `None` when valid, else the message.
#[pyfunction]
fn validate(formula: &str) -> Option<String> {
    Engine::sheets().validate(formula)
}

/// Every function the engine implements, as dicts with `name`, `category`,
/// `signature` and `description`.
#[pyfunction]
fn list_functions(py: Python<'_>) -> PyResult<Bound<'_, PyList>> {
    let registry = Registry::new();
    let out = PyList::empty(py);
    for entry in registry.get_metadata() {
        let d = PyDict::new(py);
        d.set_item("name", entry.name)?;
        d.set_item("category", entry.meta.category)?;
        d.set_item("signature", entry.meta.signature)?;
        d.set_item("description", entry.meta.description)?;
        out.append(d)?;
    }
    Ok(out)
}

#[pymodule]
fn _truecalc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Engine>()?;
    m.add_class::<Error>()?;
    m.add_class::<Date>()?;
    m.add_class::<Zoned>()?;
    m.add_class::<Sparkline>()?;
    m.add("FormulaError", m.py().get_type::<FormulaError>())?;
    m.add_function(wrap_pyfunction!(evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(list_functions, m)?)?;
    Ok(())
}
