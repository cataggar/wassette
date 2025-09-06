// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#[allow(warnings)]
mod avs;

use avs::Guest;

struct Component;

impl Guest for Component {
    fn list_private_clouds(subscription_id: String) -> Result<String, String> {
        // For wasm build, we cannot directly use the Azure Rust SDK (depends on native features).
        // Instead, demonstrate an HTTP GET to Azure REST management endpoint using Spin SDK outbound HTTP.
        // NOTE: In a real secure setup you'd have the host inject an authorization header via policy/host capability.
        spin_executor::run(async move { list_clouds_via_rest(subscription_id).await })
    }
}

async fn list_clouds_via_rest(subscription_id: String) -> Result<String, String> {
    use spin_sdk::http::{Request, send};
    use spin_sdk::http::Response;    
    use spin_sdk::variables;

    // Minimal: call list private clouds API (API version may need adjusting based on service).
    // Endpoint shape: GET https://management.azure.com/subscriptions/{subscriptionId}/providers/Microsoft.AVS/privateClouds?api-version=2024-09-01
    let url = format!("https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.AVS/privateClouds?api-version=2024-09-01");

    // Acquire token from environment/policy via Spin variables API.
    // Expect an environment variable AZURE_TOKEN (Bearer token) to be granted via policy.
    let token = variables::get("AZURE_TOKEN").map_err(|_| "AZURE_TOKEN not set or not accessible".to_string())?;
    if token.trim().is_empty() {
        return Err("AZURE_TOKEN is empty".to_string());
    }

    let mut req = Request::get(url);
    req.header("Authorization", format!("Bearer {token}"));
    let resp: Response = send(req).await.map_err(|e| e.to_string())?;
    let body_bytes = resp.body();
    if !(200..300).contains(resp.status()) {
        return Err(format!("Azure REST call failed: status={} body={}", resp.status(), String::from_utf8_lossy(body_bytes)));
    }
    let body_str = String::from_utf8_lossy(body_bytes);
    // Attempt to parse JSON and extract newline-delimited IDs from value[].id
    match serde_json::from_str::<serde_json::Value>(&body_str) {
        Ok(v) => {
            let mut ids = Vec::new();
            if let Some(arr) = v.get("value").and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                        ids.push(id.to_string());
                    }
                }
            }
            if ids.is_empty() {
                // Fallback to raw JSON if structure unexpected.
                Ok(body_str.into_owned())
            } else {
                Ok(ids.join("\n"))
            }
        }
        Err(_) => Ok(body_str.into_owned()),
    }
}

avs::export!(Component with_types_in avs);
