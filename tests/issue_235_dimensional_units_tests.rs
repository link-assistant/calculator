//! General dimensional unit algebra (issue #235).

use link_calculator::types::{Dimension, PhysicalUnit};
use link_calculator::Calculator;

fn assert_result(input: &str, expected: &str) {
    let result = Calculator::new().calculate_internal(input);
    assert!(result.success, "{input:?}: {:?}", result.error);
    assert_eq!(result.result, expected, "{input:?}");
}

fn assert_error(input: &str, fragment: &str) {
    let result = Calculator::new().calculate_internal(input);
    assert!(
        !result.success,
        "{input:?} unexpectedly gave {}",
        result.result
    );
    let error = result.error.unwrap_or_default();
    assert!(error.contains(fragment), "{input:?}: {error}");
}

#[test]
fn evidence_rows_from_the_issue() {
    for (input, expected) in [
        ("10m × 10m", "100 m^2"),
        ("4m + 0", "4 m"),
        ("2m == 200cm", "true"),
        ("(5 volts) / (2 ohms)", "2.5 A"),
        ("2 amperes times 5 ohms", "10 V"),
        ("1 mph * 1 hour", "1 mi"),
        ("1/0.5 kg", "2 kg"),
        ("2 kg^2", "2 kg^2"),
        ("128 bit in byte", "16 B"),
        ("20 celsius", "293.15 K"),
        ("25 °C in °F", "77 °F"),
        ("5 µm in nm", "5000 nm"),
        ("1 ft to cm", "30.48 cm"),
        ("5 feet 10 inches in cm", "177.8 cm"),
        ("1 light year in miles", "5878625373183.61 mi"),
        ("299792458 m⋅s−1", "299792458 m/s"),
        ("1 J - 1 kg m^2 s^-2 + 1 kg / (m^-2 s^2)", "1 J"),
    ] {
        assert_result(input, expected);
    }
    assert_error("sqrt(2m)", "non-integer dimension");
}

#[test]
fn multiplication_division_powers_and_roots_combine_dimensions() {
    for (input, expected) in [
        ("(2 m)^2", "4 m^2"),
        ("sqrt(4 m^2)", "2 m"),
        ("cbrt(27 m^3)", "3 m"),
        ("10 V * 2 A", "20 W"),
        ("1 Pa * 1 m^2", "1 N"),
        ("1 N m in J", "1 J"),
        ("9.81 m/s² * 70 kg in N", "686.7 N"),
        ("1 kg * (299792458 m/s)^2 in J", "89875517873681764 J"),
        ("100 m / 10 s", "10 m/s"),
        ("4 m / 2 m", "2"),
        ("1 kWh in J", "3600000 J"),
        ("1 mi/h in m/s", "0.44704 m/s"),
        ("1 ft^2 in m^2", "0.09290304 m^2"),
    ] {
        assert_result(input, expected);
    }
}

#[test]
fn addition_requires_equal_dimensions() {
    assert_result("1 km + 500 m", "1.5 km");
    assert_result("3 feet + 1 m in cm", "191.44 cm");
    assert_result("4m + 0 kg", "4 m");
    assert_error("1 kg + 1 m", "cannot add");
    assert_error("1 m - 1 s^2", "subtract");
    assert_result("2m == 200cm", "true");
    assert_result("1 km > 999 m", "true");
}

#[test]
fn affine_temperatures_convert_but_reject_arithmetic() {
    for (input, expected) in [
        ("20 celsius", "293.15 K"),
        ("-5 °C", "268.15 K"),
        ("25 °C in °F", "77 °F"),
        ("25 ℃ in ℉", "77 °F"),
        ("100 °F in °C", "37.77777777777778 °C"),
        ("0 K in °C", "-273.15 °C"),
        ("300 K in celsius", "26.85 °C"),
        ("25 °C == 77 °F", "true"),
    ] {
        assert_result(input, expected);
    }
    // Numbat rejects arithmetic on affine units; convert to kelvin first.
    assert_error("25 °C * 2", "affine");
    assert_error("25 °C + 1 K", "affine");
    assert_error("(25 °C)^2", "affine");
    assert_error("°C * 2", "");
}

