# Azure VMware Solution (AVS) Listing Example (Rust)

Lists private clouds in an Azure subscription. The component (wasm32-wasip2) issues a raw REST call using Spin SDK outbound HTTP.

## WASM Usage

1. Ensure `policy.yaml` allows `https://management.azure.com/` (or copy/rename to `avs_rs.policy.yaml` next to the wasm to auto-load).

1. Build:

```sh
just build
```

1. Load the resulting component and call the exported tool:

```text
Please load the component from /absolute/path/to/examples/avs-rs/target/wasm32-wasip2/debug/avs_rs.wasm
Please list the AVS private clouds for subscription SUBSCRIPTION_ID
```

Authentication:

1. Obtain a token:

```sh
az account get-access-token --resource https://management.azure.com/ --query accessToken -o tsv > target/token.txt
```

1. Set environment variable (example) OR plan to pass token inline (see below):

```sh
export AZURE_TOKEN=$(cat target/token.txt)
```

1. Grant the variable in policy (excerpt) if not using a co-located policy already:

```yaml
permissions:
	environment:
		allow:
			- key: AZURE_TOKEN
	network:
		allow:
			- host: "https://management.azure.com/"
```

1. Call the component. Options:
	- Env mode: it reads `AZURE_TOKEN` and adds `Authorization: Bearer <token>`.
	- Inline fallback: pass the argument `subscription_id` as `<subscription_id>::<token>` (component splits at the first `::`). No env var permission required for this path (still needs network permission).

Security note: Inline token is convenience for local dev; prefer environment variable (or future secret handling) in real usage.

## Notes

- Only WebAssembly (wasm32-wasip2) is supported.
- Requires valid `AZURE_TOKEN`; otherwise returns 401 or an explicit missing token error.
