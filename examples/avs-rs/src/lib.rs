#[allow(warnings)]
mod avs;

use avs::Guest;

struct Component;

impl Guest for Component {
    fn list_private_clouds(subscription_id: String) -> Result<Vec<avs::PrivateCloud>, String> {
        spin_executor::run(async move { list_clouds_via_rest(subscription_id).await })
    }
}

// Demonstrates using the Azure Rust SDK directly with a static token.
// It relies on a bearer token supplied via the AZURE_TOKEN environment variable and
// implements a lightweight TokenCredential so that the generated management client
// can operate without azure_identity.
async fn list_clouds_via_rest(subscription_id: String) -> Result<Vec<avs::PrivateCloud>, String> {
    use futures::TryStreamExt;

    // Acquire token from AZURE_TOKEN env var
    let token_value =
        std::env::var("AZURE_TOKEN").map_err(|_| "AZURE_TOKEN not set".to_string())?;
    if token_value.trim().is_empty() {
        return Err("AZURE_TOKEN is empty".to_string());
    }

    use std::sync::Arc;

    use azure_core::credentials::{AccessToken, TokenCredential};
    use time::{Duration, OffsetDateTime};

    #[derive(Debug)]
    struct StaticTokenCredential {
        token: String,
    }

    #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
    impl TokenCredential for StaticTokenCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<azure_core::credentials::TokenRequestOptions>,
        ) -> Result<AccessToken, azure_core::Error> {
            let expires_on = OffsetDateTime::now_utc() + Duration::hours(1);
            Ok(AccessToken::new(self.token.clone(), expires_on))
        }
    }

    let credential = Arc::new(StaticTokenCredential { token: token_value });

    // Use the Azure VMware Solution management client.
    let client = azure_mgmt_vmware::Client::builder(credential)
        .build()
        .map_err(|e| format!("client build error: {e}"))?;

    let mut clouds = Vec::new();
    let mut pager = client
        .private_clouds_client()
        .list_in_subscription(&subscription_id)
        .pager()
        .map_err(|e| format!("list pager error: {e}"))?;

    while let Some(cloud) = pager
        .try_next()
        .await
        .map_err(|e| format!("pager error: {e}"))?
    {
        // Map Azure SDK model to avs::PrivateCloud
        let props = cloud.properties.as_ref().unwrap();
        let pc = avs::PrivateCloud {
            management_cluster: props.management_cluster.cluster_id.unwrap_or_default().to_string(),
            internet: props.internet.as_ref().map(|v| format!("{:?}", v)),
            identity_sources: Some(props.identity_sources.iter().map(|s| format!("{:?}", s)).collect()),
            availability: props.availability.as_ref().map(|v| format!("{:?}", v)),
            encryption: props.encryption.as_ref().map(|v| format!("{:?}", v)),
            extended_network_blocks: Some(props.extended_network_blocks.clone()),
            provisioning_state: props.provisioning_state.as_ref().map(|v| format!("{:?}", v)),
            circuit: None, // TODO: extract string if needed
            endpoints: None, // TODO: extract string if needed
            network_block: props.network_block.clone(),
            management_network: props.management_network.clone(),
            provisioning_network: props.provisioning_network.clone(),
            vmotion_network: props.vmotion_network.clone(),
            vcenter_password: props.vcenter_password.clone(),
            nsxt_password: props.nsxt_password.clone(),
            vcenter_certificate_thumbprint: props.vcenter_certificate_thumbprint.clone(),
            nsxt_certificate_thumbprint: props.nsxt_certificate_thumbprint.clone(),
            external_cloud_links: Some(props.external_cloud_links.clone()),
            secondary_circuit: None, // TODO: extract string if needed
            nsx_public_ip_quota_raised: None, // TODO: extract string if needed
            virtual_network_id: props.virtual_network_id.clone(),
            dns_zone_type: None, // TODO: extract string if needed
        };
        clouds.push(pc);
    }
    Ok(clouds)
}

avs::export!(Component with_types_in avs);
