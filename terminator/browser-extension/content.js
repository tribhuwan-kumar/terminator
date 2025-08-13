// Minimal handshake to wake the MV3 service worker and ensure the WS connect loop runs.
// Fires at document_start on all pages.
(function () {
  try {
    chrome.runtime.sendMessage({ type: "terminator_content_handshake" }, () => {
      // ignore response
    });
  } catch (_) {
    // Ignore if not allowed on special pages
  }
})();
