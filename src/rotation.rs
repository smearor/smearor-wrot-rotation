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
