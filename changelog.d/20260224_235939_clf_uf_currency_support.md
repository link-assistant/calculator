### Fixed

- Add support for UF (Unidad de Fomento, ISO 4217: CLF) currency unit #20
  - `2 UF + 1 USD` now correctly converts between CLF and USD using default exchange rates
  - Both `UF` and `CLF` are recognized as the Chilean Unidad de Fomento
  - Natural language names ("unidad de fomento", "unidad fomento") are also supported
  - Added default USD/CLF exchange rate (1 USD ≈ 0.022 CLF, i.e. 1 CLF ≈ 45 USD)
  - Added historical rate data file `data/currency/usd-clf.lino`
