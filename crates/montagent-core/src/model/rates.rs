//! Exact rational frame rate representation (ADR-0199).

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// The sequence frame rate in frames per second (ADR-0199).
///
/// Accepts a whole integer (`24`, `25`, `30`, `50`, `60`), an exact rational object
/// `{"num": 24000, "den": 1001}`, or an exact rational pair `[24000, 1001]`.
/// Decimal floats (such as `23.976`) are strictly rejected by schema and `validate` (`E-SCHEMA`).
#[derive(Debug, Clone, Copy, Eq)]
pub struct FrameRate {
    num: i64,
    den: i64,
}

impl FrameRate {
    /// Create a frame rate from exact numerator and denominator.
    ///
    /// Reduces the fraction by their greatest common divisor.
    pub fn new(num: i64, den: i64) -> Self {
        if den == 0 {
            return Self { num, den: 0 };
        }
        let g = gcd(num, den);
        if g == 0 {
            return Self { num: 0, den: 1 };
        }
        let sign = if den < 0 { -1 } else { 1 };
        Self {
            num: (num / g) * sign,
            den: (den / g).abs(),
        }
    }

    /// Construct a whole-integer frame rate.
    pub const fn integer(fps: i64) -> Self {
        Self { num: fps, den: 1 }
    }

    /// True if both numerator and denominator are strictly positive.
    #[inline]
    pub const fn is_positive(&self) -> bool {
        self.num > 0 && self.den > 0
    }

    /// Numerator in frames per second.
    #[inline]
    pub const fn num(&self) -> i64 {
        self.num
    }

    /// Denominator in frames per second.
    #[inline]
    pub const fn den(&self) -> i64 {
        self.den
    }

    /// Is this rate an exact integer?
    #[inline]
    pub const fn is_integer(&self) -> bool {
        self.den == 1
    }

    /// Approximate floating-point value for display/logging.
    #[inline]
    pub fn as_f64(&self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

impl From<i64> for FrameRate {
    fn from(fps: i64) -> Self {
        Self::new(fps, 1)
    }
}

impl From<[i64; 2]> for FrameRate {
    fn from([num, den]: [i64; 2]) -> Self {
        Self::new(num, den)
    }
}

impl PartialEq for FrameRate {
    fn eq(&self, other: &Self) -> bool {
        // Since `new` always normalizes via `gcd`, comparing components is exact.
        self.num == other.num && self.den == other.den
    }
}

impl PartialEq<i64> for FrameRate {
    fn eq(&self, other: &i64) -> bool {
        self.den == 1 && self.num == *other
    }
}

impl PartialEq<FrameRate> for i64 {
    fn eq(&self, other: &FrameRate) -> bool {
        other.den == 1 && other.num == *self
    }
}

impl fmt::Display for FrameRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl Serialize for FrameRate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.den == 1 {
            serializer.serialize_i64(self.num)
        } else {
            #[derive(Serialize)]
            struct RationalObject {
                num: i64,
                den: i64,
            }
            RationalObject {
                num: self.num,
                den: self.den,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for FrameRate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FrameRateVisitor;

        impl<'de> de::Visitor<'de> for FrameRateVisitor {
            type Value = FrameRate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a positive integer, an exact rational object {\"num\": N, \"den\": D}, or an \
                     exact rational pair [num, den] (ADR-0199)",
                )
            }

            fn visit_i64<E>(self, value: i64) -> Result<FrameRate, E>
            where
                E: de::Error,
            {
                if value <= 0 {
                    return Err(E::custom(format!(
                        "frame rate must be positive, got {value}"
                    )));
                }
                Ok(FrameRate::new(value, 1))
            }

            fn visit_u64<E>(self, value: u64) -> Result<FrameRate, E>
            where
                E: de::Error,
            {
                if value == 0 || value > i64::MAX as u64 {
                    return Err(E::custom(format!(
                        "frame rate must be a positive i64, got {value}"
                    )));
                }
                Ok(FrameRate::new(value as i64, 1))
            }

            fn visit_f64<E>(self, value: f64) -> Result<FrameRate, E>
            where
                E: de::Error,
            {
                Err(E::custom(format!(
                    "decimal floats like {value} are forbidden; write exact rational {{\"num\": \
                     N, \"den\": D}} or integer fps (ADR-0199)"
                )))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<FrameRate, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let num: i64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &"2 integers [num, den]"))?;
                let den: i64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &"2 integers [num, den]"))?;
                if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &"2 integers [num, den]"));
                }
                if num <= 0 || den <= 0 {
                    return Err(de::Error::custom(format!(
                        "frame rate numerator and denominator must be positive, got [{num}, {den}]"
                    )));
                }
                Ok(FrameRate::new(num, den))
            }

            fn visit_map<M>(self, mut map: M) -> Result<FrameRate, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut num: Option<i64> = None;
                let mut den: Option<i64> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "num" => {
                            if num.is_some() {
                                return Err(de::Error::duplicate_field("num"));
                            }
                            num = Some(map.next_value()?);
                        }
                        "den" => {
                            if den.is_some() {
                                return Err(de::Error::duplicate_field("den"));
                            }
                            den = Some(map.next_value()?);
                        }
                        other => {
                            return Err(de::Error::unknown_field(other, &["num", "den"]));
                        }
                    }
                }

                let num = num.ok_or_else(|| de::Error::missing_field("num"))?;
                let den = den.ok_or_else(|| de::Error::missing_field("den"))?;
                if num <= 0 || den <= 0 {
                    return Err(de::Error::custom(format!(
                        "frame rate numerator and denominator must be positive, got {{\"num\": \
                         {num}, \"den\": {den}}}"
                    )));
                }
                Ok(FrameRate::new(num, den))
            }
        }

        deserializer.deserialize_any(FrameRateVisitor)
    }
}

