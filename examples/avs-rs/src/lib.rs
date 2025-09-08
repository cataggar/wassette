#[allow(warnings)]
mod avs;

use avs::Guest;

struct Component;

impl Guest for Component {
    fn list_private_clouds(subscription_id: String) -> Result<String, String> {
        spin_executor::run(async move { list_clouds_via_rest(subscription_id).await })
    }
}

// Demonstrates using the Azure Rust SDK directly with a static token.
// It relies on a bearer token supplied via the AZURE_TOKEN environment variable and
// implements a lightweight TokenCredential so that the generated management client
// can operate without azure_identity.
async fn list_clouds_via_rest(subscription_id: String) -> Result<String, String> {
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

    let mut ids = Vec::new();
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
        if let Some(id) = cloud.tracked_resource.resource.id.as_ref() {
            ids.push(id.clone());
        }
    }

    if ids.is_empty() {
        Ok("".to_string())
    } else {
        Ok(ids.join("\n"))
    }
}

avs::export!(Component with_types_in avs);
