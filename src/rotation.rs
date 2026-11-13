use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use serde::de::Visitor;
use smearor_commons_geometry::QuarterTurn;
use smearor_commons_geometry::Rotation;
use smearor_commons_geometry::RotationRad;
use std::fmt;

/// Domain rotation for the rotation widget and its consumers - a
/// [`Rotation`] (snapped quarter turn or free angle) carrying the
/// widget's configuration semantics: f32 degree accessors for the
/// GTK boundary, the `"0"`/`"90"`/bare-float wire format and
/// snappedness-aware helpers like [`Self::anchor`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct SmearorRotation(pub Rotation);

impl SmearorRotation {
    /// No rotation (snapped `0` degrees).
    pub const DEG_0: Self = Self(Rotation::QuarterTurn(QuarterTurn::Deg0));
    /// Snapped `90` degrees.
    pub const DEG_90: Self = Self(Rotation::QuarterTurn(QuarterTurn::Deg90));
    /// Snapped `180` degrees.
    pub const DEG_180: Self = Self(Rotation::QuarterTurn(QuarterTurn::Deg180));
    /// Snapped `270` degrees.
    pub const DEG_270: Self = Self(Rotation::QuarterTurn(QuarterTurn::Deg270));

    /// Creates a rotation from a degree value, snapping to the
    /// quarter-turn variants for exactly `0`, `90`, `180` or `270`.
    pub fn new(degrees: impl Into<f64>) -> Self {
        Self(Rotation::from_degrees(degrees))
    }

    /// Creates a free (non-snapped) rotation - a degree value of `90.0`
    /// stays free and does not become [`Self::DEG_90`].
    pub fn free(degrees: impl Into<f64>) -> Self {
        Self(Rotation::Free(RotationRad::from_degrees(degrees)))
    }

    /// The rotation angle in degrees for the GTK boundary.
    pub fn to_degrees(self) -> f32 {
        self.0.degrees() as f32
    }

    /// Whether the rotation is within `0.1` degrees of `0` or `180`.
    pub fn is_horizontal(&self) -> bool {
        let degrees = self.to_degrees();
        (degrees - 0.0).abs() < 0.1 || (degrees - 180.0).abs() < 0.1
    }

    /// Whether the rotation is within `0.1` degrees of `90` or `270`.
    pub fn is_vertical(&self) -> bool {
        let degrees = self.to_degrees();
        (degrees - 90.0).abs() < 0.1 || (degrees - 270.0).abs() < 0.1
    }

    /// The screen edge the bottom edge of the rotated content faces,
    /// for snapped rotations; `None` for free rotations.
    #[cfg(feature = "layer-shell")]
    pub fn anchor(&self) -> Option<gtk4_layer_shell::Edge> {
        use gtk4_layer_shell::Edge;
        Some(match self.0.quarter_turn()? {
            QuarterTurn::Deg0 => Edge::Bottom,
            QuarterTurn::Deg90 => Edge::Left,
            QuarterTurn::Deg180 => Edge::Top,
            QuarterTurn::Deg270 => Edge::Right,
        })
    }
}

impl Default for SmearorRotation {
    /// [`SmearorRotation::DEG_0`].
    fn default() -> Self {
        Self::DEG_0
    }
}

/// The rotation as a [`RotationRad`] for geometry math.
impl From<SmearorRotation> for RotationRad {
    fn from(rotation: SmearorRotation) -> Self {
        rotation.0.to_rotation_rad()
    }
}

/// A continuous angle as the quantized domain rotation (`DEG_0`..
/// `DEG_270` for the cardinal values, free otherwise).
impl From<RotationRad> for SmearorRotation {
    fn from(rotation: RotationRad) -> Self {
        Self(Rotation::from_degrees(rotation.to_degrees()))
    }
}

/// A snapped quarter turn as the domain rotation.
impl From<QuarterTurn> for SmearorRotation {
    fn from(turn: QuarterTurn) -> Self {
        Self(Rotation::QuarterTurn(turn))
    }
}

