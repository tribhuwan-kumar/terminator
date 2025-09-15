const WS_URL = "ws://127.0.0.1:17373";
let socket = null;
let reconnectTimer = null;

// Simple Set to track what we've attached to
const attachedTabs = new Set();
// Track which tabs have Runtime and Log domains enabled
const enabledTabs = new Set();

// Restore on startup
chrome.storage.session.get('attached').then(data => {
  if (data.attached) data.attached.forEach(id => attachedTabs.add(id));
  log(`Restored ${attachedTabs.size} attached debugger sessions`);
});

// Logging is disabled by default to reduce console noise. Toggle via Service Worker console.
let debugEnabled = true;
self.enableTerminatorDebug = () => {
  debugEnabled = true;
};
self.disableTerminatorDebug = () => {
  debugEnabled = false;
};
function log(...args) {
  if (!debugEnabled) return;
  console.log("[TerminatorBridge]", ...args);
}

// Exponential backoff to reduce repeated connection error spam
const BASE_RECONNECT_DELAY_MS = 500; // faster initial retry
const MAX_RECONNECT_DELAY_MS = 3000; // cap retries to 3s to align with host waiting
let currentReconnectDelayMs = BASE_RECONNECT_DELAY_MS;

function connect() {
  try {
    if (
      socket &&
      (socket.readyState === WebSocket.OPEN ||
        socket.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }
  } catch (_) {}
  try {
    socket = new WebSocket(WS_URL);
  } catch (e) {
    log("WebSocket construct error", e);
    scheduleReconnect();
    return;
  }

  socket.onopen = () => {
    log("Connected to", WS_URL);
    // Reset backoff on successful connection
    currentReconnectDelayMs = BASE_RECONNECT_DELAY_MS;
    socket.send(JSON.stringify({ type: "hello", from: "extension" }));
  };

  socket.onclose = () => {
    log("Socket closed");
    scheduleReconnect();
  };

  socket.onerror = (e) => {
    log("Socket error", e);
    try {
      socket.close();
    } catch (_) {}
  };

  socket.onmessage = async (event) => {
    let msg;
    try {
      msg = JSON.parse(event.data);
    } catch (e) {
      log("Invalid JSON", event.data);
      return;
    }
    if (!msg || !msg.action) return;

    if (msg.action === "eval") {
      const { id, code, awaitPromise = true } = msg;
      try {
        const tabId = await getActiveTabId();
        const result = await evalInTab(tabId, code, awaitPromise, id);
        safeSend({ id, ok: true, result });
      } catch (err) {
        safeSend({ id, ok: false, error: String(err && (err.message || err)) });
      }
    } else if (msg.action === "ping") {
      safeSend({ type: "pong" });
    }
  };
}

function ensureConnected() {
  try {
    if (
      !socket ||
      (socket.readyState !== WebSocket.OPEN &&
        socket.readyState !== WebSocket.CONNECTING)
    ) {
      connect();
    }
  } catch (_) {
    connect();
  }
}

// Ensure we have an active WS connection on first message from content script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  try {
    if (!message || message.type !== "terminator_content_handshake") return;
    log("Received handshake from content script", {
      tab: sender.tab && sender.tab.id,
    });
    // Kick the connector if not already connected; otherwise noop
    ensureConnected();
    sendResponse({ ok: true });
  } catch (e) {
    try {
      sendResponse({ ok: false, error: String(e && (e.message || e)) });
    } catch (_) {}
    // swallow
  }
  // Keep listener alive for async sendResponse
  return true;
});

// Inject a lightweight handshake into the active tab when it changes/updates
async function injectHandshake(tabId) {
  try {
    if (typeof tabId !== "number") return;
    await chrome.scripting.executeScript({
      target: { tabId },
      func: () => {
        try {
          chrome.runtime.sendMessage({ type: "terminator_content_handshake" });
        } catch (_) {}
      },
      world: "ISOLATED",
    });
  } catch (e) {
    // ignore (e.g., not permitted on special pages)
  }
}

async function getActiveTabIdSafe() {
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      lastFocusedWindow: true,
    });
    return tab && tab.id != null ? tab.id : null;
  } catch (_) {
    return null;
  }
}

