pub const TERMINATOR_JS: &str = r#"
/**
 * Terminator Helper Library – available automatically in all run_javascript scripts.
 * These helpers are thin wrappers around the underlying MCP tools and return the
 * parsed JSON result from the tool call.
 */
function __toJson(val) { return JSON.parse(val); }

function click(selector) {
  return __toJson(callTool("click_element", JSON.stringify({ selector })));
}

function typeText(selector, text) {
  return __toJson(callTool(
    "type_into_element",
    JSON.stringify({ selector, text_to_type: text })
  ));
}

function delay(ms) {
  return __toJson(callTool("delay", JSON.stringify({ delay_ms: ms })));
}

function pressKey(selector, key) {
  return __toJson(callTool(
    "press_key",
    JSON.stringify({ selector, key })
  ));
}

function waitFor(selector, condition, timeout_ms) {
  return __toJson(callTool(
    "wait_for_element",
    JSON.stringify({ selector, condition, timeout_ms })
  ));
}

/**
 * Example high-level helper: click a button and wait for disappearance
 */
function clickAndWaitDisappear(selector, timeout_ms) {
  const res = click(selector);
  waitFor(selector, "exists", timeout_ms || 2000);
  return res;
}
"#;