impl From<&str> for SmearorRotation {
    /// Parses `"0"`/`"90"`/`"180"`/`"270"` (optionally `deg`-prefixed)
    /// into the snapped variants and any other number into a free
    /// rotation; unparseable input becomes [`Self::DEG_0`].
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "0" => Self::DEG_0,
            "90" => Self::DEG_90,
            "180" => Self::DEG_180,
            "270" => Self::DEG_270,
            s => {
                let clean = s.replace("deg", "");
                clean.parse::<f32>().map(Self::free).unwrap_or(Self::DEG_0)
            }
        }
    }
}

impl Serialize for SmearorRotation {
    /// Snapped rotations serialize as the degree strings
    /// `"0"`/`"90"`/`"180"`/`"270"`, free rotations as bare numbers.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Rotation::QuarterTurn(turn) => serializer.serialize_str(&turn.to_string()),
            Rotation::Free(_) => serializer.serialize_f64(self.0.degrees()),
        }
    }
}

impl<'de> Deserialize<'de> for SmearorRotation {
    /// Accepts the degree strings `"0"`/`"90"`/`"180"`/`"270"`
    /// (and the `deg0`/`Deg0` aliases) or a bare number, which always
    /// deserializes as a free (non-snapped) rotation.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RotationVisitor;

        impl<'de> Visitor<'de> for RotationVisitor {
            type Value = SmearorRotation;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a degree string ('0', '90', '180', '270') or a number")
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "0" | "deg0" | "Deg0" => Ok(SmearorRotation::DEG_0),
                    "90" | "deg90" | "Deg90" => Ok(SmearorRotation::DEG_90),
                    "180" | "deg180" | "Deg180" => Ok(SmearorRotation::DEG_180),
                    "270" | "deg270" | "Deg270" => Ok(SmearorRotation::DEG_270),
                    _ => Err(E::custom(format!("invalid rotation '{value}'"))),
                }
            }

            fn visit_f64<E: Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(SmearorRotation(Rotation::Free(RotationRad::from_degrees(value))))
            }

            fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
                self.visit_f64(value as f64)
            }

            fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
                self.visit_f64(value as f64)
            }
        }

        deserializer.deserialize_any(RotationVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smearor_rotation_deg0() {
        assert_eq!(SmearorRotation::DEG_0.to_degrees(), 0.0);
    }

    #[test]
    fn test_smearor_rotation_deg90() {
        assert_eq!(SmearorRotation::DEG_90.to_degrees(), 90.0);
    }

    #[test]
    fn test_smearor_rotation_deg180() {
        assert_eq!(SmearorRotation::DEG_180.to_degrees(), 180.0);
    }

    #[test]
    fn test_smearor_rotation_deg270() {
        assert_eq!(SmearorRotation::DEG_270.to_degrees(), 270.0);
    }

    #[test]
    fn test_smearor_rotation_custom() {
        assert_eq!(SmearorRotation::free(45.0).to_degrees(), 45.0);
    }

    #[test]
    fn test_new_snaps_cardinal_values() {
        assert_eq!(SmearorRotation::new(90.0), SmearorRotation::DEG_90);
        assert_eq!(SmearorRotation::new(45.0), SmearorRotation::free(45.0));
    }

    #[test]
    fn test_smearor_rotation_from_str() {
        assert_eq!(SmearorRotation::from("90").to_degrees(), 90.0);
    }

    #[test]
    fn test_smearor_rotation_from_str_deg() {
        assert_eq!(SmearorRotation::from("deg180").to_degrees(), 180.0);
    }

    #[test]
    fn test_smearor_rotation_from_str_custom() {
        assert_eq!(SmearorRotation::from("45").to_degrees(), 45.0);
    }

    #[test]
    fn test_serde_wire_format() {
        assert_eq!(serde_json::to_string(&SmearorRotation::DEG_90).unwrap(), "\"90\"");
        assert_eq!(serde_json::to_string(&SmearorRotation::free(45.0)).unwrap(), "45.0");
        assert_eq!(serde_json::from_str::<SmearorRotation>("\"deg270\"").unwrap(), SmearorRotation::DEG_270);
        let free = serde_json::from_str::<SmearorRotation>("90.0").unwrap();
        assert!(!free.0.is_snapped());
        assert_eq!(free.to_degrees(), 90.0);
    }
}
