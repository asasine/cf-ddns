# Cloudflare Dynamic DNS
This project maintains a [Dynamic DNS](https://www.cloudflare.com/learning/dns/glossary/dynamic-dns/) (DDNS) service using [Cloudflare](https://www.cloudflare.com/).

It works by creating a free [Cloudflare Worker](https://www.cloudflare.com/developer-platform/products/workers/) that replies to HTTP requests with the client's IP address.
The client calls this worker and then uses a Cloudflare API token to update the DNS records of a domain with IP address in the response from the worker.
