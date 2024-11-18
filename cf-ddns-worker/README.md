# cf-ddns-worker
This crate creates a [Cloudflare Worker](https://www.cloudflare.com/developer-platform/products/workers/) that replies to HTTP requests with the client's IP address.

## Development
Developing this worker uses the [Rust support for Cloudflare Workers](https://developers.cloudflare.com/workers/languages/rust/).

### Run locally
```bash
npx wrangler dev
```

### Test
Testing is done using wasm-bindgen-test ([ref](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/usage.html)):

```bash
wasm-pack test --node
```