// Additional event-based triggers to wake the worker and maintain connection
chrome.runtime.onInstalled.addListener(() => {
  log("onInstalled → ensureConnected");
  ensureConnected();
});
chrome.runtime.onStartup.addListener(() => {
  log("onStartup → ensureConnected");
  ensureConnected();
});
chrome.webNavigation.onCommitted.addListener(() => {
  ensureConnected();
});
chrome.alarms.clear("terminator_keepalive");
chrome.alarms.create("terminator_keepalive", { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm && alarm.name === "terminator_keepalive") {
    ensureConnected();
  }
});
chrome.tabs.onActivated.addListener(async () => {
  ensureConnected();
  const tabId = await getActiveTabIdSafe();
  if (tabId != null) injectHandshake(tabId);
});
chrome.tabs.onUpdated.addListener(async (tabId, changeInfo) => {
  if (
    changeInfo &&
    (changeInfo.status === "loading" || changeInfo.status === "complete")
  ) {
    ensureConnected();
    injectHandshake(tabId);
  }
});

// Clean up our tracking when tab closes
chrome.tabs.onRemoved.addListener(tabId => {
  if (attachedTabs.has(tabId)) {
    attachedTabs.delete(tabId);
    enabledTabs.delete(tabId);  // Also clean enabled domains state
    chrome.storage.session.set({attached: [...attachedTabs]});
    log(`Tab ${tabId} closed, removed from attached tabs and enabled domains`);
  }
});

// Clean up when debugger is manually detached (user clicked Cancel)
chrome.debugger.onDetach.addListener((source, reason) => {
  const tabId = source.tabId;
  if (attachedTabs.has(tabId)) {
    attachedTabs.delete(tabId);
    enabledTabs.delete(tabId);  // Also clean enabled domains state
    chrome.storage.session.set({attached: [...attachedTabs]});
    log(`Debugger detached from tab ${tabId}, reason: ${reason}`);
  }
});

function scheduleReconnect() {
  if (reconnectTimer) return;
  const delay = currentReconnectDelayMs;
  currentReconnectDelayMs = Math.min(
    currentReconnectDelayMs * 2,
    MAX_RECONNECT_DELAY_MS
  );
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    log(`Reconnecting... (delay=${delay}ms)`);
    connect();
  }, delay);
}

function safeSend(obj) {
  try {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(obj));
    }
  } catch (e) {
    log("Failed to send", e);
  }
}

async function getActiveTabId() {
  const [tab] = await chrome.tabs.query({
    active: true,
    lastFocusedWindow: true,
  });
  if (!tab || tab.id == null) throw new Error("No active tab");
  return tab.id;
}

function formatRemoteObject(obj) {
  try {
    if (obj === null || obj === undefined) return null;
    if (Object.prototype.hasOwnProperty.call(obj, "value")) return obj.value;
    if (obj.description !== undefined) return obj.description;
    return obj.type || null;
  } catch (_) {
    return null;
  }
}

