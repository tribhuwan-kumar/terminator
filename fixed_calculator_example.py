import asyncio
import terminator

async def run_calculator():
    """Fixed Windows Calculator automation example"""
    desktop = terminator.Desktop(log_level="error")
    
    try:
        print("Opening Calculator...")
        # Use calc.exe instead of UWP identifier for better compatibility
        calculator = desktop.open_application("calc.exe")
        await asyncio.sleep(2)  # Allow app to open
        
        # Get display element - note the await for first()
        display_element = calculator.locator("nativeid:CalculatorResults")
        
        # Get buttons - note we await the first() calls
        button_1 = await calculator.locator("Name:One").first()
        button_plus = await calculator.locator("Name:Plus").first()
        button_2 = await calculator.locator("Name:Two").first()
        button_equals = await calculator.locator("Name:Equals").first()
        
        # Get initial display text
        print("Getting initial display text...")
        try:
            element = await display_element.first()
            text = element.name()
            print(f"Initial display: {text}")
        except Exception as e:
            print(f"Warning: Could not get initial display text: {e}")
        
        # Perform calculation: 1 + 2 =
        print("Performing calculation: 1 + 2 =")
        
        button_1.click()
        await asyncio.sleep(0.5)
        
        button_plus.click()
        await asyncio.sleep(0.5)
        
        button_2.click()
        await asyncio.sleep(0.5)
        
        button_equals.click()
        await asyncio.sleep(1.0)
        
        # Get and verify result
        print("Getting result...")
        try:
            element = await display_element.first()
            result = element.name()
            print(f"Result: {result}")
            
            if "3" in result:
                print("Calculation successful! Result contains '3'")
            else:
                print(f"Unexpected result: {result}")
        except Exception as e:
            print(f"Could not get result: {e}")
        
        # Additional info about the equals button
        print("\nEquals button info:")
        print(f"  Visible: {button_equals.is_visible()}")
        attrs = button_equals.attributes()
        # attrs is a UIElementAttributes object, access properties directly
        print(f"  Name: {attrs.name if hasattr(attrs, 'name') else 'N/A'}")
        print(f"  Role: {attrs.role if hasattr(attrs, 'role') else 'N/A'}")
        
    except terminator.PlatformError as e:
        print(f"Platform Error: {e}")
    except Exception as e:
        print(f"Error: {type(e).__name__}: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(run_calculator())