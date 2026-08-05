"""Formula engine with exact Google Sheets semantics.

    >>> from truecalc.core import Engine
    >>> engine = Engine.sheets()
    >>> engine.evaluate("=SUM(A1,A2,A3)", {"A1": 10, "A2": 20, "A3": 30})
    60.0

Spreadsheet errors are *values*, not exceptions -- `=1/0` returns an `Error`,
matching how formulas actually behave (`IFERROR` and `ISNA` exist because
errors flow through expressions). Opt into exceptions per call when you want
them::

    >>> engine.evaluate("=1/0")
    Error("#DIV/0!")
    >>> engine.evaluate("=1/0", raise_on_error=True)
    Traceback (most recent call last):
    truecalc.core.FormulaError: #DIV/0!

Note that `truecalc` is a PEP 420 namespace package: `truecalc.core` and
`truecalc.workbook` ship as separate distributions and install side by side.
"""

from ._truecalc import (
    Date,
    Engine,
    Error,
    FormulaError,
    Sparkline,
    Zoned,
    __version__,
    evaluate,
    list_functions,
    validate,
)

__all__ = [
    "Date",
    "Engine",
    "Error",
    "FormulaError",
    "Sparkline",
    "Zoned",
    "__version__",
    "evaluate",
    "list_functions",
    "validate",
]