async function evalInTab(tabId, code, awaitPromise, evalId) {
  const perfStart = performance.now();
  const timings = {};
  
  // Only attach if we haven't before
  if (!attachedTabs.has(tabId)) {
    const attachStart = performance.now();
    try {
      await debuggerAttach(tabId);
      attachedTabs.add(tabId);
      // Persist (fire and forget)
      chrome.storage.session.set({attached: [...attachedTabs]});
      timings.attach = performance.now() - attachStart;
      log(`Debugger attached to tab ${tabId} (${timings.attach.toFixed(1)}ms)`);
    } catch (e) {
      // Don't care if already attached, just log it
      log(`Could not attach to tab ${tabId}:`, e.message);
    }
  } else {
    log(`Reusing existing debugger for tab ${tabId}`);
    timings.attach = 0;
  }
  
  let onEvent = null;
  try {
    // Only enable domains if not already enabled
    if (!enabledTabs.has(tabId)) {
      const enableStart = performance.now();
      try {
        const runtimeStart = performance.now();
        await sendCommand(tabId, "Runtime.enable", {});
        timings.runtimeEnable = performance.now() - runtimeStart;
        
        const logStart = performance.now();
        await sendCommand(tabId, "Log.enable", {});
        timings.logEnable = performance.now() - logStart;
        
        enabledTabs.add(tabId);
        timings.totalEnable = performance.now() - enableStart;
        log(`Domains enabled for tab ${tabId} - Runtime: ${timings.runtimeEnable.toFixed(1)}ms, Log: ${timings.logEnable.toFixed(1)}ms, Total: ${timings.totalEnable.toFixed(1)}ms`);
      } catch (e) {
        log(`Could not enable domains for tab ${tabId}:`, e.message);
      }
    } else {
      log(`Reusing enabled domains for tab ${tabId}`);
      timings.runtimeEnable = 0;
      timings.logEnable = 0;
      timings.totalEnable = 0;
    }

    // Listen for console/log/exception events for this tab while the eval runs
    onEvent = (source, method, params) => {
      try {
        if (!source || source.tabId !== tabId) return;
        if (method === "Runtime.consoleAPICalled") {
          const level = params.type || "log";
          const args = (params.args || []).map((a) => formatRemoteObject(a));
          const stackTrace = params.stackTrace || null;
          safeSend({
            type: "console_event",
            id: evalId,
            level,
            args,
            stackTrace,
            ts: params.timestamp || Date.now(),
          });
        } else if (method === "Runtime.exceptionThrown") {
          safeSend({
            type: "exception_event",
            id: evalId,
            details: params.exceptionDetails || params || null,
          });
        } else if (method === "Log.entryAdded") {
          safeSend({
            type: "log_event",
            id: evalId,
            entry: params.entry || params || null,
          });
        }
      } catch (e) {
        // swallow
      }
    };
    chrome.debugger.onEvent.addListener(onEvent);
    
    const evalStart = performance.now();
    const { result, exceptionDetails } = await sendCommand(
      tabId,
      "Runtime.evaluate",
      {
        expression: code,
        awaitPromise: !!awaitPromise,
        returnByValue: true,
        userGesture: true,
      }
    );
    timings.evaluate = performance.now() - evalStart;
    
    if (exceptionDetails) {
      // Build rich error details for MCP side
      const details = {
        text: exceptionDetails.text,
        url: exceptionDetails.url,
        lineNumber: exceptionDetails.lineNumber,
        columnNumber: exceptionDetails.columnNumber,
        exception:
          (exceptionDetails.exception &&
            (exceptionDetails.exception.description ||
              exceptionDetails.exception.value)) ||
          null,
        stackTrace:
          (exceptionDetails.stackTrace &&
            Array.isArray(exceptionDetails.stackTrace.callFrames) &&
            exceptionDetails.stackTrace.callFrames.map((cf) => ({
              functionName: cf.functionName,
              url: cf.url,
              lineNumber: cf.lineNumber,
              columnNumber: cf.columnNumber,
            }))) ||
          null,
      };
      // Emit console error for visibility in extension worker logs
      console.error("[TerminatorBridge] Eval exception:", details);
      // Throw a JSON-encoded error so the bridge returns full context
      throw new Error(
        JSON.stringify({
          code: "EVAL_ERROR",
          message: details.text || "Evaluation error",
          details,
        })
      );
    }
    
    // Log timing summary
    timings.total = performance.now() - perfStart;
    log(`[TIMING] Total: ${timings.total.toFixed(1)}ms | Attach: ${timings.attach.toFixed(1)}ms | Enable: ${timings.totalEnable.toFixed(1)}ms | Eval: ${timings.evaluate.toFixed(1)}ms`);
    
    // Return JSON-serializable value
    return result?.value;
  } finally {
    try {
      // Best-effort: remove onEvent listener
      if (
        onEvent &&
        chrome &&
        chrome.debugger &&
        chrome.debugger.onEvent &&
        chrome.debugger.onEvent.removeListener
      ) {
        chrome.debugger.onEvent.removeListener(onEvent);
      }
      // DON'T DISABLE DOMAINS - Keep them enabled for reuse
      // Removed: await sendCommand(tabId, "Log.disable", {});
      // Removed: await sendCommand(tabId, "Runtime.disable", {});
    } catch (_) {}
    // DON'T DETACH - Keep debugger attached for reuse
    // This was: await debuggerDetach(tabId);
  }
}

function debuggerAttach(tabId) {
  return new Promise((resolve, reject) => {
    chrome.debugger.attach({ tabId }, "1.3", (err) => {
      if (chrome.runtime.lastError) return reject(chrome.runtime.lastError);
      resolve();
    });
  });
}

function debuggerDetach(tabId) {
  return new Promise((resolve) => {
    chrome.debugger.detach({ tabId }, () => resolve());
  });
}

function sendCommand(tabId, method, params) {
  return new Promise((resolve, reject) => {
    chrome.debugger.sendCommand({ tabId }, method, params, (result) => {
      const err = chrome.runtime.lastError;
      if (err) return reject(err);
      resolve(result || {});
    });
  });
}

connect();
