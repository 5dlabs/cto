//! Inventory management for bare metal providers.
//!
//! This module provides intelligent region selection based on actual
//! stock availability, helping avoid failed provisioning due to
//! out-of-stock conditions.

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::providers::latitude::Latitude;

/// Stock level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StockLevel {
    /// No stock available.
    Unavailable,
    /// Very limited stock (1-2 servers).
    Low,
    /// Moderate stock available.
    Medium,
    /// High stock availability.
    High,
}

impl StockLevel {
    /// Parse stock level from Latitude API string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            _ => Self::Unavailable,
        }
    }

    /// Check if this stock level is considered available.
    #[must_use]
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}

/// Region availability information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAvailability {
    /// Region slug (e.g., "ASH", "LAX", "DAL").
    pub region: String,
    /// Stock level for this region.
    pub stock_level: StockLevel,
    /// Hourly price in USD.
    pub hourly_price: f64,
    /// Country/region name.
    pub country: String,
}

/// Plan availability across all regions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAvailability {
    /// Plan slug (e.g., "c2-small-x86").
    pub plan: String,
    /// Plan display name.
    pub name: String,
    /// Available regions with stock info.
    pub regions: Vec<RegionAvailability>,
}

impl PlanAvailability {
    /// Get the best region for this plan based on stock level.
    ///
    /// Prioritizes by stock level (High > Medium > Low), then by price.
    #[must_use]
    pub fn best_region(&self) -> Option<&RegionAvailability> {
        self.regions
            .iter()
            .filter(|r| r.stock_level.is_available())
            .max_by(|a, b| {
                // First compare by stock level (higher is better)
                match a.stock_level.cmp(&b.stock_level) {
                    std::cmp::Ordering::Equal => {
                        // Then by price (lower is better), reversed for max_by
                        b.hourly_price
                            .partial_cmp(&a.hourly_price)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }
                    other => other,
                }
            })
    }

    /// Get the best region from a preferred list.
    ///
    /// Returns the first preferred region that has stock, or falls back to `best_region`.
    #[must_use]
    pub fn best_region_from_preferred(&self, preferred: &[&str]) -> Option<&RegionAvailability> {
        // First try preferred regions in order
        for pref in preferred {
            if let Some(region) = self
                .regions
                .iter()
                .find(|r| r.region.eq_ignore_ascii_case(pref) && r.stock_level.is_available())
            {
                return Some(region);
            }
        }
        // Fall back to best available
        self.best_region()
    }

    /// Get all regions with stock, sorted by preference.
    #[must_use]
    pub fn available_regions(&self) -> Vec<&RegionAvailability> {
        let mut regions: Vec<_> = self
            .regions
            .iter()
            .filter(|r| r.stock_level.is_available())
            .collect();

        // Sort by stock level (descending), then price (ascending)
        regions.sort_by(|a, b| match b.stock_level.cmp(&a.stock_level) {
            std::cmp::Ordering::Equal => a
                .hourly_price
                .partial_cmp(&b.hourly_price)
                .unwrap_or(std::cmp::Ordering::Equal),
            other => other,
        });

        regions
    }
}

/// Inventory manager for querying and selecting regions.
pub struct InventoryManager {
    provider: Latitude,
    /// Cached plan availability (`plan_slug` -> availability).
    cache: HashMap<String, PlanAvailability>,
}

impl InventoryManager {
    /// Create a new inventory manager.
    #[must_use]
    pub fn new(provider: Latitude) -> Self {
        Self {
            provider,
            cache: HashMap::new(),
        }
    }

    /// Get availability for a specific plan.
    ///
    /// # Errors
    ///
    /// Returns an error if the API call fails or the plan is not found.
    ///
    /// # Panics
    ///
    /// This function will not panic in normal operation. The internal
    /// `expect` is guarded by a `contains_key` check and insert.
    pub async fn get_plan_availability(&mut self, plan_slug: &str) -> Result<&PlanAvailability> {
        // Check if we need to fetch
        if !self.cache.contains_key(plan_slug) {
            let availability = self.fetch_plan_availability(plan_slug).await?;
            self.cache.insert(plan_slug.to_string(), availability);
        }

        // Safe because we just inserted if it wasn't there
        Ok(self.cache.get(plan_slug).expect("just inserted"))
    }

