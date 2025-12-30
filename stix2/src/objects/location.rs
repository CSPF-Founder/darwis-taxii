//! Location SDO (STIX 2.1)
//!
//! A Location represents a geographic location.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_properties_dependency};
use crate::vocab::Region;
use serde::{Deserialize, Serialize};

/// Location STIX Domain Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(flatten)]
    pub common: CommonProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub administrative_area: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
}

impl Location {
    pub const TYPE: &'static str = "location";

    pub fn builder() -> LocationBuilder {
        LocationBuilder::new()
    }
}

impl_sdo_traits!(Location, "location");

impl Constrained for Location {
    /// Validate Location constraints.
    ///
    /// - If `precision` is set, `latitude` and `longitude` must also be set
    /// - `latitude` must be between -90 and 90
    /// - `longitude` must be between -180 and 180
    /// - `precision` must be >= 0
    /// - `latitude` and `longitude` must both be present or both absent
    fn validate_constraints(&self) -> Result<()> {
        // Validate latitude range (-90 to 90)
        if let Some(lat) = self.latitude
            && !(-90.0..=90.0).contains(&lat)
        {
            return Err(Error::InvalidPropertyValue {
                property: "latitude".to_string(),
                message: "latitude must be between -90 and 90".to_string(),
            });
        }

        // Validate longitude range (-180 to 180)
        if let Some(lon) = self.longitude
            && !(-180.0..=180.0).contains(&lon)
        {
            return Err(Error::InvalidPropertyValue {
                property: "longitude".to_string(),
                message: "longitude must be between -180 and 180".to_string(),
            });
        }

        // Validate precision >= 0
        if let Some(prec) = self.precision
            && prec < 0.0
        {
            return Err(Error::InvalidPropertyValue {
                property: "precision".to_string(),
                message: "precision must be >= 0".to_string(),
            });
        }

        // latitude and longitude must both be present or both absent
        match (&self.latitude, &self.longitude) {
            (Some(_), None) => {
                return Err(Error::PropertyDependency {
                    dependent: "latitude".to_string(),
                    dependency: "longitude".to_string(),
                });
            }
            (None, Some(_)) => {
                return Err(Error::PropertyDependency {
                    dependent: "longitude".to_string(),
                    dependency: "latitude".to_string(),
                });
            }
            _ => {}
        }

        // precision requires latitude and longitude
        check_properties_dependency(
            &["latitude", "longitude"],
            &["precision"],
            |prop| match prop {
                "latitude" => self.latitude.is_some(),
                "longitude" => self.longitude.is_some(),
                "precision" => self.precision.is_some(),
                _ => false,
            },
        )
    }
}

#[derive(Debug, Default)]
pub struct LocationBuilder {
    name: Option<String>,
    description: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    precision: Option<f64>,
    region: Option<Region>,
    country: Option<String>,
    administrative_area: Option<String>,
    city: Option<String>,
    street_address: Option<String>,
    postal_code: Option<String>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(LocationBuilder);

impl LocationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn coordinates(mut self, latitude: f64, longitude: f64) -> Self {
        self.latitude = Some(latitude);
        self.longitude = Some(longitude);
        self
    }

    pub fn precision(mut self, precision: f64) -> Self {
        self.precision = Some(precision);
        self
    }

    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    pub fn administrative_area(mut self, area: impl Into<String>) -> Self {
        self.administrative_area = Some(area.into());
        self
    }

    pub fn city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    pub fn street_address(mut self, address: impl Into<String>) -> Self {
        self.street_address = Some(address.into());
        self
    }

    pub fn postal_code(mut self, postal_code: impl Into<String>) -> Self {
        self.postal_code = Some(postal_code.into());
        self
    }

    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    pub fn build(self) -> Result<Location> {
        // At least one of region, country, or lat/long must be present
        if self.region.is_none()
            && self.country.is_none()
            && (self.latitude.is_none() || self.longitude.is_none())
        {
            return Err(Error::AtLeastOneRequired(vec![
                "region".to_string(),
                "country".to_string(),
                "latitude/longitude".to_string(),
            ]));
        }

        let location = Location {
            type_: Location::TYPE.to_string(),
            id: Identifier::new(Location::TYPE)?,
            common: self.common,
            name: self.name,
            description: self.description,
            latitude: self.latitude,
            longitude: self.longitude,
            precision: self.precision,
            region: self.region,
            country: self.country,
            administrative_area: self.administrative_area,
            city: self.city,
            street_address: self.street_address,
            postal_code: self.postal_code,
        };

        // Validate constraints
        location.validate_constraints()?;

        Ok(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_location() {
        let loc = Location::builder()
            .name("Moscow")
            .country("RU")
            .city("Moscow")
            .build()
            .unwrap();

        assert_eq!(loc.type_, "location");
        assert_eq!(loc.country, Some("RU".to_string()));
    }

    #[test]
    fn test_location_with_coordinates() {
        let loc = Location::builder()
            .name("Target Location")
            .coordinates(55.7558, 37.6173)
            .build()
            .unwrap();

        assert_eq!(loc.latitude, Some(55.7558));
        assert_eq!(loc.longitude, Some(37.6173));
    }
}
