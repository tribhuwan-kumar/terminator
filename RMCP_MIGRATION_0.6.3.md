# RMCP 0.4.0 to 0.6.3 Migration Guide

## Overview
This PR updates the rmcp dependency from 0.4.0 to 0.6.3. Due to significant breaking changes in the rmcp API, the code requires substantial modifications to compile.

## Breaking Changes Identified

### 1. Transport API Changes
- `StreamableHttpClientTransport::from_uri()` has been removed
- New constructor requires explicit client and config:
  ```rust
  // Old (0.4.0)
  let transport = StreamableHttpClientTransport::from_uri(url);
  
  // New (0.6.3)
  let transport = StreamableHttpClientTransport::with_client(
      reqwest::Client::new(),
      StreamableHttpClientTransportConfig {
          base_url: url.to_string(),
          ..Default::default()
      },
  );
  ```

### 2. Content Field Type Change
- `CallToolResult.content` changed from `Option<Vec<Content>>` to `Vec<Content>`
  ```rust
  // Old (0.4.0)
  if let Some(content_vec) = &result.content {
      for content in content_vec { ... }
  }
  
  // New (0.6.3)
  if !result.content.is_empty() {
      for content in &result.content { ... }
  }
  ```

### 3. Import Path Changes
- `Parameters` struct moved to different module:
  ```rust
  // Old
  use rmcp::handler::server::tool::Parameters;
  
  // New
  use rmcp::handler::server::wrapper::parameters::Parameters;
  ```

### 4. New Enum Variants
- `RawContent` enum has new `ResourceLink` variant that needs handling
- Requires adding match arms for exhaustive pattern matching

### 5. HTTP Transport Header Requirements
- The HTTP transport now strictly requires `Accept: application/json, text/event-stream` header
- This causes 406 Not Acceptable errors with default reqwest clients

## Migration Status

### Files Modified
- `terminator-cli/Cargo.toml` - Updated rmcp to 0.6.3
- `terminator-mcp-agent/Cargo.toml` - Updated rmcp to 0.6.3
- `terminator-cli/src/mcp_client.rs` - Partial fixes applied
- `terminator-mcp-agent/src/server.rs` - Import path fixed

### Remaining Work
- [ ] Fix all 79 compilation errors
- [ ] Update all `from_uri()` calls to use new constructor pattern
- [ ] Convert all Option<Vec<Content>> patterns to Vec<Content>
- [ ] Add handling for new ResourceLink variant
- [ ] Test HTTP transport with proper headers
- [ ] Update integration tests

## Benefits of Upgrading

1. **Better HTTP Transport Support** - More robust handling of SSE and JSON responses
2. **Performance Improvements** - Various optimizations in the transport layer
3. **Bug Fixes** - Multiple issues resolved since 0.4.0
4. **Future Compatibility** - Stay current with the MCP ecosystem

## Recommendation

Due to the extensive breaking changes, this migration should be done in phases:

1. **Phase 1** (This PR): Update dependencies and document breaking changes
2. **Phase 2**: Fix compilation errors systematically
3. **Phase 3**: Update tests and verify functionality
4. **Phase 4**: Test with MCP cluster deployment

## Testing Required

After migration:
- [ ] Verify stdio transport still works
- [ ] Test HTTP transport with cluster
- [ ] Verify SSE transport functionality
- [ ] Test all MCP tools and commands
- [ ] Validate UI automation workflows

## Notes

The primary blocker for the MCP cluster is the HTTP transport header requirement. The 0.6.3 version properly validates Accept headers, which the 0.4.0 client doesn't send correctly. This migration will resolve the 406 Not Acceptable errors encountered with the cluster deployment.