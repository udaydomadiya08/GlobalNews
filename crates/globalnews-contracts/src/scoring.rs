use crate::error::ContractError;
use serde::{Deserialize, Serialize};

/// Represents real-world event significance decoupled from media coverage volume.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImportanceScore {
    pub final_score: f64,      // Range: 1.0 to 10.0
    pub domain_severity: f64,  // IPTC domain base severity (1.0 to 10.0)
    pub geographic_reach: f64, // Local (1.0) to Planetary (10.0)
    pub physical_impact: f64,  // Quantifiable casualties, economic loss, scale (0.0 to 10.0)
    pub volume_dampener: f64,  // Logarithmic source count dampener (1.0 to 3.0)
}

impl ImportanceScore {
    pub fn new(
        domain_severity: f64,
        geographic_reach: f64,
        physical_impact: f64,
        source_count: u32,
    ) -> Result<Self, ContractError> {
        let volume_dampener = 1.0 + 0.15 * (1.0 + source_count as f64).log2();
        let raw_score = (0.4 * domain_severity + 0.3 * geographic_reach + 0.3 * physical_impact)
            * (volume_dampener / 1.5);
        let final_score = raw_score.clamp(1.0, 10.0);

        let score = Self {
            final_score,
            domain_severity,
            geographic_reach,
            physical_impact,
            volume_dampener,
        };
        score.validate()?;
        Ok(score)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if !(1.0..=10.0).contains(&self.final_score) {
            return Err(ContractError::InvalidScoreRange(
                self.final_score,
                1.0,
                10.0,
            ));
        }
        if !(1.0..=10.0).contains(&self.domain_severity) {
            return Err(ContractError::InvalidScoreRange(
                self.domain_severity,
                1.0,
                10.0,
            ));
        }
        if !(1.0..=10.0).contains(&self.geographic_reach) {
            return Err(ContractError::InvalidScoreRange(
                self.geographic_reach,
                1.0,
                10.0,
            ));
        }
        if !(0.0..=10.0).contains(&self.physical_impact) {
            return Err(ContractError::InvalidScoreRange(
                self.physical_impact,
                0.0,
                10.0,
            ));
        }
        if self.volume_dampener < 1.0 {
            return Err(ContractError::ValidationError {
                field: "volume_dampener",
                message: "Volume dampener cannot be less than 1.0".into(),
            });
        }
        Ok(())
    }
}

/// Quantifies evidence strength, source diversity, and jurisdictional corroboration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ConfidenceScore {
    pub final_confidence: f64, // Range: 0.1 to 1.0
    pub independent_source_count: u32,
    pub jurisdictional_diversity: f64, // 0.0 to 1.0 (cross-border corroboration)
    pub has_sensor_validation: bool,   // Ground truth from physical sensors (USGS/GDACS)
    pub is_disputed: bool,             // Conflicting reports detected
}

impl ConfidenceScore {
    pub fn new(
        independent_source_count: u32,
        jurisdictional_diversity: f64,
        has_sensor_validation: bool,
        is_disputed: bool,
    ) -> Result<Self, ContractError> {
        let mut base = if has_sensor_validation { 0.85 } else { 0.25 };
        base += (independent_source_count.min(10) as f64) * 0.05;
        base += jurisdictional_diversity.clamp(0.0, 1.0) * 0.15;
        if is_disputed {
            base *= 0.6; // Downgrade confidence when reports conflict
        }
        let final_confidence = base.clamp(0.1, 1.0);

        let conf = Self {
            final_confidence,
            independent_source_count,
            jurisdictional_diversity,
            has_sensor_validation,
            is_disputed,
        };
        conf.validate()?;
        Ok(conf)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if !(0.1..=1.0).contains(&self.final_confidence) {
            return Err(ContractError::InvalidScoreRange(
                self.final_confidence,
                0.1,
                1.0,
            ));
        }
        if !(0.0..=1.0).contains(&self.jurisdictional_diversity) {
            return Err(ContractError::InvalidScoreRange(
                self.jurisdictional_diversity,
                0.0,
                1.0,
            ));
        }
        Ok(())
    }
}