    /// Fetch plan availability from the API.
    async fn fetch_plan_availability(&self, plan_slug: &str) -> Result<PlanAvailability> {
        // Fetch from API
        let plans = self
            .provider
            .list_plans()
            .await
            .context("Failed to list plans")?;

        let plan = plans
            .iter()
            .find(|p| p.attributes.slug.as_ref().is_some_and(|s| s == plan_slug))
            .with_context(|| format!("Plan not found: {plan_slug}"))?;

        let mut regions = Vec::new();

        // Parse regions from the plan
        if let Some(ref plan_regions) = plan.attributes.regions {
            for region in plan_regions {
                let region_name = region.name.clone().unwrap_or_default();
                let stock_level = region
                    .stock_level
                    .as_ref()
                    .map_or(StockLevel::Unavailable, |s| StockLevel::parse(s));

                let hourly_price = region
                    .pricing
                    .as_ref()
                    .and_then(|p| p.usd.as_ref())
                    .and_then(|p| p.hour)
                    .unwrap_or(0.0);

                // Get in-stock locations for this region
                if let Some(ref locations) = region.locations {
                    if let Some(ref in_stock) = locations.in_stock {
                        for site in in_stock {
                            regions.push(RegionAvailability {
                                region: site.clone(),
                                stock_level,
                                hourly_price,
                                country: region_name.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(PlanAvailability {
            plan: plan.attributes.slug.clone().unwrap_or_default(),
            name: plan.attributes.name.clone().unwrap_or_default(),
            regions,
        })
    }

    /// Find the best region for a plan.
    ///
    /// # Errors
    ///
    /// Returns an error if no regions have stock for this plan.
    pub async fn find_best_region(&mut self, plan_slug: &str) -> Result<String> {
        let availability = self.get_plan_availability(plan_slug).await?;

        let best = availability
            .best_region()
            .with_context(|| format!("No regions have stock for plan: {plan_slug}"))?;

        info!(
            "Best region for {}: {} ({:?} stock, ${:.2}/hr)",
            plan_slug, best.region, best.stock_level, best.hourly_price
        );

        Ok(best.region.clone())
    }

    /// Find the best region from a list of preferred regions.
    ///
    /// # Errors
    ///
    /// Returns an error if no preferred regions have stock and fallback is disabled.
    pub async fn find_best_region_from_preferred(
        &mut self,
        plan_slug: &str,
        preferred: &[&str],
        allow_fallback: bool,
    ) -> Result<String> {
        let availability = self.get_plan_availability(plan_slug).await?;

        // Try preferred regions first
        for pref in preferred {
            if let Some(region) = availability
                .regions
                .iter()
                .find(|r| r.region.eq_ignore_ascii_case(pref) && r.stock_level.is_available())
            {
                info!(
                    "Found preferred region {} for {} ({:?} stock)",
                    region.region, plan_slug, region.stock_level
                );
                return Ok(region.region.clone());
            }
            debug!("Preferred region {} not available for {}", pref, plan_slug);
        }

        // Fallback to best available
        if allow_fallback {
            warn!(
                "No preferred regions available for {}, falling back to best available",
                plan_slug
            );
            let best = availability
                .best_region()
                .with_context(|| format!("No regions have stock for plan: {plan_slug}"))?;

            info!(
                "Fallback region for {}: {} ({:?} stock, ${:.2}/hr)",
                plan_slug, best.region, best.stock_level, best.hourly_price
            );

            Ok(best.region.clone())
        } else {
            anyhow::bail!(
                "No preferred regions ({}) have stock for plan: {}",
                preferred.join(", "),
                plan_slug
            );
        }
    }

    /// List all available regions for a plan.
    ///
    /// # Errors
    ///
    /// Returns an error if the API call fails.
    pub async fn list_available_regions(&mut self, plan_slug: &str) -> Result<Vec<String>> {
        let availability = self.get_plan_availability(plan_slug).await?;

        let regions: Vec<String> = availability
            .available_regions()
            .iter()
            .map(|r| r.region.clone())
            .collect();

        Ok(regions)
    }

    /// Validate that a region has stock for a plan.
    ///
    /// # Errors
    ///
    /// Returns an error if the region doesn't have stock.
    pub async fn validate_region(&mut self, plan_slug: &str, region: &str) -> Result<StockLevel> {
        let availability = self.get_plan_availability(plan_slug).await?;

        let region_info = availability
            .regions
            .iter()
            .find(|r| r.region.eq_ignore_ascii_case(region))
            .with_context(|| {
                format!(
                    "Region {} not available for plan {}. Available: {:?}",
                    region,
                    plan_slug,
                    availability
                        .available_regions()
                        .iter()
                        .map(|r| &r.region)
                        .collect::<Vec<_>>()
                )
            })?;

        if !region_info.stock_level.is_available() {
            anyhow::bail!("Region {region} is out of stock for plan {plan_slug}");
        }

        Ok(region_info.stock_level)
    }

    /// Clear the cache to force fresh data on next query.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Region selection strategy for cluster provisioning.
#[derive(Debug, Clone)]
pub enum RegionStrategy {
    /// Use a specific region (fail if unavailable).
    Fixed(String),
    /// Use a specific region, fall back to best available.
    PreferredWithFallback(String),
    /// Choose from a list of preferred regions, fail if none available.
    PreferredList(Vec<String>),
    /// Choose from a list, fall back to any available region.
    PreferredListWithFallback(Vec<String>),
    /// Automatically choose the best available region.
    Auto,
}

impl RegionStrategy {
    /// Select a region based on this strategy.
    ///
    /// # Errors
    ///
    /// Returns an error if no suitable region is available.
    pub async fn select(&self, manager: &mut InventoryManager, plan: &str) -> Result<String> {
        match self {
            Self::Fixed(region) => {
                manager.validate_region(plan, region).await?;
                Ok(region.clone())
            }
            Self::PreferredWithFallback(region) => {
                manager
                    .find_best_region_from_preferred(plan, &[region.as_str()], true)
                    .await
            }
            Self::PreferredList(regions) => {
                let refs: Vec<&str> = regions.iter().map(String::as_str).collect();
                manager
                    .find_best_region_from_preferred(plan, &refs, false)
                    .await
            }
            Self::PreferredListWithFallback(regions) => {
                let refs: Vec<&str> = regions.iter().map(String::as_str).collect();
                manager
                    .find_best_region_from_preferred(plan, &refs, true)
                    .await
            }
            Self::Auto => manager.find_best_region(plan).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_level_ordering() {
        assert!(StockLevel::High > StockLevel::Medium);
        assert!(StockLevel::Medium > StockLevel::Low);
        assert!(StockLevel::Low > StockLevel::Unavailable);
    }

    #[test]
    fn test_stock_level_parse() {
        assert_eq!(StockLevel::parse("high"), StockLevel::High);
        assert_eq!(StockLevel::parse("HIGH"), StockLevel::High);
        assert_eq!(StockLevel::parse("medium"), StockLevel::Medium);
        assert_eq!(StockLevel::parse("low"), StockLevel::Low);
        assert_eq!(StockLevel::parse("unknown"), StockLevel::Unavailable);
    }

    #[test]
    fn test_stock_level_availability() {
        assert!(StockLevel::High.is_available());
        assert!(StockLevel::Medium.is_available());
        assert!(StockLevel::Low.is_available());
        assert!(!StockLevel::Unavailable.is_available());
    }

    #[test]
    fn test_plan_availability_best_region() {
        let availability = PlanAvailability {
            plan: "c2-small-x86".to_string(),
            name: "c2.small.x86".to_string(),
            regions: vec![
                RegionAvailability {
                    region: "LAX".to_string(),
                    stock_level: StockLevel::High,
                    hourly_price: 0.18,
                    country: "United States".to_string(),
                },
                RegionAvailability {
                    region: "ASH".to_string(),
                    stock_level: StockLevel::Medium,
                    hourly_price: 0.18,
                    country: "United States".to_string(),
                },
                RegionAvailability {
                    region: "SAO".to_string(),
                    stock_level: StockLevel::Low,
                    hourly_price: 0.15,
                    country: "Brazil".to_string(),
                },
            ],
        };

        let best = availability.best_region().unwrap();
        assert_eq!(best.region, "LAX"); // High stock wins
    }

    #[test]
    fn test_plan_availability_preferred_region() {
        let availability = PlanAvailability {
            plan: "c2-small-x86".to_string(),
            name: "c2.small.x86".to_string(),
            regions: vec![
                RegionAvailability {
                    region: "LAX".to_string(),
                    stock_level: StockLevel::High,
                    hourly_price: 0.18,
                    country: "United States".to_string(),
                },
                RegionAvailability {
                    region: "ASH".to_string(),
                    stock_level: StockLevel::Medium,
                    hourly_price: 0.18,
                    country: "United States".to_string(),
                },
            ],
        };

        // Prefer ASH even though LAX has higher stock
        let best = availability
            .best_region_from_preferred(&["ASH", "DAL"])
            .unwrap();
        assert_eq!(best.region, "ASH");

        // Fall back to best if preferred not available
        let best = availability
            .best_region_from_preferred(&["DAL", "CHI"])
            .unwrap();
        assert_eq!(best.region, "LAX");
    }
}
