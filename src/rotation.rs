use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum SmearorRotation {
    #[default]
    #[serde(rename = "0", alias = "deg0", alias = "Deg0")]
    Deg0,
    #[serde(rename = "90", alias = "deg90", alias = "Deg90")]
    Deg90,
    #[serde(rename = "180", alias = "deg180", alias = "Deg180")]
    Deg180,
    #[serde(rename = "270", alias = "deg270", alias = "Deg270")]
    Deg270,
    Deg(f32),
}

impl SmearorRotation {
    pub fn new(degrees: f32) -> Self {
        match degrees {
            0.0 => Self::Deg0,
            90.0 => Self::Deg90,
            180.0 => Self::Deg180,
            270.0 => Self::Deg270,
            _ => Self::Deg(degrees),
        }
    }
    pub fn to_degrees(self) -> f32 {
        match self {
            Self::Deg0 => 0.0,
            Self::Deg90 => 90.0,
            Self::Deg180 => 180.0,
            Self::Deg270 => 270.0,
            Self::Deg(deg) => deg,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        let degrees = self.to_degrees();
        (degrees - 0.0).abs() < 0.1 || (degrees - 180.0).abs() < 0.1
    }

    pub fn is_vertical(&self) -> bool {
        let degrees = self.to_degrees();
        (degrees - 90.0).abs() < 0.1 || (degrees - 270.0).abs() < 0.1
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor(&self) -> Option<gtk4_layer_shell::Edge> {
        use gtk4_layer_shell::Edge;
        Some(match self {
            Self::Deg0 => Edge::Bottom,
            Self::Deg90 => Edge::Left,
            Self::Deg180 => Edge::Top,
            Self::Deg270 => Edge::Right,
            Self::Deg(_) => return None,
        })
    }
}

impl From<&str> for SmearorRotation {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "0" => Self::Deg0,
            "90" => Self::Deg90,
            "180" => Self::Deg180,
            "270" => Self::Deg270,
            s => {
                let clean = s.replace("deg", "").replace("Deg", "");
                clean.parse::<f32>().map(Self::Deg).unwrap_or(Self::Deg0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smearor_rotation_deg0() {
        let rotation = SmearorRotation::Deg0;
        assert_eq!(rotation.to_degrees(), 0.0);
    }

    #[test]
    fn test_smearor_rotation_deg90() {
        let rotation = SmearorRotation::Deg90;
        assert_eq!(rotation.to_degrees(), 90.0);
    }

    #[test]
    fn test_smearor_rotation_deg180() {
        let rotation = SmearorRotation::Deg180;
        assert_eq!(rotation.to_degrees(), 180.0);
    }

    #[test]
    fn test_smearor_rotation_deg270() {
        let rotation = SmearorRotation::Deg270;
        assert_eq!(rotation.to_degrees(), 270.0);
    }

    #[test]
    fn test_smearor_rotation_custom() {
        let rotation = SmearorRotation::Deg(45.0);
        assert_eq!(rotation.to_degrees(), 45.0);
    }

    #[test]
    fn test_smearor_rotation_from_str() {
        let rotation = SmearorRotation::from("90");
        assert_eq!(rotation.to_degrees(), 90.0);
    }

    #[test]
    fn test_smearor_rotation_from_str_deg() {
        let rotation = SmearorRotation::from("deg180");
        assert_eq!(rotation.to_degrees(), 180.0);
    }

    #[test]
    fn test_smearor_rotation_from_str_custom() {
        let rotation = SmearorRotation::from("45");
        assert_eq!(rotation.to_degrees(), 45.0);
    }
}
