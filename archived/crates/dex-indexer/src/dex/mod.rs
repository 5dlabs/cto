pub mod jupiter;
pub mod lifinity;
pub mod meteora;
pub mod openbook;
pub mod orca;
pub mod phoenix;
pub mod pumpfun;
pub mod raydium;

use std::collections::HashMap;

/// Identifies DEX programs by their on-chain program ID.
#[derive(Debug, Clone)]
pub struct DexRegistry {
    programs: HashMap<String, DexProgram>,
}

#[derive(Debug, Clone)]
pub struct DexProgram {
    pub name: &'static str,
    pub label: &'static str,
}

impl DexRegistry {
    pub fn new() -> Self {
        let mut programs = HashMap::new();

        for (id, label) in raydium::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "raydium",
                    label,
                },
            );
        }
        for (id, label) in orca::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "orca",
                    label,
                },
            );
        }
        for (id, label) in meteora::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "meteora",
                    label,
                },
            );
        }
        for (id, label) in pumpfun::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "pumpfun",
                    label,
                },
            );
        }
        for (id, label) in lifinity::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "lifinity",
                    label,
                },
            );
        }
        for (id, label) in phoenix::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "phoenix",
                    label,
                },
            );
        }
        for (id, label) in openbook::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "openbook",
                    label,
                },
            );
        }
        for (id, label) in jupiter::program_ids() {
            programs.insert(
                id.to_string(),
                DexProgram {
                    name: "jupiter",
                    label,
                },
            );
        }

        tracing::info!(count = programs.len(), "DEX registry initialised");
        Self { programs }
    }

    /// Return the DEX program matching any of the given account keys, or `None`.
    pub fn identify(&self, account_keys: &[String]) -> Option<&DexProgram> {
        // Prefer non-aggregator (Raydium/Orca/Meteora) over Jupiter so we
        // attribute swaps to the underlying pool when Jupiter is the router.
        let mut found: Option<&DexProgram> = None;
        for key in account_keys {
            if let Some(dex) = self.programs.get(key) {
                if dex.name != "jupiter" {
                    return Some(dex);
                }
                if found.is_none() {
                    found = Some(dex);
                }
            }
        }
        found
    }

    /// All registered program IDs (for the gRPC subscription filter).
    pub fn all_program_ids(&self) -> Vec<String> {
        self.programs.keys().cloned().collect()
    }
}

impl Default for DexRegistry {
    fn default() -> Self {
        Self::new()
    }
}
