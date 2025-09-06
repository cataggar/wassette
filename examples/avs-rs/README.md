# Azure VMware Solution (AVS) Listing Example (Rust)

Lists private clouds in an Azure subscription. The component (wasm32-wasip2) issues a raw REST call using Spin SDK outbound HTTP.

## WASM Usage

1. Ensure `policy.yaml` allows `https://management.azure.com/`.

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

2. Set environment variable (example):

	```sh
	export AZURE_TOKEN=$(cat target/token.txt)
	```

3. Grant the variable in policy (excerpt):

	 ```yaml
	 permissions:
		 environment-variables:
			 allow:
				 - key: AZURE_TOKEN
		 network:
			 allow:
				 - host: "https://management.azure.com/"
	 ```

4. Call the component; it reads `AZURE_TOKEN` and adds `Authorization: Bearer <token>`.

## Notes

- Only WebAssembly (wasm32-wasip2) is supported.
- Requires valid `AZURE_TOKEN`; otherwise returns 401 or an explicit missing token error.
