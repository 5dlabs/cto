//! Skill learning pipeline.
//!
//! This module provides:
//! - Session collection from Healer
//! - Task extraction and analysis
//! - Complexity filtering
//! - SOP extraction and skill learning

mod collector;
mod complexity;
mod extractor;
mod learner;

pub use collector::SessionCollector;
pub use complexity::ComplexityFilter;
pub use extractor::TaskExtractor;
pub use learner::SkillLearner;
