"""
Craigslist automation example using terminator.py
This script navigates to Craigslist, searches for items, and extracts listing data
"""

# The terminator Python engine automatically provides:
# - desktop object for automation
# - env dictionary with environment variables
# - sleep(ms) helper function
# - log() helper function

async def main():

    log("Starting Craigslist automation...")

    # Open browser and navigate to Craigslist
    log("Opening browser...")
    await desktop.open_application("chrome")
    await sleep(2000)

    # Navigate to Craigslist (San Francisco for example)
    log("Navigating to Craigslist...")
    browser_window = await desktop.locator("role:Window|name:Chrome").first()

    if browser_window:
        # Click address bar and type URL
        address_bar = await desktop.locator("role:Edit|name:Address").first()
        if address_bar:
            await address_bar.click()
            await address_bar.type_text("https://sfbay.craigslist.org")
            await address_bar.press_key("{Enter}")
            await sleep(3000)

        # Search for items (e.g., furniture)
        log("Searching for furniture listings...")
        search_box = await desktop.locator("role:Edit|name:search").first()
        if not search_box:
            # Try alternative selector
            search_box = await desktop.locator("role:Edit").first()

        if search_box:
            await search_box.click()
            await search_box.type_text("furniture")
            await search_box.press_key("{Enter}")
            await sleep(3000)

            # Extract listing data from the page
            log("Extracting listing data...")
            listings_data = []

            # Find listing items (usually links or groups)
            listings = await desktop.locator("role:Hyperlink").all()

            # Extract data from first 10 listings
            for i, listing in enumerate(listings[:10]):
                try:
                    listing_text = await listing.name()
                    if listing_text and len(listing_text) > 10:  # Filter out navigation links
                        listing_info = {
                            "index": i + 1,
                            "title": listing_text,
                            "extracted": True
                        }
                        listings_data.append(listing_info)
                        log(f"Found listing {i+1}: {listing_text[:50]}...")
                except Exception as e:
                    log(f"Error extracting listing {i+1}: {e}")

            # Return extracted data
            result = {
                "status": "success",
                "listings_found": len(listings_data),
                "listings": listings_data,
                "search_term": "furniture",
                "location": "San Francisco Bay Area"
            }

            log(f"Extraction complete! Found {len(listings_data)} listings")

            # Set environment variables for workflow
            return {
                "set_env": {
                    "listings_count": len(listings_data),
                    "search_completed": True
                },
                "result": result
            }
        else:
            log("Could not find search box")
            return {
                "status": "error",
                "message": "Search box not found"
            }
    else:
        log("Browser window not found")
        return {
            "status": "error",
            "message": "Browser window not found"
        }

# The Python engine expects the script to return a value
# It wraps the code and handles async execution
result = await main()
return result