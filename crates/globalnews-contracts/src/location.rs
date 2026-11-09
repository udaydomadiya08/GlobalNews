use crate::error::ContractError;
use serde::{Deserialize, Serialize};

/// Represents a resolved physical coordinate with explicit uncertainty modeling.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GeographicLocation {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude_m: Option<f64>,
    pub toponym_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geonames_id: Option<i64>,
    pub country_code: String, // ISO 3166-1 alpha-2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin1_code: Option<String>,
    pub uncertainty_radius_km: f64,
}

impl GeographicLocation {
    pub fn new(
        latitude: f64,
        longitude: f64,
        toponym_name: impl Into<String>,
        country_code: impl Into<String>,
        uncertainty_radius_km: f64,
    ) -> Result<Self, ContractError> {
        let loc = Self {
            latitude,
            longitude,
            altitude_m: None,
            toponym_name: toponym_name.into(),
            geonames_id: None,
            country_code: country_code.into(),
            admin1_code: None,
            uncertainty_radius_km,
        };
        loc.validate()?;
        Ok(loc)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if !(-90.0..=90.0).contains(&self.latitude) {
            return Err(ContractError::InvalidLatitude(self.latitude));
        }
        if !(-180.0..=180.0).contains(&self.longitude) {
            return Err(ContractError::InvalidLongitude(self.longitude));
        }
        if self.country_code.len() != 2
            || !self.country_code.chars().all(|c| c.is_ascii_uppercase())
        {
            return Err(ContractError::InvalidCountryCode(self.country_code.clone()));
        }
        if self.uncertainty_radius_km < 0.0 {
            return Err(ContractError::ValidationError {
                field: "uncertainty_radius_km",
                message: "Uncertainty radius must be non-negative".into(),
            });
        }
        if self.toponym_name.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "toponym_name",
                message: "Toponym name cannot be empty".into(),
            });
        }
        Ok(())
    }
}