#[test]
fn si_prefixes_include_micro_spellings() {
    for input in ["5 µm in nm", "5 μm in nm", "5 um in nm"] {
        assert_result(input, "5000 nm");
    }
    assert_result("3 MHz in Hz", "3000000 Hz");
    assert_result("2 mA * 3 kΩ", "6 V");
    assert_result("1 GiB in MB", "1073.741824 MB");
    assert_result("1 KiB in bit", "8192 bit");
}

#[test]
fn imperial_electrical_and_angle_units() {
    assert_result("1 inch in mm", "25.4 mm");
    assert_result("1 gal in L", "3.785411784 L");
    assert_result("1 hp in W", "745.69987158227 W");
    assert_result("180 deg in rad", "3.141592653589793 rad");
    assert_result("sin(30°)", "0.5");
    assert_result("cos(180 deg)", "-1");
}

#[test]
fn aliases_include_plural_and_russian_names() {
    for (input, expected) in [
        ("1 foot in cm", "30.48 cm"),
        ("2 feet in cm", "60.96 cm"),
        ("2 ft in cm", "60.96 cm"),
        ("3 metres in cm", "300 cm"),
        ("3 meters in cm", "300 cm"),
        ("5 м in cm", "500 cm"),
        ("5 км in m", "5000 m"),
        ("2 ч in minutes", "120 minutes"),
    ] {
        assert_result(input, expected);
    }
}

#[test]
fn compound_quantities_sum_parts_of_the_same_dimension() {
    assert_result("5 feet 10 inches in cm", "177.8 cm");
    assert_result("1 meter 20 cm", "1.2 m");
    assert_result("6'2\" in cm", "187.96 cm");
    assert_result("6′2″ in cm", "187.96 cm");
    assert_result("1 h 30 min", "1.5 hours");
    // Different dimensions stay separate operands.
    assert_error("5 feet 10 kg", "");
}

#[test]
fn metre_and_minute_rule() {
    // `m` after a number is metre and `min` is minute.
    assert_result("5m", "5 m");
    assert_result("5 min", "5 minutes");
    assert_result("5 m in cm", "500 cm");
    // `5m` is a duration only in an explicit time context.
    assert_result("5m in s", "300 seconds");
    assert_result("5m in minutes", "5 minutes");
    assert_result("5m + 1h", "65 minutes");
    assert_result("2 km + 5m", "2.005 km");
    assert_result("5m + 2m", "7 m");
    // Other SI suffixes keep their prefix meaning.
    assert_result("5T", "5000000000000");
    assert_result("500m USD", "0.5 USD");
    assert_result("2 min(3, 4)", "6");
}

#[test]
fn unicode_and_ascii_unit_expressions() {
    for input in [
        "299792458 m⋅s−1",
        "299792458 m·s⁻¹",
        "299792458 m s^-1",
        "299792458 m*s^-1",
        "299792458 m/s",
        "299792458 m per s",
    ] {
        assert_result(input, "299792458 m/s");
    }
    assert_result("100 km/h", "100 km/h");
    assert_result("60 km per hour", "60 km/h");
    assert_result("1 kg·m²/s²", "1 kg m^2/s^2");
    assert_result("1 kg·m²/s² in J", "1 J");
    assert_result("5 m²", "5 m^2");
    assert_result("2 m³ in L", "2000 L");
    assert_result("1 kg m^2 s^(-2) in J", "1 J");
}

#[test]
fn legacy_units_do_not_regress() {
    for (input, expected) in [
        ("5 USD * 2", "10 USD"),
        ("2h in minutes", "120 minutes"),
        ("1 GB in MB", "1000 MB"),
        ("2 kg in g", "2000 g"),
        ("1 lb in kg", "0.45359237 kg"),
        ("5h USD", "500 USD"),
        ("2 pi", "6.28318530717959"),
        ("2π", "6.28318530717959"),
        ("10 meters + 20%", "12 m"),
    ] {
        assert_result(input, expected);
    }
}

#[test]
fn unit_database_exposes_dimensions() {
    let newton = PhysicalUnit::parse("N").expect("newton");
    let product = PhysicalUnit::parse("kg")
        .unwrap()
        .product(&PhysicalUnit::parse("m").unwrap())
        .product(&PhysicalUnit::parse("s").unwrap().powi(-2));
    assert_eq!(newton.dimension(), product.dimension());
    assert_eq!(
        PhysicalUnit::parse("ft").unwrap().dimension(),
        Dimension::base("length").unwrap()
    );
    assert!(PhysicalUnit::parse("°C").unwrap().has_affine());
}
