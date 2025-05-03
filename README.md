# Pollhook: Webhook-to-Polling Conversion Service

A flexible Rust-based service that converts webhook-based integrations to polling-based ones, enabling webhook usage in environments where exposing public endpoints is challenging.

> **Key Benefit**: Deploy on free cloud instances with public IPs to receive webhooks, while processing data on your powerful local machines. Perfect for development environments or restrictive network setups.

## 🚀 Overview

This service creates a lightweight bridge between webhook providers and your local environment:

1. Receives webhooks on a public cloud instance
2. Handles webhook verification flows
3. Caches received data with custom aliases
4. Allows secure polling from your local machines

## 🌐 Cloud Deployment Options

Many providers offer free VM instances with public IPs:

- **Oracle Cloud**: Always Free tier includes 2 AMD-based Compute VMs
- **Google Cloud**: Free tier includes 1 f1-micro instance
- **AWS**: 12-month free tier includes t2.micro/t3.micro instances
- **Azure**: 12-month free tier includes B1s instances

**Pro Tip**: Get free TLS certificates from Cloudflare by:
1. Register a domain (or use a free subdomain service)
2. Add it to Cloudflare's free plan
3. Use Cloudflare's Origin CA to generate certificates
4. Configure the service with your certificates

## ✨ Features

- **Webhook Verification**: Handles various verification schemes
- **Data Caching**: Stores webhook payloads with custom aliases
- **Secure Polling**: Retrieve data with token authentication
- **YAML Configuration**: No code changes needed for new integrations
- **TLS Support**: Optional HTTPS with certificate configuration

## 🔧 Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/webhook-poller
cd webhook-poller
```

2. Build the project:

```bash
cargo build --release
```

3. Create a configuration file (see Configuration section below)

4. Set required environment variables:

```bash
export CONFIG_FILE_PATH=./config_webhook.yaml
export VERIFY_TOKEN=your_secret_token
export DATA_RETRIEVE_TOKEN=your_polling_token
export PORT=8080
export CACHE_TTL=300
export POLLING_TIMEOUT=20
export POLL_ITEMS_COUNT=5
```

## 🛠️ Configuration

The service uses a YAML configuration file. Here's a sample configuration:

```yaml
verification:
  path: /pollhook/webhook
  method: GET  # Default is GET if not specified
  token:
    in: query
    locate: hub.verify_token
  challenge:
    in: query
    locate: hub.challenge
  response:
    type: text/plain
    data: "@challenge"

data:
  meta_event:
    path: /callhook/meta
    method: POST

```

### Configuration Options

#### Verification Section

- `path`: The base path for verification endpoints (must start with "callhook")
- `method`: HTTP method (GET, POST, etc.)
- `token`: How to extract the verification token
    - `in`: Location (query, header, body, path)
    - `locate`: Parameter name or path
- `challenge`: How to extract the challenge
    - `in`: Location (query, header, body, path)
    - `locate`: Parameter name or path
- `response`: How to format the response
    - `type`: Content type (text/plain, application/json)
    - `data`: Response data template (use @challenge for the challenge value)
    - `in_path`: For JSON responses, specifies where to put the data


You can use `...` as a wildcard in paths:

```yaml
verification:
  path: /callhook/.../callback
```

This will match paths like:
- `/callhook/facebook/callback`

#### Sample Yaml For Configuration Options

```yaml
# Basic query parameter verification
verification:
  path: /callhook/webhook # Matches any path starting with /callhook/webhook
  token:
    in: query
    locate: hub.verify_token # Extracts token from query parameter named 'hub.verify_token'
  challenge:
    in: query
    locate: hub.challenge # Extracts challenge from query parameter named 'hub.challenge'
  response:
    type: text/plain # Supported types: text/plain, application/json
    data: "@challenge" # Returns the challenge value as plain text. Currently only @challenge is supported.
```
```yaml
# Path parameter verification with wildcards
verification:
  path: /callhook/... # Matches any path with exactly 2 segments starting with /callhook (e.g., /callhook/callback)
  token:
    in: query
    locate: hub.verify_token # Extracts token from query parameter named 'hub.verify_token'
  challenge:
    in: path
    locate: 4 # Extracts from the 4th path segment (e.g., /callhook/one/two/three => extracts 'three')
  response:
    type: text/plain # Supported types: text/plain, application/json
    data: "@challenge" # Returns the challenge value as plain text. Currently only @challenge is supported.
```
```yaml
# Header and body verification with JSON response
verification:
  path: /callhook/.../callback # Matches paths like /callhook/{any}/callback (exactly one wildcard segment)
  method: POST # HTTP method (default: GET)
  token:
    in: header
    locate: X-API-TOKEN # Extracts token from request header 'X-API-TOKEN'
  challenge:
    in: body
    locate: data::token # Extracts token from request body at path: data.token
  response:
    type: application/json
    in: verification::resp
    data: "@challenge" # Returns challenge value in JSON format: {"verification": {"resp": "value"}}
```

#### Data Section

This section defines endpoints for capturing webhook data. The configuration below specifies that any event sent to the path `/callhook/meta` using the POST method will be cached for later polling.

The system uses **alias** to differentiate between events sent to different endpoints. In this example, the **alias** is `meta_event`:

```yaml
data:
  meta_event:
    path: /callhook/meta
    method: POST
```
#### Polling Section

To retrieve webhook data from your local environment, use the following command:
  
```bash
curl -H "Authorization: Bearer your_polling_token" \
     https://your-domain.com/pollhook/{alias}
```
The response varies based on the **alias** used. For the meta_event alias, you'll receive a response similar to this:

```json
{
  "success": true,
  "message": "Retrieved 1 items after polling",
  "count": 1,
  "data": [
    {
      "_cache_key": "9282e48fbaf9b4f320bd3af07852387c41a2c6f45c559e6f7817412c8818cfe3",
      "entry": [
        {
          "id": "578564948682799",
          "messaging": [
            {
              "message": {
                "mid": "m_X9N_5uR02eMA3iEvsGa_HeEQSg_J7Zyru3LWfIH4ajqc-AygIDOlNLVlOS8oW7WCtA9yVcGVLqC-9HBXUO_9Ug",
                "text": "hello"
              },
              "recipient": {
                "id": "578564948682799"
              },
              "sender": {
                "id": "29393601576952459"
              },
              "timestamp": 1746313252871
            }
          ],
          "time": 1746313253959
        }
      ],
      "object": "page"
    }
  ]
}
```
* _cache_key: The unique identifier for the cached message stored in memory
* entry: The actual webhook payload

## 🚀 Usage

1. Configure your yaml file for verification and data retrieval

2. Config necessary IP, domain name and SSL/TLS settings

3. Start the server in your cloud virtual machine.

4. Use your polling client to retrieve webhook data from your local network



## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
