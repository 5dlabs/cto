use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use tokio::sync::Semaphore;

use crate::dex::DexRegistry;
use crate::error::Error;
use crate::SwapEvent;

const MAX_CONCURRENT_RPC: usize = 10;

pub struct Backfiller {
    rpc_url: String,
    http: reqwest::Client,
    registry: DexRegistry,
    semaphore: Arc<Semaphore>,
}

// ── Solana JSON-RPC response types ───────────────────────────────────────

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignatureInfo {
    signature: String,
    #[allow(dead_code)]
    slot: u64,
    #[allow(dead_code)]
    block_time: Option<i64>,
    err: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionEnvelope {
    slot: u64,
    meta: Option<TransactionMeta>,
    transaction: RpcTransaction,
    block_time: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionMeta {
    fee: u64,
    pre_balances: Vec<u64>,
    post_balances: Vec<u64>,
    pre_token_balances: Option<Vec<RpcTokenBalance>>,
    post_token_balances: Option<Vec<RpcTokenBalance>>,
    loaded_addresses: Option<LoadedAddresses>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RpcTokenBalance {
    account_index: u32,
    mint: String,
    owner: Option<String>,
    ui_token_amount: UiTokenAmount,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UiTokenAmount {
    ui_amount: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadedAddresses {
    writable: Vec<String>,
    readonly: Vec<String>,
}

#[derive(Deserialize)]
struct RpcTransaction {
    message: RpcMessage,
    signatures: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RpcMessage {
    account_keys: Vec<String>,
}

struct TokenDelta {
    mint: String,
    owner: String,
    amount: f64,
}

impl Backfiller {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("http client"),
            registry: DexRegistry::new(),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_RPC)),
        }
    }

    /// Walk backward from the most recent signatures for a given program.
    /// Yields decoded swap events.
    pub async fn walk_program(
        &self,
        program_id: &str,
        stop_before_nanos: Option<i64>,
    ) -> Result<Vec<SwapEvent>, Error> {
        let mut all_swaps = Vec::new();
        let mut before_sig: Option<String> = None;

        loop {
            let sigs = self
                .get_signatures(program_id, before_sig.as_deref(), 1000)
                .await?;

            if sigs.is_empty() {
                tracing::info!(program_id, "no more signatures, backfill complete");
                break;
            }

            tracing::info!(
                program_id,
                batch_size = sigs.len(),
                first = %sigs[0].signature,
                "fetched signature batch"
            );

            // Process in parallel with semaphore rate limiting.
            let mut handles = Vec::new();
            for sig_info in &sigs {
                if sig_info.err.is_some() {
                    continue; // skip failed transactions
                }
                let sig = sig_info.signature.clone();
                let sem = Arc::clone(&self.semaphore);
                let http = self.http.clone();
                let rpc_url = self.rpc_url.clone();
                let registry = self.registry.clone();

                handles.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.expect("semaphore open");
                    fetch_and_decode(&http, &rpc_url, &sig, &registry).await
                }));
            }

            for handle in handles {
                match handle.await {
                    Ok(Ok(swaps)) => {
                        for swap in swaps {
                            // Check stop condition.
                            if let Some(stop_ts) = stop_before_nanos {
                                if swap.timestamp <= stop_ts {
                                    tracing::info!("reached existing data boundary, stopping");
                                    return Ok(all_swaps);
                                }
                            }
                            all_swaps.push(swap);
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::debug!(error = %e, "decode error in backfill");
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "backfill task panicked");
                    }
                }
            }

            // Cursor for next batch.
            before_sig = sigs.last().map(|s| s.signature.clone());

            if sigs.len() < 1000 {
                break; // no more pages
            }
        }

        Ok(all_swaps)
    }

    async fn get_signatures(
        &self,
        program_id: &str,
        before: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SignatureInfo>, Error> {
        let mut config = serde_json::json!({
            "limit": limit,
            "commitment": "confirmed"
        });
        if let Some(b) = before {
            config["before"] = serde_json::Value::String(b.to_string());
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": [program_id, config]
        });

        let resp: RpcResponse<Vec<SignatureInfo>> = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Decode(format!("RPC request failed: {e}")))?
            .json()
            .await
            .map_err(|e| Error::Decode(format!("RPC parse failed: {e}")))?;

        Ok(resp.result.unwrap_or_default())
    }
}

