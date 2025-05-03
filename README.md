# Webhook-to-Polling Conversion Service

A flexible Rust-based service that allows converting webhook-based integrations to polling-based ones, helping you use webhook-based services in environments where exposing public endpoints is challenging.

> **Key Benefit**: Many cloud providers offer cheap or even free solutions with a public IP address that can receive webhooks. This service lets you host a lightweight webhook receiver in the cloud, while polling from your more powerful home or office machines to process the data. Perfect for developers with robust local development environments but limited public internet exposure.

## 🚀 Overview

Many cloud services use webhooks to notify your application about events, requiring a public endpoint to receive these notifications.

**This service leverages affordable (often free) cloud hosting options with public IP addresses to:**

1. Provide a lightweight public endpoint that can receive webhook calls
2. Support webhook verification flows from various providers
3. Store received webhook data
4. Allow your more powerful local machines to poll for this data instead of exposing endpoints

This setup gives you the best of both worlds - an always-available public webhook endpoint on an inexpensive cloud instance, while processing the data on your more powerful local hardware. Perfect for development environments, restrictive network setups, or situations where running comprehensive processing on cloud instances would be costly.

## ✨ Features

- **Flexible Verification**: Handles various webhook verification schemes with configurable token and challenge extraction
- **Dynamic Path Routing**: Support for wildcards and path variations
- **Adaptive Response Generation**: Customizable response formats and content types
- **Configurable Through YAML**: No code changes needed for new webhook integrations
- **Multiple Extraction Methods**: Get tokens, challenges, and data from:
    - URL Query Parameters
    - Request Headers
    - Request Body (JSON)
    - URL Path Segments

## 📋 Requirements

- Rust 1.54 or newer
- Actix Web 4.0+
- Serde and Serde YAML for configuration
- Environment with public internet access

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
export PORT=8080
```

## 🛠️ Configuration

The service uses a YAML configuration file. Here's a sample configuration:

```yaml
verification:
  path: /verification/webhook
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
  hello:
    path: /update/{id}
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

### Path Wildcards

You can use `...` as a wildcard in paths:

```yaml
verification:
  path: /callhook/.../callback
```

This will match paths like:
- `/callhook/facebook/callback`

### Sample Yaml
### Configuration Options

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

## 🚀 Usage

1. Configure your yaml file for verification and data retrieval

2. Start the server.

3. Use the polling client (in development) to retrieve webhook data from your local network


## 🔒 Security Considerations

- Store your `VERIFY_TOKEN` securely
- Consider implementing rate limiting to prevent abuse
- For production use, add authentication to your polling endpoints
- Use HTTPS for all communications

## 🔄 Webhook Providers Tested

- Facebook Messenger Platform
- Slack Events API
- GitHub Webhooks
- Generic webhook systems

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request