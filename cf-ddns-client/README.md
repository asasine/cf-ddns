# cf-ddns-client
This binary calls the [cf-ddns Worker](../cf-ddns-worker/) to get the client's IP address and then updates the DNS records of a domain with the IP address using the Cloudflare API.

## Development
When developing alongside the [cf-ddns Worker](../cf-ddns-worker/), you can change the URL with the `--url`.

## Installation
```bash
deb=$(cargo deb)
sudo apt install $deb
sudo systemctl edit cf-ddns.service
# [Service]
# Environment=ZONE_NAME=example.com
# Environment=RECORD_NAME=some-record

sudo vim /etc/cf-ddns/token.txt
# <API_TOKEN>

sudo systemctl start cf-ddns.service
```
