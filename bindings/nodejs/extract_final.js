const { Desktop } = require('.');

async function extractQuotesFinal() {
    console.log('🚀 Final quote extraction with nth selector navigation...');
    
    const desktop = new Desktop(false, false, 'info');
    
    try {
        // Click first View Details button to open modal
        const firstViewDetails = await desktop.locator('role:Text|name:View Details').first();
        firstViewDetails.invoke();
        console.log('✅ Opened first quote modal');
        await sleep(2000);

        const extractedQuotes = [];
        let quoteIndex = 1;
        
        // Loop until nth-2 text equals nth-1 text (indicating last quote)
        while (true) {
            try {
                console.log(`\n=== 📋 Processing Quote ${quoteIndex} ===`);

                // Extract content from current quote - try parent navigation first, fallback to all text
                let bestText = '';
                let bestLength = 0;
                let allTextContent = [];
                
                try {
                    // Try to find pricing text and get siblings via parent
                    const pricingText = await desktop.locator('text:Pricing:').first();
                    const parent = await pricingText.locator('..').first(); // Get parent
                    const allSiblings = await parent.locator('role:Text').all(); // Get all text siblings
                    console.log(`📄 Found ${allSiblings.length} siblings via parent navigation`);
                    
                    for (let i = 0; i < allSiblings.length; i++) {
                        try {
                            const text = allSiblings[i].text(1); // Get immediate text content
                            if (text && text.trim().length > 0) {
                                allTextContent.push(text.trim());
                            }
                        } catch (e) {
                            // Skip elements that can't provide text
                        }
                    }
                } catch (e) {
                    console.log(`⚠️  Parent navigation failed: ${e.message}, falling back to all text elements`);
                    
                    // Fallback: get all text elements in the modal
                    try {
                        const textElements = await desktop.locator('role:Text').all();
                        console.log(`📄 Found ${textElements.length} text elements via fallback`);
                        
                        for (let i = 0; i < textElements.length; i++) {
                            try {
                                const text = textElements[i].text(1); // Get immediate text content
                                if (text && text.trim().length > 0) {
                                    allTextContent.push(text.trim());
                                }
                            } catch (e) {
                                // Skip elements that can't provide text
                            }
                        }
                    } catch (e2) {
                        console.log(`⚠️  Fallback also failed: ${e2.message}`);
                    }
                }
                
                bestText = allTextContent.join(' | ');
                bestLength = bestText.length;
                
                console.log(`📄 Concatenated ${allTextContent.length} text elements`);
                console.log(`📄 Extracted ${bestLength} characters`);
                console.log(`🔍 Preview: ${bestText.substring(0, 200)}...`);
                
                extractedQuotes.push({
                    quoteIndex: quoteIndex,
                    extractedAt: new Date().toISOString(),
                    fullText: bestText,
                    textLength: bestLength
                });
                
                // Check if we're at the last quote by comparing nth-2 and nth-1 text values
                try {
                    const nthMinus3 = await desktop.locator('role:Text >> nth=-3').first();
                    const nthMinus1 = await desktop.locator('role:Text >> nth=-1').first();
                    
                    const text3 = nthMinus3.text();
                    const text1 = nthMinus1.text();
                    
                    console.log(`🔍 Checking navigation: nth=-3="${text3}" vs nth=-1="${text1}"`);
                    
                    if (text3 === text1) {
                        console.log('🏁 Reached last quote (nth-3 equals nth-1)');
                        break;
                    }
                    
                    // Navigate to next quote using nth-3 group
                    console.log('➡️  Navigating to next quote...');
                    const nextButton = await desktop.locator('role:Group >> nth=-3').first();
                    nextButton.invoke();
                    await sleep(2000);
                    
                } catch (e) {
                    console.log(`❌ Navigation check failed: ${e.message}`);
                    break;
                }
                
                quoteIndex++;
                
            } catch (error) {
                console.log(`❌ Error processing quote ${quoteIndex}: ${error.message}`);
                extractedQuotes.push({
                    quoteIndex: quoteIndex,
                    error: error.message,
                    extractedAt: new Date().toISOString()
                });
                break;
            }
        }

        // Close final modal
        try {
            const closeButton = await desktop.locator('nativeid:closePlanInfo').first();
            closeButton.click();
            console.log('✅ Closed final modal');
        } catch (e) {
            await desktop.pressKey('Escape');
        }

        const results = {
            success: true,
            timestamp: new Date().toISOString(),
            totalQuotes: extractedQuotes.length,
            quotes: extractedQuotes
        };
        
        console.log('📄 Individual quote text files saved');
        
        return results;
    } catch (error) {
        console.log(`❌ Fatal error: ${error.message}`);
        return { error: error.message };
    }
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

if (require.main === module) {
    extractQuotesFinal()
        .then(results => {
            console.log('\n📊 Final Results:');
            console.log(`Total Quotes: ${results.totalQuotes || 0}`);
            console.log(`Successful Extractions: ${results.quotes?.filter(q => q.fullText && q.textLength > 100).length || 0}`);
            
            if (results.quotes) {
                results.quotes.forEach(quote => {
                    console.log(`\n📋 Quote ${quote.quoteIndex}:`);
                    if (quote.fullText && quote.textLength > 100) {
                        console.log(`  ✅ Success: ${quote.textLength} characters`);
                        console.log(`  Preview: "${quote.fullText.substring(0, 100)}..."`);
                    } else if (quote.error) {
                        console.log(`  ❌ Error: ${quote.error}`);
                    } else {
                        console.log(`  ⚠️  Short content: ${quote.textLength || 0} characters`);
                    }
                });
            }
        })
        .catch(error => {
            console.error('❌ Script failed:', error);
            process.exit(1);
        });
}

module.exports = { extractQuotesFinal }; 