# Craigslist Automation Example

This example demonstrates how to use the Terminator MCP Python engine to automate web scraping tasks on Craigslist.

## Files

- **`craigslist_scraper.py`** - Python script that uses terminator.py bindings to automate browser interactions and extract listing data
- **`craigslist_workflow.yml`** - YAML workflow that executes the Python script via the MCP server's `run_command` tool with `engine: python`

## Features

- Browser automation using terminator.py
- Navigation to Craigslist website
- Search functionality automation
- Data extraction from listings
- Environment variable support for customization
- Error handling and troubleshooting steps

## How to Run

### Using the workflow file:
```bash
terminator mcp run examples/python-craigslist-automation/craigslist_workflow.yml --command "npx -y terminator-mcp-agent"
```

### With custom parameters:
```bash
terminator mcp run examples/python-craigslist-automation/craigslist_workflow.yml \
  --command "npx -y terminator-mcp-agent" \
  --inputs '{"search_term": "laptops", "max_listings": 20}'
```

## Python Engine Details

The Python engine in Terminator MCP:
- Automatically imports `terminator` module
- Provides `desktop` object for UI automation
- Supports async/await syntax
- Includes helper functions: `sleep(ms)` and `log()`
- Can return data via `set_env` for subsequent workflow steps
- Supports both inline Python code and external script files

## Example Output

The script extracts:
- Listing titles
- Number of listings found
- Search parameters used
- Location information

Results can be:
- Returned as JSON
- Saved to files
- Passed to subsequent workflow steps via environment variables

## Customization

Modify the variables in `craigslist_workflow.yml`:
- `search_term` - What to search for
- `location_url` - Which Craigslist regional site to use
- `max_listings` - How many listings to extract