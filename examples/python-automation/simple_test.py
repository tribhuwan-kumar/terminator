# Simple test of Python bindings
log("Testing Python bindings...")

# Test with async/await
log("Getting desktop object...")
try:
    result = desktop.locator("role:Window")
    log(f"Locator created: {result}")

    # Try to get all windows - need to await
    windows = await result.all()
    log(f"Windows found: {len(windows) if windows else 0}")
    
    return {
        "status": "success",
        "window_count": len(windows) if windows else 0,
        "message": "Python bindings test complete"
    }
except Exception as e:
    log(f"Error: {e}")
    import traceback
    log(traceback.format_exc())
    return {
        "status": "error",
        "error": str(e),
        "message": "Python bindings test failed"
    }