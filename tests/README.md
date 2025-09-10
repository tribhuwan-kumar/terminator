# OpenTelemetry Integration Tests

This directory contains integration tests for the OpenTelemetry feature in terminator-mcp-agent.

## Structure

- `integration/` - Integration tests for telemetry functionality
  - `test-http-events.js` - Mock OTLP collector for testing
  - `test-otel-integration.js` - Main integration test suite
- `fixtures/` - Test fixtures and sample workflows
  - `test-workflow.yml` - Sample workflow for testing

## Running Tests

### Prerequisites
```bash
# Install Node.js dependencies
npm install

# Build the project with telemetry feature
cargo build -p terminator-mcp-agent --features telemetry
```

### Run all tests
```bash
npm test
```

### Run specific tests

#### Test without telemetry
```bash
cargo test -p terminator-mcp-agent
```

#### Test with telemetry
```bash
cargo test -p terminator-mcp-agent --features telemetry
```

#### Integration tests
```bash
# Start mock OTLP collector
node tests/integration/test-http-events.js &

# Run integration tests
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 node tests/integration/test-otel-integration.js
```

## Environment Variables

- `OTEL_SDK_DISABLED` - Set to `true` to disable telemetry even when compiled with the feature
- `OTEL_EXPORTER_OTLP_ENDPOINT` - OTLP collector endpoint (default: `http://localhost:4318`)

## CI/CD

Tests are automatically run in CI via GitHub Actions on:
- Push to main or feature branches
- Pull requests targeting main
- Changes to MCP agent or test files

See `.github/workflows/test-telemetry.yml` for the CI configuration.