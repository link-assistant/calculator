### Fixed

- Improved Links Notation for indefinite integrals to be more explicit and unambiguous. The differential variable is now shown as `(differential of (x))` and the multiplication between the integrand and differential is made explicit. For example, `integrate cos(x) dx` now produces `(integrate ((cos (x)) * (differential of (x))))` instead of `(integrate (cos (x)) dx)`.
