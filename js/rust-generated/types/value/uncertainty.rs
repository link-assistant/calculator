// meta-language:self-translation:v1 source=JavaScript target=Rust sha256=d6cb31e7668f5548bdc88c149116caba810b5e5dc4001720b9235b1a73e42f5b bytes=1514

// meta-language:prelude begin
#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]
// meta-language:prelude end

// meta-language:carried JavaScript comment (comment the target cannot hold)
// | /**
// |  * Measurements with an absolute uncertainty (port of
// |  * `src/types/value/uncertainty.rs`).
// |  */

// meta-language:translated JavaScript import_statement items=1 sha256=06145d87f70822cfbf65c7ddd45c0eb4f933e5915cd74bb9d541cc55ef0e6316
// | import { CalculatorError } from '../../error.js';
use crate::error::CalculatorError;
// meta-language:translated JavaScript import_statement items=1 sha256=861f79501be16f7495e6cbaf6f9dec8cef312906ff269349ccf839352aba15c7
// | import { eq, err, unreachable } from '../../rust/core.js';
use crate::rust::core_::{eq, err, unreachable};
// meta-language:translated JavaScript import_statement items=1 sha256=339676b2131126017f11e192c0e934b42ef2baed4d155de843d5cf1e32c63c5c
// | import { ValueKind } from './kind.js';
use crate::types::value::kind::ValueKind;
// meta-language:translated JavaScript import_statement items=1 sha256=6227932a669ff41f119a2566a3f97623e5311d86e6e144a0c641e05447aa9418
// | import { Value } from './index.js';
use crate::types::value::index::Value;

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::with_uncertainty`: creates a measurement with a non-negative
// |  * absolute error in the same unit. Throws `Err(CalculatorError)`.
// |  * @param {Value} self
// |  * @param {Value} error
// |  * @returns {Value}
// |  */
// | export function withUncertainty(self, error) {
// |   if (!eq(self.unit, error.unit)) {
// |     err(CalculatorError.InvalidOperation('uncertainty and central value must have matching units'));
// |   }
// |   const value = self.toRational();
// |   if (value === null) {
// |     err(CalculatorError.domain('central value must be numeric'));
// |   }
// |   const uncertainty = error.toRational();
// |   if (uncertainty === null) {
// |     err(CalculatorError.domain('uncertainty must be numeric'));
// |   }
// |   if (uncertainty.isNegative()) {
// |     err(CalculatorError.domain('uncertainty must be non-negative'));
// |   }
// |   return new Value({ kind: ValueKind.Uncertainty({ value, uncertainty }), unit: self.unit });
// | }

// meta-language:carried JavaScript export_statement (unsupported)
// | /**
// |  * `Value::negate_uncertainty`.
// |  * @param {Value} self
// |  * @returns {Value}
// |  */
// | export function negateUncertainty(self) {
// |   const kind = self.kind;
// |   if (kind.tag !== 'Uncertainty') {
// |     unreachable();
// |   }
// |   return new Value({
// |     kind: ValueKind.Uncertainty({ value: kind.value.neg(), uncertainty: kind.uncertainty }),
// |     unit: self.unit,
// |   });
// | }
