# Browser Automation Script
# Opens a browser, navigates to a website, and extracts information

log("Starting browser automation...")

# Open Chrome browser
log("Opening Chrome...")
await desktop.open_application("chrome")
await sleep(2000)

# Find the browser window
browser = await desktop.locator("role:Window|name:Chrome").first()
if not browser:
    log("Chrome window not found!")
    return {"status": "error", "message": "Could not find Chrome window"}

log("Chrome window found")

# Navigate to a website
log("Navigating to example.com...")
address_bar = await desktop.locator("role:Edit|name:Address").first()
if address_bar:
    await address_bar.click()
    await address_bar.type_text("https://example.com")
    await address_bar.press_key("{Enter}")
    await sleep(3000)
    log("Navigation complete")
else:
    log("Could not find address bar")
    return {"status": "error", "message": "Address bar not found"}

# Take a screenshot
try:
    screenshot_data = await browser.capture_screenshot()
    log("Screenshot captured")
    screenshot_saved = True
except:
    log("Could not capture screenshot")
    screenshot_saved = False

# Extract page information
log("Extracting page information...")
page_info = {
    "url_visited": "https://example.com",
    "browser": "Chrome",
    "screenshot_captured": screenshot_saved
}

# Try to find some text on the page
text_elements = await desktop.locator("role:Text").all()
page_text_samples = []
for elem in text_elements[:5]:  # Get first 5 text elements
    try:
        text = await elem.name()
        if text and len(text) > 5:  # Filter out very short text
            page_text_samples.append(text[:100])  # Limit to 100 chars
    except:
        pass

if page_text_samples:
    page_info["text_found"] = page_text_samples
    log(f"Found {len(page_text_samples)} text elements")

log("Browser automation complete!")

return {
    "status": "success",
    "page_info": page_info,
    "message": "Successfully automated browser interaction"
}