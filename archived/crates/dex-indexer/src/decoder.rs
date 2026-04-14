use std::collections::HashMap;

use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;
use yellowstone_grpc_proto::prelude::TokenBalance;

use crate::dex::DexRegistry;
use crate::error::Error;
use crate::SwapEvent;

#[derive(Clone)]
pub struct Decoder {
    registry: DexRegistry,
}

/// A token balance change for a single owner+mint pair.
struct TokenDelta {
    mint: String,
    owner: String,
    /// Positive = received, negative = sent.  Decimal-adjusted (ui_amount).
    amount: f64,
}

impl Decoder {
    pub fn new(registry: DexRegistry) -> Self {
        Self { registry }
    }

    /// Decode a transaction into zero or more swap events.
    ///
    /// Returns an empty vec if the transaction is not a recognisable swap.
    pub fn decode(
        &self,
        info: &SubscribeUpdateTransactionInfo,
        slot: u64,
    ) -> Result<Vec<SwapEvent>, Error> {
        let meta = info
            .meta
            .as_ref()
            .ok_or_else(|| Error::Decode("missing meta".into()))?;
        let tx = info
            .transaction
            .as_ref()
            .ok_or_else(|| Error::Decode("missing transaction".into()))?;
        let msg = tx
            .message
            .as_ref()
            .ok_or_else(|| Error::Decode("missing message".into()))?;

        // Build full account-key list: static keys + loaded addresses (v0 txs).
        let account_keys: Vec<String> = msg
            .account_keys
            .iter()
            .chain(meta.loaded_writable_addresses.iter())
            .chain(meta.loaded_readonly_addresses.iter())
            .map(|k| bs58::encode(k).into_string())
            .collect();

        // Identify DEX program.
        let dex = match self.registry.identify(&account_keys) {
            Some(d) => {
                tracing::debug!(dex = d.label, "matched DEX program");
                d
            }
            None => return Ok(Vec::new()),
        };

        // Signer is the first account key.
        let signer = account_keys
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let signature = bs58::encode(&info.signature).into_string();

        // Compute token-balance deltas.
        let mut deltas = compute_deltas(&meta.pre_token_balances, &meta.post_token_balances);

        // Include native SOL changes for the signer (not in token balances).
        // Adjust for tx fee so we only see swap-related SOL movement.
        if !meta.pre_balances.is_empty() && !meta.post_balances.is_empty() {
            let fee = meta.fee as i64;
            let sol_lamport_delta =
                (meta.post_balances[0] as i64 - meta.pre_balances[0] as i64) + fee;
            let sol_amount = sol_lamport_delta as f64 / 1_000_000_000.0;
            if sol_amount.abs() > 0.000_001 {
                deltas.push(TokenDelta {
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    owner: signer.clone(),
                    amount: sol_amount,
                });
            }
        }

        if deltas.is_empty() {
            return Ok(Vec::new());
        }

        let now_nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);

        // Group deltas by owner. Find any owner with opposing deltas (sold + bought).
        // Priority: signer first, then any other owner (captures swaps where signer
        // uses ephemeral WSOL accounts invisible to token balance tracking).
        let mut best_owner: Option<(&str, Vec<&TokenDelta>, Vec<&TokenDelta>)> = None;

        // Collect unique owners.
        let mut owners: Vec<&str> = deltas.iter().map(|d| d.owner.as_str()).collect();
        owners.sort_unstable();
        owners.dedup();

        // Check signer first, then others.
        let ordered: Vec<&str> = std::iter::once(signer.as_str())
            .chain(owners.iter().copied().filter(|o| *o != signer.as_str()))
            .collect();

        for owner in &ordered {
            let mut sold = Vec::new();
            let mut bought = Vec::new();
            for d in &deltas {
                if d.owner == *owner {
                    if d.amount < -f64::EPSILON {
                        sold.push(d);
                    } else if d.amount > f64::EPSILON {
                        bought.push(d);
                    }
                }
            }
            if !sold.is_empty() && !bought.is_empty() {
                best_owner = Some((owner, sold, bought));
                break;
            }
        }

        let (swap_owner, sold, bought) = match best_owner {
            Some(v) => v,
            None => return Ok(Vec::new()),
        };

        // Pool = first owner that isn't the swap initiator.
        let pool = deltas
            .iter()
            .find(|d| d.owner != swap_owner && d.owner != "unknown")
            .map(|d| d.owner.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let token_in_delta = sold
            .iter()
            .min_by(|a, b| {
                a.amount
                    .partial_cmp(&b.amount)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        let token_out_delta = bought
            .iter()
            .max_by(|a, b| {
                a.amount
                    .partial_cmp(&b.amount)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        let amount_in = token_in_delta.amount.abs();
        let amount_out = token_out_delta.amount;
        let price = if amount_in > f64::EPSILON {
            amount_out / amount_in
        } else {
            0.0
        };

        Ok(vec![SwapEvent {
            timestamp: now_nanos,
            slot,
            signature,
            dex: dex.label.to_string(),
            pool,
            token_in: token_in_delta.mint.clone(),
            token_out: token_out_delta.mint.clone(),
            amount_in,
            amount_out,
            price,
            signer,
        }])
    }
}

/// Diff pre/post token balances into per-owner deltas.
fn compute_deltas(pre: &[TokenBalance], post: &[TokenBalance]) -> Vec<TokenDelta> {
    // Map account_index → pre ui_amount.
    let pre_map: HashMap<u32, f64> = pre
        .iter()
        .map(|b| (b.account_index, ui_amount(b)))
        .collect();

    let post_indices: std::collections::HashSet<u32> =
        post.iter().map(|b| b.account_index).collect();

    let mut deltas = Vec::new();

    // Accounts present in post (may or may not be in pre).
    for b in post {
        let pre_amt = pre_map.get(&b.account_index).copied().unwrap_or(0.0);
        let post_amt = ui_amount(b);
        let delta = post_amt - pre_amt;
        if delta.abs() > f64::EPSILON {
            deltas.push(TokenDelta {
                mint: b.mint.clone(),
                owner: b.owner.clone(),
                amount: delta,
            });
        }
    }

    // Accounts that existed in pre but vanished in post (closed accounts).
    for b in pre {
        if !post_indices.contains(&b.account_index) {
            let amt = ui_amount(b);
            if amt.abs() > f64::EPSILON {
                deltas.push(TokenDelta {
                    mint: b.mint.clone(),
                    owner: b.owner.clone(),
                    amount: -amt,
                });
            }
        }
    }

    deltas
}

fn ui_amount(b: &TokenBalance) -> f64 {
    b.ui_token_amount
        .as_ref()
        .map(|u| u.ui_amount)
        .unwrap_or(0.0)
}
