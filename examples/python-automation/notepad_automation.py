# Notepad Automation Script
# Opens Notepad, types text, and demonstrates keyboard shortcuts

log("Starting Notepad automation...")

# Open Notepad application
log("Opening Notepad...")
desktop.open_application("notepad")
await sleep(2000)

# Find the Notepad window
notepad = await desktop.locator("role:Window|name:Notepad").first()
if not notepad:
    log("Notepad window not found!")
    return {"status": "error", "message": "Could not find Notepad window"}

log("Notepad window found")

# Find the text editor area
editor = await desktop.locator("role:Edit|name:Text Editor").first()
if not editor:
    # Try alternative selector for different Notepad versions
    editor = await desktop.locator("role:Document").first()
    if not editor:
        log("Text editor not found!")
        return {"status": "error", "message": "Could not find text editor"}

log("Text editor found, typing text...")

# Type some text
editor.click()
editor.type_text("Hello from Python automation!\n")
await sleep(500)
editor.type_text("This is an automated test using Terminator MCP.\n")
await sleep(500)
editor.type_text("\nDemonstrating keyboard shortcuts:\n")
await sleep(500)

# Select all text (Ctrl+A)
log("Selecting all text...")
editor.press_key("{Control+a}")
await sleep(1000)

# Deselect and move to end
editor.press_key("{End}")
await sleep(500)

# Type more text
editor.type_text("\nAutomation complete!\n")
editor.type_text(f"Timestamp: {__import__('datetime').datetime.now().isoformat()}")

# Try to access menu (File menu)
log("Attempting to access File menu...")
file_menu = await desktop.locator("role:MenuItem|name:File").first()
if file_menu:
    file_menu.click()
    await sleep(1000)
    # Close menu
    desktop.press_key("{Escape}")
    log("File menu accessed")
else:
    log("Could not find File menu")

# Get final text content if possible
try:
    text_content = editor.value()
    log(f"Text content has {len(text_content) if text_content else 0} characters")
except:
    text_content = None
    log("Could not retrieve text content")

log("Notepad automation complete!")

return {
    "status": "success",
    "actions_performed": [
        "Opened Notepad",
        "Typed text",
        "Used keyboard shortcuts",
        "Accessed menu"
    ],
    "text_length": len(text_content) if text_content else "unknown",
    "message": "Successfully automated Notepad interaction"
}