impl JsonSchema for FrameRate {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FrameRate".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::Schema::try_from(serde_json::json!({
            "description": "Sequence frame rate in frames per second. Whole integer (24, 25, 30, \
                            50, 60), exact rational object ({\"num\": 24000, \"den\": 1001}), or \
                            exact rational pair ([24000, 1001]). Decimal floats are strictly \
                            forbidden (ADR-0199).",
            "oneOf": [
                {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Whole frame rate in frames per second."
                },
                {
                    "type": "object",
                    "properties": {
                        "num": { "type": "integer", "minimum": 1 },
                        "den": { "type": "integer", "minimum": 1 }
                    },
                    "required": ["num", "den"],
                    "additionalProperties": false,
                    "description": "Exact rational frame rate (e.g. {\"num\": 24000, \"den\": \
                                    1001} for 23.976 fps)."
                },
                {
                    "type": "array",
                    "items": { "type": "integer", "minimum": 1 },
                    "minItems": 2,
                    "maxItems": 2,
                    "description": "Exact rational pair [num, den]."
                }
            ]
        }))
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_frame_rate_roundtrips() {
        let rate: FrameRate = serde_json::from_str("25").unwrap();
        assert_eq!(rate, 25);
        assert_eq!(rate.num(), 25);
        assert_eq!(rate.den(), 1);
        assert!(rate.is_integer());
        assert_eq!(serde_json::to_string(&rate).unwrap(), "25");
    }

    #[test]
    fn rational_object_roundtrips() {
        let rate: FrameRate = serde_json::from_str(r#"{"num": 24000, "den": 1001}"#).unwrap();
        assert_eq!(rate.num(), 24000);
        assert_eq!(rate.den(), 1001);
        assert!(!rate.is_integer());
        assert_eq!(
            serde_json::to_string(&rate).unwrap(),
            r#"{"num":24000,"den":1001}"#
        );
    }

    #[test]
    fn rational_pair_deserializes() {
        let rate: FrameRate = serde_json::from_str("[24000, 1001]").unwrap();
        assert_eq!(rate.num(), 24000);
        assert_eq!(rate.den(), 1001);
        // Serializes canonically as object
        assert_eq!(
            serde_json::to_string(&rate).unwrap(),
            r#"{"num":24000,"den":1001}"#
        );
    }

    #[test]
    fn reducible_fractions_are_normalized() {
        let rate = FrameRate::new(60000, 2002);
        assert_eq!(rate.num(), 30000);
        assert_eq!(rate.den(), 1001);
    }

    #[test]
    fn decimal_float_is_strictly_rejected() {
        let err = serde_json::from_str::<FrameRate>("23.976").unwrap_err();
        assert!(
            err.to_string()
                .contains("decimal floats like 23.976 are forbidden"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn zero_and_negative_are_rejected() {
        assert!(serde_json::from_str::<FrameRate>("0").is_err());
        assert!(serde_json::from_str::<FrameRate>("-25").is_err());
        assert!(serde_json::from_str::<FrameRate>(r#"{"num": 0, "den": 1}"#).is_err());
        assert!(serde_json::from_str::<FrameRate>(r#"{"num": 25, "den": -1}"#).is_err());
        assert!(serde_json::from_str::<FrameRate>("[0, 1]").is_err());
    }
}
