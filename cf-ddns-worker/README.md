# cf-ddns-worker
This crate creates a [Cloudflare Worker](https://www.cloudflare.com/developer-platform/products/workers/) that replies to HTTP requests with the client's IP address.

## Development
Developing this worker uses the [Rust support for Cloudflare Workers](https://developers.cloudflare.com/workers/languages/rust/).

### Run locally
```bash
npx wrangler dev
```

If you fail to start the local server with an uncaught link error, try overridding the Rust version to `1.81` ([ref](https://github.com/cloudflare/workers-rs/issues/658)):

```bash
rustup override set 1.81
```

### Test
Testing is done using wasm-bindgen-test ([ref](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/usage.html)):

```bash
wasm-pack test --node
```
