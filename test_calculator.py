import asyncio
import terminator

async def test_calculator():
    """Test the Windows Calculator automation example from the docs"""
    try:
        # Create a Desktop instance (main entry point for automation)
        desktop = terminator.Desktop(log_level="error")
        
        print("Opening Calculator...")
        # Try different approaches to open the calculator
        try:
            # Method 1: Using UWP app ID
            calculator = desktop.open_application("uwp:Microsoft.WindowsCalculator")
        except Exception as e:
            print(f"Failed with UWP method: {e}")
            # Method 2: Try direct executable
            try:
                calculator = desktop.open_application("calc.exe")
            except Exception as e2:
                print(f"Failed with calc.exe: {e2}")
                # Method 3: Try full path
                calculator = desktop.open_application("C:\\Windows\\System32\\calc.exe")
        
        await asyncio.sleep(2)  # Allow app to open
        
        print("Calculator opened successfully!")
        
        # Try to interact with the calculator
        # Using different selector approaches
        try:
            # Try using Name selector
            button_1 = await calculator.locator("Name:One").first()
            print("Found button 1 using Name selector")
        except Exception as e:
            print(f"Name selector failed: {e}")
            # Try role-based selector
            buttons = await calculator.locator("role:button").all()
            print(f"Found {len(buttons)} buttons")
            for button in buttons[:5]:  # Print first 5 buttons
                print(f"  Button: {button.name()}")
        
        # Test basic visibility check
        if hasattr(calculator, 'is_visible'):
            print(f"Calculator visible: {calculator.is_visible()}")
        
        # Try to get window title or other attributes
        if hasattr(calculator, 'attributes'):
            attrs = calculator.attributes()
            print(f"Calculator attributes: {attrs}")
            
    except terminator.PlatformError as e:
        print(f"Platform Error: {e}")
    except AttributeError as e:
        print(f"Attribute Error (API issue): {e}")
        print("This might be the expect_visible issue mentioned in the screenshot")
    except Exception as e:
        print(f"Unexpected error: {type(e).__name__}: {e}")

if __name__ == "__main__":
    asyncio.run(test_calculator())