async fn fetch_and_decode(
    http: &reqwest::Client,
    rpc_url: &str,
    signature: &str,
    registry: &DexRegistry,
) -> Result<Vec<SwapEvent>, Error> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [signature, {
            "encoding": "jsonParsed",
            "maxSupportedTransactionVersion": 0,
            "commitment": "confirmed"
        }]
    });

    let resp: RpcResponse<TransactionEnvelope> = http
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::Decode(format!("getTransaction failed: {e}")))?
        .json()
        .await
        .map_err(|e| Error::Decode(format!("getTransaction parse: {e}")))?;

    let envelope = match resp.result {
        Some(e) => e,
        None => return Ok(Vec::new()),
    };

    let meta = match envelope.meta {
        Some(m) => m,
        None => return Ok(Vec::new()),
    };

    // Build full account key list.
    let mut account_keys: Vec<String> = envelope.transaction.message.account_keys.clone();
    if let Some(loaded) = &meta.loaded_addresses {
        account_keys.extend(loaded.writable.clone());
        account_keys.extend(loaded.readonly.clone());
    }

    // Identify DEX.
    let dex = match registry.identify(&account_keys) {
        Some(d) => d,
        None => return Ok(Vec::new()),
    };

    let signer = account_keys
        .first()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    let sig = envelope
        .transaction
        .signatures
        .first()
        .cloned()
        .unwrap_or_else(|| signature.to_string());

    // Compute token balance deltas.
    let pre = meta.pre_token_balances.as_deref().unwrap_or(&[]);
    let post = meta.post_token_balances.as_deref().unwrap_or(&[]);
    let mut deltas = compute_deltas(pre, post);

    // Include native SOL changes.
    if !meta.pre_balances.is_empty() && !meta.post_balances.is_empty() {
        let fee = meta.fee as i64;
        let sol_delta = (meta.post_balances[0] as i64 - meta.pre_balances[0] as i64) + fee;
        let sol_amount = sol_delta as f64 / 1_000_000_000.0;
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

    // Timestamp from blockTime (seconds) → nanoseconds.
    let timestamp = envelope
        .block_time
        .map(|t| t * 1_000_000_000)
        .unwrap_or_else(|| chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));

    // Find swap owner with opposing deltas.
    let mut owners: Vec<&str> = deltas.iter().map(|d| d.owner.as_str()).collect();
    owners.sort_unstable();
    owners.dedup();

    let ordered: Vec<&str> = std::iter::once(signer.as_str())
        .chain(owners.iter().copied().filter(|o| *o != signer.as_str()))
        .collect();

    let mut best_owner: Option<(&str, Vec<&TokenDelta>, Vec<&TokenDelta>)> = None;
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
        timestamp,
        slot: envelope.slot,
        signature: sig,
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

fn compute_deltas(pre: &[RpcTokenBalance], post: &[RpcTokenBalance]) -> Vec<TokenDelta> {
    let pre_map: HashMap<u32, f64> = pre
        .iter()
        .map(|b| (b.account_index, b.ui_token_amount.ui_amount.unwrap_or(0.0)))
        .collect();

    let post_indices: std::collections::HashSet<u32> =
        post.iter().map(|b| b.account_index).collect();

    let mut deltas = Vec::new();

    for b in post {
        let pre_amt = pre_map.get(&b.account_index).copied().unwrap_or(0.0);
        let post_amt = b.ui_token_amount.ui_amount.unwrap_or(0.0);
        let delta = post_amt - pre_amt;
        if delta.abs() > f64::EPSILON {
            deltas.push(TokenDelta {
                mint: b.mint.clone(),
                owner: b.owner.clone().unwrap_or_else(|| "unknown".to_string()),
                amount: delta,
            });
        }
    }

    for b in pre {
        if !post_indices.contains(&b.account_index) {
            let amt = b.ui_token_amount.ui_amount.unwrap_or(0.0);
            if amt.abs() > f64::EPSILON {
                deltas.push(TokenDelta {
                    mint: b.mint.clone(),
                    owner: b.owner.clone().unwrap_or_else(|| "unknown".to_string()),
                    amount: -amt,
                });
            }
        }
    }

    deltas
}
