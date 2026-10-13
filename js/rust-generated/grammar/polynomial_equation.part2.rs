// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {Rational} value @param {number} degree @returns {Rational | null} */
// | function rationalNthRoot(value, degree) {
// |   if (value.isNegative()) {
// |     if (degree % 2 === 0) {
// |       return null;
// |     }
// |
// |     const root = rationalNthRoot(value.abs(), degree);
// |     return root === null ? null : root.neg();
// |   }
// |
// |   const numerator = U128.tryFrom(value.numer());
// |   if (numerator === null) {
// |     return null;
// |   }
// |   const denominator = U128.tryFrom(value.denom());
// |   if (denominator === null) {
// |     return null;
// |   }
// |   const numeratorRoot = integerNthRoot(numerator, degree);
// |   if (numeratorRoot === null) {
// |     return null;
// |   }
// |   const denominatorRoot = integerNthRoot(denominator, degree);
// |   if (denominatorRoot === null) {
// |     return null;
// |   }
// |
// |   const numer = I128.tryFrom(numeratorRoot);
// |   const denom = I128.tryFrom(denominatorRoot);
// |   if (numer === null || denom === null) {
// |     return null;
// |   }
// |   return Rational.new(numer, denom);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {bigint} value @param {number} degree @returns {bigint | null} */
// | function integerNthRoot(value, degree) {
// |   let low = 0n;
// |   let high = value;
// |
// |   while (low <= high) {
// |     const midpoint = low + (high - low) / 2n;
// |     const power = checkedPowU128(midpoint, degree);
// |     if (power !== null && power === value) {
// |       return midpoint;
// |     } else if (power !== null && power < value) {
// |       low = midpoint + 1n;
// |     } else {
// |       if (midpoint === 0n) {
// |         break;
// |       }
// |       high = midpoint - 1n;
// |     }
// |   }
// |
// |   return null;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {bigint} base @param {number} exponent @returns {bigint | null} */
// | function checkedPowU128(base, exponent) {
// |   /** @type {bigint | null} */
// |   let result = 1n;
// |
// |   for (let i = 0; i < exponent; i += 1) {
// |     result = U128.checkedMul(result, base);
// |     if (result === null) {
// |       return null;
// |     }
// |   }
// |
// |   return result;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {PolynomialForm} polynomial @returns {Rational[]} */
// | function rationalRootCandidates(polynomial) {
// |   /** @type {Map<string, Rational>} */
// |   const candidates = new Map();
// |   for (let n = -ROOT_SEARCH_BOUND; n <= ROOT_SEARCH_BOUND; n += 1n) {
// |     insertRational(candidates, Rational.fromBigint(n));
// |   }
// |
// |   const exactCandidates = rationalRootTheoremCandidates(polynomial);
// |   if (exactCandidates !== null) {
// |     for (const candidate of exactCandidates) {
// |       insertRational(candidates, candidate);
// |     }
// |   }
// |
// |   return sortedRationals(candidates);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {PolynomialForm} polynomial @returns {Rational[] | null} */
// | function rationalRootTheoremCandidates(polynomial) {
// |   if (polynomial.terms.size === 0) {
// |     return null;
// |   }
// |   const lowestDegree = Math.min(...polynomial.terms.keys());
// |   const highestDegree = polynomial.degree();
// |   /** @type {bigint | null} */
// |   let denominatorLcm = 1n;
// |
// |   for (const [, coefficient] of sortedTerms(polynomial.terms)) {
// |     denominatorLcm = checkedLcm(denominatorLcm, coefficient.denom());
// |     if (denominatorLcm === null) {
// |       return null;
// |     }
// |   }
// |
// |   const constant = scaledIntegerCoefficient(polynomial, lowestDegree, denominatorLcm);
// |   if (constant === null) {
// |     return null;
// |   }
// |   const leading = scaledIntegerCoefficient(polynomial, highestDegree, denominatorLcm);
// |   if (leading === null) {
// |     return null;
// |   }
// |   const numeratorFactors = positiveDivisors(constant);
// |   if (numeratorFactors === null) {
// |     return null;
// |   }
// |   const denominatorFactors = positiveDivisors(leading);
// |   if (denominatorFactors === null) {
// |     return null;
// |   }
// |   /** @type {Map<string, Rational>} */
// |   const candidates = new Map();
// |
// |   for (const numerator of numeratorFactors) {
// |     for (const denominator of denominatorFactors) {
// |       insertRational(candidates, Rational.new(numerator, denominator));
// |       insertRational(candidates, Rational.new(-numerator, denominator));
// |     }
// |   }
// |
// |   if (lowestDegree > 0) {
// |     insertRational(candidates, Rational.zero());
// |   }
// |
// |   return sortedRationals(candidates);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /**
// |  * @param {PolynomialForm} polynomial
// |  * @param {number} degree
// |  * @param {bigint} denominatorLcm
// |  * @returns {bigint | null}
// |  */
// | function scaledIntegerCoefficient(polynomial, degree, denominatorLcm) {
// |   const coefficient = polynomial.coefficient(degree);
// |   const factor = I128.checkedDiv(denominatorLcm, coefficient.denom());
// |   if (factor === null) {
// |     return null;
// |   }
// |   return I128.checkedMul(coefficient.numer(), factor);
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {bigint} left @param {bigint} right @returns {bigint | null} */
// | function checkedLcm(left, right) {
// |   const gcd = gcdI128(left, right);
// |   const quotient = I128.checkedDiv(left, gcd);
// |   if (quotient === null) {
// |     return null;
// |   }
// |   const product = I128.checkedMul(quotient, right);
// |   if (product === null) {
// |     return null;
// |   }
// |   return I128.checkedAbs(product);
// | }

