# MCP Server Authentication

The Terminator MCP server and CLI support Bearer token authentication for HTTP transport, following OAuth 2.1 best practices.

## Overview

- **Authentication Type**: Bearer Token (OAuth 2.1 compliant)
- **Supported Transports**: HTTP only (SSE transport does not support authentication yet)
- **Configuration**: Environment variable or command-line argument

## Server Configuration

### Using Environment Variable

```bash
# Set the authentication token
export MCP_AUTH_TOKEN="your-secret-token-here"

# Start the server with HTTP transport
terminator-mcp-agent -t http --port 3000
```

### Using Command-Line Argument

```bash
# Start the server with authentication enabled
terminator-mcp-agent -t http --port 3000 --auth-token "your-secret-token-here"
```

### Server Logs

When authentication is enabled, you'll see:
```
🔒 Authentication enabled - Bearer token required
```

When authentication is disabled:
```
⚠️  Authentication disabled - server is publicly accessible
```

## Client Configuration

### Using Environment Variable

```bash
# Set the authentication token
export MCP_AUTH_TOKEN="your-secret-token-here"

# Connect to authenticated server
terminator mcp chat --url http://localhost:3000
```

The CLI will automatically detect the token and show:
```
🔒 Using authentication token from MCP_AUTH_TOKEN environment variable
```

### Example Usage

#### 1. Start authenticated server:
```bash
export MCP_AUTH_TOKEN="my-secure-token-123"
terminator-mcp-agent -t http --port 3000
```

#### 2. Connect from client:
```bash
export MCP_AUTH_TOKEN="my-secure-token-123"
terminator mcp chat --url http://localhost:3000
```

#### 3. Execute workflow:
```bash
export MCP_AUTH_TOKEN="my-secure-token-123"
terminator mcp run --url http://localhost:3000 workflow.yml
```

## Security Best Practices

1. **Token Generation**: Use a strong, randomly generated token
   ```bash
   # Generate a secure token (example)
   openssl rand -base64 32
   ```

2. **Token Storage**: Store tokens in environment variables, not in code
   - Use `.env` files for development (excluded from git)
   - Use system environment variables or secrets managers for production

3. **HTTPS**: Use HTTPS in production to encrypt token transmission
   ```bash
   terminator-mcp-agent -t http --port 3000 --auth-token "$MCP_AUTH_TOKEN"
   # In production, put this behind an HTTPS reverse proxy (nginx, Caddy, etc.)
   ```

4. **Token Rotation**: Regularly rotate authentication tokens

## Authentication Flow

```
Client Request:
  GET /mcp HTTP/1.1
  Authorization: Bearer your-secret-token-here

Server Response (Success):
  HTTP/1.1 200 OK
  { "result": "..." }

Server Response (Failure):
  HTTP/1.1 401 Unauthorized
  { "error": { "code": -32001, "message": "Unauthorized - invalid or missing Bearer token" } }
```

## Limitations

- **SSE Transport**: Authentication is not yet supported for SSE transport. Use HTTP transport when authentication is required.
- **STDIO Transport**: Not applicable (local process communication)

## Troubleshooting

### 401 Unauthorized Error

If you receive a 401 error, check:

1. Token is set on both server and client
2. Token values match exactly
3. Using HTTP transport (not SSE)
4. Token is being sent in Authorization header

### Server logs showing "Authentication disabled"

This means no `--auth-token` or `MCP_AUTH_TOKEN` was provided to the server. The server will accept all requests without authentication.

## Implementation Details

- **Server**: Uses Axum middleware for Bearer token validation
- **Client**: Uses RMCP's `StreamableHttpClientTransportConfig::auth_header()` method to add Authorization header
- **Token Format**: `Authorization: Bearer <token>`
- **Standards Compliance**: Follows OAuth 2.1 and RFC 8707 (Resource Servers)

### Code Example

**Client-side (terminator-cli):**
```rust
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;

// With authentication
let config = StreamableHttpClientTransportConfig::with_uri("http://localhost:3000/mcp")
    .auth_header("your-token-here");
let transport = StreamableHttpClientTransport::with_client(reqwest::Client::new(), config);

// Without authentication
let transport = StreamableHttpClientTransport::from_uri("http://localhost:3000/mcp");
```

**Server-side (terminator-mcp-agent):**
```rust
// Authentication middleware validates Bearer token
async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
        if let Some(token) = auth_header.to_str().ok().and_then(|v| v.strip_prefix("Bearer ")) {
            if state.auth_token.as_deref() == Some(token) {
                return next.run(req).await;
            }
        }
    }
    // Return 401 Unauthorized
}
```
