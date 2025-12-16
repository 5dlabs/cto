//! Latitude.sh GPU VM API client implementation.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use tracing::{debug, info, warn};

use super::models::{
    ApiResponse, CreateVirtualMachineAttributes, CreateVirtualMachineBody,
    CreateVirtualMachineData, VirtualMachineActionAttributes, VirtualMachineActionBody,
    VirtualMachinePlanResource, VirtualMachineResource,
};
use crate::providers::traits::{
    CreateGpuVmRequest, GpuPlan, GpuProvider, GpuProviderError, GpuSpecs, GpuVm, GpuVmStatus,
};

/// Base URL for Latitude.sh API.
const API_BASE_URL: &str = "https://api.latitude.sh";

/// Default timeout for API requests.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Polling interval when waiting for VM status.
const POLL_INTERVAL_SECS: u64 = 10;

/// Latitude.sh GPU VM provider.
#[derive(Clone)]
pub struct Latitude {
    /// HTTP client.
    client: Client,
    /// API key for authentication.
    api_key: String,
    /// Project ID for VM operations.
    project_id: String,
}

impl Latitude {
    /// Create a new Latitude GPU provider.
    ///
    /// # Arguments
    /// * `api_key` - Latitude.sh API key
    /// * `project_id` - Project ID for VM operations
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created.
    pub fn new(
        api_key: impl Into<String>,
        project_id: impl Into<String>,
    ) -> Result<Self, GpuProviderError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            project_id: project_id.into(),
        })
    }

    /// Make an authenticated GET request.
    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, GpuProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "GET request");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request.
    async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, GpuProviderError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "POST request");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request that returns empty body.
    async fn post_empty<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), GpuProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "POST request (empty response)");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(GpuProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Make an authenticated DELETE request.
    async fn delete(&self, path: &str) -> Result<(), GpuProviderError> {
        let url = format!("{API_BASE_URL}{path}");
        debug!(url = %url, "DELETE request");

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let status = response.status();
        if status.is_success() || status == StatusCode::NOT_FOUND {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(GpuProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Handle API response, parsing JSON or error.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, GpuProviderError> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&text).map_err(|e| {
                warn!(error = %e, body = %text, "Failed to parse response");
                GpuProviderError::Serialization(e)
            })
        } else if status == StatusCode::NOT_FOUND {
            Err(GpuProviderError::NotFound(text))
        } else {
            Err(GpuProviderError::Api {
                status: status.as_u16(),
                message: text,
            })
        }
    }

    /// Convert API VM resource to our `GpuVm` type.
    fn to_gpu_vm(resource: &VirtualMachineResource) -> GpuVm {
        let status = match resource.attributes.status.as_str() {
            "Scheduling" => GpuVmStatus::Scheduling,
            "Scheduled" => GpuVmStatus::Scheduled,
            "Starting" => GpuVmStatus::Starting,
            "Configuring network" => GpuVmStatus::ConfiguringNetwork,
            "Running" => GpuVmStatus::Running,
            "Stopped" => GpuVmStatus::Stopped,
            _ => GpuVmStatus::Unknown,
        };

        let specs = resource.attributes.specs.as_ref().map(|s| GpuSpecs {
            gpu_model: s.gpu.clone().unwrap_or_else(|| "Unknown".to_string()),
            gpu_count: 1, // Latitude VMs have 1 GPU
            gpu_memory_gb: None,
            vcpus: s.vcpu.unwrap_or(0),
            ram_gb: 0, // RAM is in string format
            storage_gb: 0,
        });

        GpuVm {
            id: resource.id.clone(),
            name: resource.attributes.name.clone(),
            status,
            host: resource
                .attributes
                .credentials
                .as_ref()
                .and_then(|c| c.host.clone()),
            username: resource
                .attributes
                .credentials
                .as_ref()
                .and_then(|c| c.username.clone()),
            plan_id: resource
                .attributes
                .plan
                .as_ref()
                .and_then(|p| p.id.clone())
                .unwrap_or_default(),
            specs,
            created_at: resource
                .attributes
                .created_at
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        }
    }

    /// Convert API plan resource to our `GpuPlan` type.
    fn to_gpu_plan(resource: &VirtualMachinePlanResource) -> GpuPlan {
        let name = resource
            .attributes
            .name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        // Extract GPU model from plan name (e.g., "vm.h100.small" -> "H100")
        let gpu_model = if name.contains("h100") {
            "H100".to_string()
        } else if name.contains("l40s") {
            "L40S".to_string()
        } else if name.contains("rtx6k") || name.contains("rtx6000") {
            "RTX 6000 Pro".to_string()
        } else if name.contains("a100") {
            "A100".to_string()
        } else {
            "Unknown GPU".to_string()
        };

        let specs = resource.attributes.specs.as_ref();
        let vcpus = specs.and_then(|s| s.vcpus).unwrap_or(0);
        let memory_gb = specs.and_then(|s| s.memory).unwrap_or(0);
        let storage_gb = specs
            .and_then(|s| s.disk.as_ref())
            .and_then(|d| d.size.as_ref())
            .and_then(|s| s.amount)
            .unwrap_or(0);

        // Get pricing from first region
        let (price_per_hour, price_per_month) = resource
            .attributes
            .regions
            .as_ref()
            .and_then(|regions| regions.first())
            .and_then(|r| r.pricing.as_ref())
            .and_then(|p| p.usd.as_ref())
            .map_or((0.0, 0.0), |usd| {
                (usd.hour.unwrap_or(0.0), usd.month.unwrap_or(0.0))
            });

        // Get available regions
        let available_regions = resource
            .attributes
            .regions
            .as_ref()
            .map(|regions| {
                regions
                    .iter()
                    .filter_map(|r| r.available.as_ref())
                    .flatten()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        GpuPlan {
            id: resource.id.clone(),
            name,
            specs: GpuSpecs {
                gpu_model,
                gpu_count: 1,
                gpu_memory_gb: None,
                vcpus,
                ram_gb: memory_gb, // API returns GB directly
                storage_gb,
            },
            price_per_hour,
            price_per_month,
            available_regions,
            stock_level: resource
                .attributes
                .stock_level
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

#[async_trait]
impl GpuProvider for Latitude {
    async fn list_gpu_plans(&self) -> Result<Vec<GpuPlan>, GpuProviderError> {
        let response: ApiResponse<Vec<VirtualMachinePlanResource>> =
            self.get("/plans/virtual_machines").await?;
        Ok(response.data.iter().map(Self::to_gpu_plan).collect())
    }

    async fn create_gpu_vm(&self, req: CreateGpuVmRequest) -> Result<GpuVm, GpuProviderError> {
        info!(name = %req.name, plan_id = %req.plan_id, "Creating GPU VM");

        let body = CreateVirtualMachineBody {
            data: CreateVirtualMachineData {
                resource_type: "virtual_machines".to_string(),
                attributes: CreateVirtualMachineAttributes {
                    name: req.name,
                    plan: req.plan_id,
                    ssh_keys: req.ssh_keys,
                    project: self.project_id.clone(),
                },
            },
        };

        let response: ApiResponse<VirtualMachineResource> =
            self.post("/virtual_machines", &body).await?;
        let vm = Self::to_gpu_vm(&response.data);

        info!(vm_id = %vm.id, status = %vm.status, "GPU VM created");
        Ok(vm)
    }

    async fn get_gpu_vm(&self, id: &str) -> Result<GpuVm, GpuProviderError> {
        let response: ApiResponse<VirtualMachineResource> =
            self.get(&format!("/virtual_machines/{id}")).await?;
        Ok(Self::to_gpu_vm(&response.data))
    }

    async fn list_gpu_vms(&self) -> Result<Vec<GpuVm>, GpuProviderError> {
        let response: ApiResponse<Vec<VirtualMachineResource>> = self
            .get("/virtual_machines?extra_fields[virtual_machines]=credentials")
            .await?;
        Ok(response.data.iter().map(Self::to_gpu_vm).collect())
    }

    async fn delete_gpu_vm(&self, id: &str) -> Result<(), GpuProviderError> {
        info!(vm_id = %id, "Deleting GPU VM");
        self.delete(&format!("/virtual_machines/{id}")).await?;
        info!(vm_id = %id, "GPU VM deleted");
        Ok(())
    }

    async fn gpu_vm_action(&self, id: &str, action: &str) -> Result<(), GpuProviderError> {
        info!(vm_id = %id, action = %action, "Running GPU VM action");

        let body = VirtualMachineActionBody {
            id: id.to_string(),
            resource_type: "virtual_machines".to_string(),
            attributes: VirtualMachineActionAttributes {
                action: action.to_string(),
            },
        };

        self.post_empty(&format!("/virtual_machines/{id}/actions"), &body)
            .await?;

        info!(vm_id = %id, action = %action, "GPU VM action completed");
        Ok(())
    }

    async fn wait_ready(&self, id: &str, timeout_secs: u64) -> Result<GpuVm, GpuProviderError> {
        info!(vm_id = %id, timeout_secs, "Waiting for GPU VM to be ready");

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let vm = self.get_gpu_vm(id).await?;

            debug!(
                vm_id = %id,
                status = %vm.status,
                elapsed_secs = start.elapsed().as_secs(),
                "Polling GPU VM status"
            );

            if vm.status == GpuVmStatus::Running {
                info!(vm_id = %id, "GPU VM is ready");
                return Ok(vm);
            }

            if start.elapsed() > timeout {
                return Err(GpuProviderError::Timeout(timeout_secs));
            }

            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_vm_status_display() {
        assert_eq!(GpuVmStatus::Running.to_string(), "running");
        assert_eq!(GpuVmStatus::Scheduling.to_string(), "scheduling");
        assert_eq!(
            GpuVmStatus::ConfiguringNetwork.to_string(),
            "configuring_network"
        );
    }
}



