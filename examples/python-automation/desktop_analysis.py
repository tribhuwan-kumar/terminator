# Desktop Analysis Script
# Analyzes current desktop state and returns information about windows and applications

log("Starting desktop analysis...")

# Get all windows
windows = await desktop.locator("role:Window").all()
log(f"Found {len(windows)} windows")

# Analyze windows
window_data = []
for window in windows[:10]:  # Limit to first 10 windows
    try:
        name = window.name()
        if name:
            # Get window properties
            is_enabled = window.is_enabled()
            is_focused = window.is_focused()

            window_info = {
                "name": name,
                "enabled": is_enabled,
                "focused": is_focused
            }
            window_data.append(window_info)

            if is_focused:
                log(f"Focused window: {name}")
    except Exception as e:
        log(f"Error processing window: {e}")

# Group windows by application
apps = {}
for win in window_data:
    # Extract app name from window title (simple heuristic)
    app_name = win["name"].split(" - ")[-1] if " - " in win["name"] else win["name"]
    if app_name not in apps:
        apps[app_name] = []
    apps[app_name].append(win["name"])

log(f"\nApplications found: {len(apps)} different apps")

# Count total windows (synchronously)
total_windows = len(windows) if windows else 0

# Return analysis results
return {
    "status": "success",
    "total_windows": total_windows,
    "analyzed_windows": len(window_data),
    "applications": apps,
    "focused_window": next((w["name"] for w in window_data if w.get("focused")), None),
    "window_details": window_data[:5]  # Return first 5 for detail
}