// meta-language:translated JavaScript function_declaration items=2 sha256=5e3d8d289e3b9eb71a285603885604200766fc04e90fab7ee05987d2ebbe104e
// | /** @param {bigint} left @param {bigint} right @returns {bigint} */
// | function gcdI128(left, right) {
// |   left = left < 0n ? -left : left;
// |   right = right < 0n ? -right : right;
// |
// |   while (right !== 0n) {
// |     const remainder = left % right;
// |     left = right;
// |     right = remainder;
// |   }
// |
// |   return left;
// | }
pub fn gcd_i128(left: crate::ml::Big, right: crate::ml::Big) -> crate::ml::Big {
    {
        let left_2 = if (left < (crate::ml::Big::from_i128(0))) {
            left.neg()
        } else {
            left.clone()
        };
        {
            let right_2 = if (right < (crate::ml::Big::from_i128(0))) {
                right.neg()
            } else {
                right.clone()
            };
            {
                let left_3 = crate::ml_gcd_i128_loop1(left_2.clone(), right_2.clone());
                left_3.clone()
            }
        }
    }
}

pub fn ml_gcd_i128_loop1(mut left: crate::ml::Big, mut right: crate::ml::Big) -> crate::ml::Big {
    loop {
        return if (right != (crate::ml::Big::from_i128(0))) {
            {
                let remainder = crate::ml::divide(&left, &right, "trunc", Some("Division by zero"), true);
                {
                    let left_2 = right.clone();
                    {
                        let right_2 = remainder.clone();
                        {
                            (left, right) = (left_2.clone(), right_2.clone());
                            continue;
                        }
                    }
                }
            }
        } else {
            left.clone()
        };
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {bigint} value @returns {bigint[] | null} */
// | function positiveDivisors(value) {
// |   const abs = I128.checkedAbs(value);
// |   if (abs === null) {
// |     return null;
// |   }
// |   if (abs === 0n || abs > MAX_FACTORABLE_COEFFICIENT_ABS) {
// |     return null;
// |   }
// |
// |   /** @type {bigint[]} */
// |   const divisors = [];
// |   let candidate = 1n;
// |   while (candidate <= abs / candidate) {
// |     if (abs % candidate === 0n) {
// |       divisors.push(candidate);
// |       const paired = abs / candidate;
// |       if (paired !== candidate) {
// |         divisors.push(paired);
// |       }
// |     }
// |     candidate += 1n;
// |   }
// |
// |   return divisors;
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {string} variable @param {Rational[]} roots @returns {string} */
// | function formatSolutions(variable, roots) {
// |   return roots.map((root) => `${variable} = ${root.toDisplayString()}`).join(' or ');
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {string} variable @param {Rational[]} roots @returns {string} */
// | function formatRootFactors(variable, roots) {
// |   return roots.map((root) => formatRootFactor(variable, root)).join(' * ');
// | }

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {string} variable @param {Rational} root @returns {string} */
// | function formatRootFactor(variable, root) {
// |   const v = formatVariablePower(variable, 1);
// |   if (root.isZero()) {
// |     return v;
// |   }
// |
// |   if (root.isNegative()) {
// |     return `(${v} + ${root.abs().toDisplayString()})`;
// |   } else {
// |     return `(${v} - ${root.toDisplayString()})`;
// |   }
// | }

// meta-language:translated JavaScript function_declaration items=1 sha256=2357b1fb0b013b225d2fb4e53367b413c24bdb33b5bc9d7ee9b8bb5b2ddc6e44
// | /** @param {string} variable @param {number} degree @returns {string} */
// | function formatVariablePower(variable, degree) {
// |   const v = variable === '*' ? '(*)' : variable;
// |
// |   if (degree === 1) {
// |     return v;
// |   } else {
// |     return `${v}^${degree}`;
// |   }
// | }
pub fn format_variable_power(variable: String, degree: f64) -> String {
    {
        let v = if (variable == "*") {
            String::from("(*)")
        } else {
            variable.clone()
        };
        if (degree == (1f64)) {
            v.clone()
        } else {
            format!("{}{}", (format!("{}{}", v, (String::from("^")))), (crate::ml_number::js_number(degree)))
        }
    }
}

// meta-language:carried JavaScript function_declaration (unsupported)
// | /** @param {[boolean, string][]} parts @returns {string} */
// | function joinSignedParts(parts) {
// |   let result = '';
// |
// |   parts.forEach(([isNegative, body], index) => {
// |     if (index === 0) {
// |       if (isNegative) {
// |         result += '-';
// |       }
// |     } else if (isNegative) {
// |       result += ' - ';
// |     } else {
// |       result += ' + ';
// |     }
// |     result += body;
// |   });
// |
// |   return result === '' ? '0' : result;
// | }
