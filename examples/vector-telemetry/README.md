# Vector Telemetry Collection for MCP Workflows

This directory contains examples of using [Vector](https://vector.dev) to collect and process OpenTelemetry data from the MCP server.

## What is Vector?

Vector is a high-performance observability data pipeline that can:
- Collect logs, metrics, and traces from various sources
- Transform and enrich data in real-time
- Route data to multiple destinations

## Files in this Directory

### `vector-simple.toml`
Simple Vector configuration that:
- Receives OTLP traces on port 4318
- Formats them nicely for console output
- Shows workflow and step execution with timing

### `vector.toml`
Advanced configuration with:
- Multiple output formats (JSON and text)
- Span filtering and transformation
- File output for analysis
- Metrics generation

### `run-with-vector.py`
Python script that:
- Starts Vector automatically
- Executes a workflow through MCP
- Shows real-time telemetry output

### `docker-compose.yml`
Docker setup for running Vector without installation

## Quick Start

### Option 1: Using Docker (Easiest)

```bash
# Start Vector in Docker
docker-compose up -d

# Run a workflow (Vector will collect telemetry)
python ../../workflow_streaming_simple.py

# View Vector logs
docker-compose logs -f vector
```

### Option 2: Install Vector Locally

#### Windows (using Scoop)
```powershell
scoop install vector
```

#### macOS (using Homebrew)
```bash
brew install vectordotdev/brew/vector
```

#### Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.vector.dev | bash
```

Then run:
```bash
# Start Vector with simple config
vector --config vector-simple.toml

# In another terminal, run the example
python run-with-vector.py
```

### Option 3: Manual Setup

1. Start Vector manually:
```bash
vector --config vector-simple.toml
```

2. Run any MCP workflow with telemetry:
```bash
# Set the OTLP endpoint to Vector
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318

# Run your workflow
python ../../workflow_streaming_simple.py
```

## How It Works

1. **MCP Server** (built with `--features telemetry`) sends OpenTelemetry data
2. **Vector** receives OTLP traces on port 4318
3. **Transforms** parse and format the span data
4. **Sinks** output to console, files, or other destinations

## Example Output

When running a workflow, Vector will show:

```
13:45:23.456 | 🎯 WORKFLOW START: execute_sequence | Steps: 5
13:45:23.457 |   📍 Step 1/5: delay [200ms]
13:45:23.658 |   📍 Step 2/5: get_applications [1523ms]
13:45:25.181 |   📍 Step 3/5: delay [200ms]
13:45:25.382 |   📍 Step 4/5: get_focused_window_tree [892ms]
13:45:26.274 |   📍 Step 5/5: delay [100ms]
```

## Advanced Usage

### Custom Transformations

Edit `vector.toml` to add custom processing:

```toml
[transforms.my_transform]
type = "remap"
inputs = ["otlp"]
source = '''
# Your custom logic here
.custom_field = "processed"
'''
```

### Multiple Destinations

Send telemetry to multiple places:

```toml
[sinks.elasticsearch]
type = "elasticsearch"
inputs = ["format"]
endpoints = ["http://localhost:9200"]

[sinks.prometheus]
type = "prometheus_exporter"
inputs = ["metrics"]
address = "0.0.0.0:9090"
```

### Filtering

Only process specific spans:

```toml
[transforms.filter_tools]
type = "filter"
inputs = ["otlp"]
condition = '.attributes."tool.name" == "screenshot"'
```

## Troubleshooting

### Vector doesn't receive data
- Check MCP server was built with telemetry: `cargo build --release --features telemetry`
- Verify OTEL_EXPORTER_OTLP_ENDPOINT is set to `http://localhost:4318`
- Check Vector is listening: `netstat -an | grep 4318`

### Vector shows errors
- Check Vector logs: `vector --config vector.toml -vv`
- Verify config syntax: `vector validate vector.toml`

### No output visible
- Ensure console sink is configured
- Check transforms aren't filtering out all data
- Try the simpler `vector-simple.toml` config first

## Benefits of Using Vector

1. **Real-time Processing** - See telemetry as it happens
2. **Flexible Routing** - Send to multiple destinations
3. **Data Transformation** - Enrich and format data
4. **High Performance** - Handles high volume efficiently
5. **No Vendor Lock-in** - Works with any observability stack

## Next Steps

- Explore Vector's [documentation](https://vector.dev/docs/)
- Try sending data to Elasticsearch, Datadog, or other sinks
- Build dashboards with the collected telemetry
- Set up alerts based on workflow performance