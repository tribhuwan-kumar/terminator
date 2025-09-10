# OpenTelemetry Integration for Terminator MCP Agent

## Overview

The Terminator MCP Agent now supports OpenTelemetry tracing, allowing you to monitor and trace workflow executions and individual tool calls. This is especially useful for debugging complex workflows and understanding performance bottlenecks.

## Enabling OpenTelemetry

OpenTelemetry is disabled by default. To enable it:

### 1. Build with the telemetry feature flag

```bash
cargo build --features telemetry
```

### 2. Configure the OTLP endpoint

The agent will send traces to an OpenTelemetry collector. Configure the endpoint using environment variables:

```bash
# Set the OTLP endpoint (default: http://localhost:4318)
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318

# Optionally disable telemetry completely
export OTEL_SDK_DISABLED=true
```

## Setting up an OpenTelemetry Collector

### Using Docker Compose

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  # Jaeger for trace visualization
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "14250:14250"  # gRPC
    environment:
      - COLLECTOR_OTLP_ENABLED=true

  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4318:4318"   # OTLP HTTP
      - "4317:4317"   # OTLP gRPC
    depends_on:
      - jaeger
```

Create `otel-collector-config.yaml`:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

exporters:
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true
  
  logging:
    loglevel: debug

processors:
  batch:

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [jaeger, logging]
```

Start the services:

```bash
docker-compose up -d
```

## Running the MCP Agent with Telemetry

```bash
# Build with telemetry support
cargo build --release --features telemetry

# Run with telemetry enabled
./target/release/terminator-mcp-agent --transport http --port 3000

# Or with custom OTLP endpoint
OTEL_EXPORTER_OTLP_ENDPOINT=http://my-collector:4318 \
  ./target/release/terminator-mcp-agent --transport http --port 3000
```

## Example JavaScript Client

Here's an example of using the MCP agent and viewing traces:

```javascript
const { Client } = require('@modelcontextprotocol/sdk/client/index.js');
const { StdioClientTransport } = require('@modelcontextprotocol/sdk/client/stdio.js');

async function runWorkflowWithTracing() {
  // Start the MCP agent with telemetry
  const transport = new StdioClientTransport({
    command: 'cargo',
    args: ['run', '--features', 'telemetry', '--bin', 'terminator-mcp-agent'],
    env: {
      ...process.env,
      OTEL_EXPORTER_OTLP_ENDPOINT: 'http://localhost:4318'
    }
  });

  const client = new Client({
    name: 'telemetry-test-client',
    version: '1.0.0',
  }, {
    capabilities: {}
  });

  await client.connect(transport);

  // Execute a workflow - this will generate traces
  const result = await client.callTool({
    name: 'execute_workflow',
    arguments: {
      workflow_file: 'examples/web_monitor.yml'
    }
  });

  console.log('Workflow result:', result);
  
  // View traces at http://localhost:16686 (Jaeger UI)
  console.log('View traces at: http://localhost:16686');
  
  await client.close();
}

runWorkflowWithTracing().catch(console.error);
```

## Trace Structure

The telemetry implementation creates the following spans:

### Workflow Spans
- **Span Name**: The workflow name
- **Span Kind**: Server
- **Attributes**:
  - `workflow.name`: Name of the workflow
  - `workflow.file`: Path to workflow file
  - `workflow.total_steps`: Number of steps

### Step Spans
- **Span Name**: `step.<tool_name>`
- **Span Kind**: Internal
- **Attributes**:
  - `tool.name`: Tool being executed
  - `step.id`: Step identifier (if provided)
  - `step.index`: Step index in sequence
  - `step.arguments`: Tool arguments (as JSON)

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP collector endpoint | `http://localhost:4318` |
| `OTEL_SDK_DISABLED` | Disable telemetry completely | `false` |
| `OTEL_SERVICE_NAME` | Override service name | `terminator-mcp-agent` |

## Viewing Traces

1. Open Jaeger UI at http://localhost:16686
2. Select "terminator-mcp-agent" from the service dropdown
3. Click "Find Traces"
4. Click on a trace to see the detailed span hierarchy

## Performance Impact

When telemetry is disabled (default), there is zero performance impact as the telemetry code is not compiled. When enabled, the overhead is minimal:
- Span creation: ~1-2 microseconds
- Attribute setting: ~100 nanoseconds
- Network export: Asynchronous, doesn't block execution

## Troubleshooting

### No traces appearing
1. Check the collector is running: `docker ps`
2. Verify the endpoint: `curl http://localhost:4318/v1/traces`
3. Check agent logs for connection errors
4. Ensure telemetry feature is enabled in build

### Connection refused errors
- Ensure the OTLP collector is running
- Check firewall settings
- Verify the endpoint URL is correct

### High memory usage
- Reduce batch size in collector config
- Increase export interval
- Consider sampling strategies