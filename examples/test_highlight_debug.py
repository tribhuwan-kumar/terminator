#!/usr/bin/env python3
"""
Debug test for highlighting - tests if the highlight function works directly
"""

import asyncio
import time
from terminator import Desktop, TextPosition, FontStyle

async def test_highlight_directly():
    """Test highlighting directly on a UI element to debug the border issue"""
    print("🔍 Direct Highlighting Debug Test")
    print("=" * 50)
    
    # Initialize desktop
    desktop = Desktop(False, False)
    print("✅ Desktop initialized")
    
    # Find an element to highlight - let's find a window
    print("\n📋 Looking for a window to highlight...")
    apps = desktop.applications()
    
    if not apps:
        print("❌ No applications found")
        return
        
    # Find a suitable window
    target_app = None
    for app in apps:
        if app.attributes().name and "Chrome" in app.attributes().name:
            target_app = app
            break
    
    if not target_app:
        # Just use the first available app
        target_app = apps[0]
    
    app_name = target_app.attributes().name or "Unknown"
    print(f"📍 Found application: {app_name}")
    
    # Get the first window or control
    children = target_app.children()
    if children:
        element = children[0]
        print(f"📍 Found element: {element.role()}")
    else:
        element = target_app
        print(f"📍 Using app itself as element")
    
    # Test 1: Text only (should work based on user report)
    print("\n🧪 Test 1: Text overlay only (no border)")
    handle1 = element.highlight(
        None,  # No color means no border
        3000,  # duration_ms
        "TEXT ONLY",  # text
        TextPosition.top(),  # text_position
        FontStyle(20, True, 0xFFFFFF)  # font_style: size, bold, color (white)
    )
    print("   ✅ Should see WHITE text on BLACK background")
    await asyncio.sleep(3.5)
    
    # Test 2: Border only (no text)
    print("\n🧪 Test 2: Border only (no text)")
    handle2 = element.highlight(
        0x0000FF,  # Red border (BGR format)
        3000,      # duration_ms
        None,      # No text
        None,      # No text_position
        None       # No font_style
    )
    print("   🔴 Should see RED border around element")
    await asyncio.sleep(3.5)
    
    # Test 3: Both text and border
    print("\n🧪 Test 3: Both text and border")
    handle3 = element.highlight(
        0x0000FF,  # Red border
        3000,      # duration_ms
        "BOTH",    # text
        TextPosition.top(),  # text_position
        FontStyle(20, True, 0xFFFFFF)  # font_style
    )
    print("   🔴 Should see RED border AND white text")
    await asyncio.sleep(3.5)
    
    # Test 4: Different colors
    print("\n🧪 Test 4: Green border with yellow text")
    handle4 = element.highlight(
        0x00FF00,  # Green border (BGR format)
        3000,      # duration_ms
        "GREEN",   # text
        TextPosition.top(),  # text_position
        FontStyle(20, True, 0x00FFFF)  # Yellow text
    )
    print("   🟢 Should see GREEN border with YELLOW text")
    await asyncio.sleep(3.5)
    
    # Test 5: Longer duration
    print("\n🧪 Test 5: Long duration (5 seconds)")
    handle5 = element.highlight(
        0xFF0000,  # Blue border (BGR format)
        5000,      # duration_ms
        "5 SECONDS",  # text
        TextPosition.top(),  # text_position
        FontStyle(20, True, 0xFFFFFF)  # White text
    )
    print("   🔵 Should see BLUE border for 5 seconds")
    await asyncio.sleep(5.5)
    
    print("\n✅ All tests complete!")
    print("\nDiagnostic Questions:")
    print("1. Did you see the white text in Test 1? (Expected: YES)")
    print("2. Did you see the red border in Test 2? (Expected: YES)")
    print("3. Did you see both in Test 3? (Expected: YES)")
    print("4. Were the colors correct in Test 4? (Expected: YES)")
    print("5. Did Test 5 last 5 seconds? (Expected: YES)")
    
if __name__ == "__main__":
    asyncio.run(test_highlight_directly())
