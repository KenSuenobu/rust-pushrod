// Validation tests: masks, integer and number validators.

use Validity::{Intermediate, Invalid, Valid};
use pushrod_widgets::text_field::validate::{
    IntegerValidator, Mask, NumberValidator, Validator, Validity,
};

#[test]
fn a_mask_judges_prefixes_and_whole_texts() {
    let mask = Mask::new("AA-99-X");

    assert_eq!(mask.len(), 7);
    assert!(!mask.is_empty());
    assert_eq!(mask.validate(""), Intermediate);
    assert_eq!(mask.validate("ab-1"), Intermediate);
    assert_eq!(mask.validate("ab-12-!"), Valid);
    assert_eq!(mask.validate("a1"), Invalid);
    assert_eq!(mask.validate("ab_"), Invalid);
    assert_eq!(mask.validate("ab-12-xx"), Invalid, "too long");
}

#[test]
fn an_empty_mask_accepts_only_empty_text() {
    let mask = Mask::new("");

    assert!(mask.is_empty());
    assert_eq!(mask.validate(""), Valid);
    assert_eq!(mask.validate("a"), Invalid);
    assert_eq!(mask.append(&[], 'a'), None);
}

#[test]
fn mask_append_fills_literals_and_trim_drops_them() {
    let mask = Mask::new("99/99");
    let two: Vec<char> = "12".chars().collect();

    assert_eq!(
        mask.append(&two, '3').unwrap().iter().collect::<String>(),
        "12/3"
    );
    assert_eq!(
        mask.append(&two, '/').unwrap().iter().collect::<String>(),
        "12/"
    );
    assert_eq!(mask.append(&two, 'x'), None);
    assert_eq!(mask.literal_at(2), Some('/'));
    assert_eq!(mask.literal_at(0), None);
    assert_eq!(mask.trim(&"12/".chars().collect::<Vec<_>>()), two);
    assert!(mask.accepts(0, '5'));
    assert!(!mask.accepts(9, '5'));
}

#[test]
fn integers_in_range() {
    let v = IntegerValidator::new(120, 0);

    assert_eq!(v.validate(""), Intermediate);
    assert_eq!(v.validate("0"), Valid);
    assert_eq!(v.validate("120"), Valid);
    assert_eq!(v.validate("121"), Invalid);
    assert_eq!(v.validate("-1"), Invalid);
    assert_eq!(v.validate("1.5"), Invalid);
    assert_eq!(v.validate("99999999999999999999"), Invalid);

    let signed = IntegerValidator::new(-10, 10);

    assert_eq!(signed.validate("-"), Intermediate);
    assert_eq!(signed.validate("-10"), Valid);
    assert_eq!(signed.validate("-11"), Invalid);

    let high = IntegerValidator::new(10, 20);

    assert_eq!(high.validate("1"), Intermediate);
    assert_eq!(high.validate("5"), Intermediate);
    assert_eq!(high.validate("100"), Invalid);

    let negative = IntegerValidator::new(-20, -10);

    assert_eq!(negative.validate("-1"), Intermediate);
    assert_eq!(negative.validate("-15"), Valid);
}

#[test]
fn numbers_in_range() {
    let v = NumberValidator::new(0.0, 100.0, 2);

    assert_eq!(v.validate(""), Intermediate);
    assert_eq!(v.validate("."), Intermediate);
    assert_eq!(v.validate("3."), Valid);
    assert_eq!(v.validate(".5"), Valid);
    assert_eq!(v.validate("3.14"), Valid);
    assert_eq!(v.validate("3.141"), Invalid);
    assert_eq!(v.validate("3.1.4"), Invalid);
    assert_eq!(v.validate("100.01"), Invalid);
    assert_eq!(v.validate("-1"), Invalid);
    assert_eq!(v.validate("abc"), Invalid);

    let whole = NumberValidator::new(f64::NAN, f64::INFINITY, 0);

    assert_eq!(whole.validate("0"), Valid);
    assert_eq!(whole.validate("0."), Invalid, "no decimals allowed");

    let signed = NumberValidator::new(-5.0, 5.0, 1);

    assert_eq!(signed.validate("-"), Intermediate);
    assert_eq!(signed.validate("-5.5"), Invalid);
    assert_eq!(signed.validate("-2.5"), Valid);
}
