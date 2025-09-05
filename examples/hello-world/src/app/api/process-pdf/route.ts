import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { generateText } from 'ai';
import { ApiError, DesktopUseClient, Locator, sleep } from "desktop-use";
import fs from "fs/promises";
import { NextRequest, NextResponse } from "next/server";
import os from "os";
import path from "path";
import { z } from 'zod';

export const runtime = "nodejs";

// --- Configuration --- //
const PDF_URL = "https://raw.githubusercontent.com/py-pdf/sample-files/main/001-trivial/minimal-document.pdf";
const DOWNLOAD_DIR = path.join(os.tmpdir(), "pdf_workflow_downloads");
const PDF_FILENAME = "downloaded_minimal_document.pdf";
const PDF_PATH = path.join(DOWNLOAD_DIR, PDF_FILENAME);
const APP_WINDOW_TITLE = "Terminator Workflows"; // From layout.tsx

// --- Locators --- //
// !! These WILL likely need adjustment for your specific PDF reader and OS !!
// Try using Accessibility Insights or similar tool to find reliable locators
// Adjust this locator based on the actual structure of your PDF reader
// A broad locator like 'role:document' might work, or you might need something more specific.
const PDF_TEXT_AREA_LOCATOR = `window`;
const PDF_CLOSE_BUTTON_LOCATOR = `Name:Close`; // Common name for close buttons

// Define the structure we want the AI to output
const spreadsheetSchema = z.object({
  columns: z.array(z.string()).describe("The column headers"),
  rows: z.array(z.array(z.string())).describe("The data rows, where each inner array corresponds to a row"),
});

export async function POST(request: NextRequest) {
  console.log("api route /api/process-pdf called");

  const client = new DesktopUseClient();
  let pdfWindowLocator: Locator | null = null; // To store locator for closing later
  let apiKey: string | undefined = undefined;

  try {
    // Parse API key from request body
    const body = await request.json();
    apiKey = body?.apiKey;

    if (!apiKey || typeof apiKey !== 'string') {
      console.error("API key missing or invalid in request body.");
      return NextResponse.json({ message: "Gemini API Key is required in the request body." }, { status: 400 });
    }

    // Initialize AI client with the provided key *inside* the try block
    const google = createGoogleGenerativeAI({ apiKey });
    const model = google('models/gemini-2.0-flash-001');
    console.log("AI client initialized with provided API key.");

    // Determine default PDF reader (Windows only for now)
    let pdfWindowLocatorString = `window`;

    // 0. Ensure download directory exists
    await fs.mkdir(DOWNLOAD_DIR, { recursive: true });
    console.log(`ensured download directory exists: ${DOWNLOAD_DIR}`);

    // 1. Download the PDF
    console.log(`downloading pdf from: ${PDF_URL} to ${PDF_PATH}`);
    const downloadResult = await client.run(
      `curl -L "${PDF_URL}" -o "${PDF_PATH}" --create-dirs`
    );
    console.log("download command result:", downloadResult);
    if (downloadResult.exit_code !== 0) {
      throw new Error(`failed to download PDF. Exit code: ${downloadResult.exit_code}. stderr: ${downloadResult.stderr}`);
    }
    console.log(`pdf downloaded successfully.`);
    await sleep(1000);

    // 2. Open the downloaded PDF
    console.log(`attempting to open pdf: ${PDF_PATH}`);
    await client.openFile(PDF_PATH);
    await sleep(5000); // Wait for PDF reader to potentially load

    // 3. Locate the PDF reader window and activate it
    console.log(`attempting to locate and activate pdf reader window using locator: ${pdfWindowLocatorString}`);
    pdfWindowLocator = client.locator(pdfWindowLocatorString); // Use dynamic locator string
    // await client.activateApplication(APP_WINDOW_TITLE); // Bring window to front
    // await pdfWindowLocator.expectVisible(15000); // Wait longer for window

    console.log("pdf reader window located and activated.");
    await sleep(1000);

    // 4. Extract text data using locators
    console.log(`attempting to extract text using locator: ${PDF_TEXT_AREA_LOCATOR}`);
    // Use the window locator as the base for the text area locator
    const textResponse = await pdfWindowLocator.getText(10); // Use default depth
    const extractedData = textResponse?.text;
    if (!extractedData) {
        throw new Error(`Failed to extract text using locator: ${PDF_TEXT_AREA_LOCATOR}`);
    }
    console.log("Extracted data (raw):", extractedData); // Log snippet
    await sleep(500);

    // 5. Process data with AI
    console.log("attempting to process extracted data with AI...");
    const result = await generateText({
        model, // Use the model initialized with the request-specific key
        prompt: `Extract the tabular data from the following text content obtained from a PDF. Structure the output as a simple spreadsheet with columns and rows. Text Content:\n\n${extractedData}`,
        tools: {
            spreadsheet: {
                description: 'Tool to format extracted data into a spreadsheet structure.',
                parameters: spreadsheetSchema
            }
        },
        toolChoice: 'required', // Force it to use the tool
    });

    // Check if the tool call was successful and get the result
    const toolCall = result.toolCalls.find((tc: any) => tc.toolName === 'spreadsheet');
    if (!toolCall || !toolCall.args) {
         throw new Error("AI failed to generate spreadsheet data.");
    }
    const spreadsheetData = toolCall.args;

    // Validate the output against the schema (optional but good practice)
    try {
        spreadsheetSchema.parse(spreadsheetData);
         console.log("ai processing complete, data structured.");
         console.log("Spreadsheet data:", JSON.stringify(spreadsheetData, null, 2));
    } catch (validationError) {
        console.error("AI output validation failed:", validationError);
        throw new Error("AI generated data in an unexpected format.");
    }
    await sleep(500);

    // 7. Switch back to the app's browser tab
    // try {
    //     console.log(`attempting to switch back to app window: ${APP_WINDOW_TITLE}`);
    //     await client.activateBrowserWindowByTitle(APP_WINDOW_TITLE);
    //     console.log("switched back to app window.");
    //     await sleep(500);
    // } catch (switchError) {
    //     console.warn(`failed to switch back to the app window: ${switchError instanceof Error ? switchError.message : switchError}.`);
    //     // Don't fail the whole process if switching fails
    // }

    return NextResponse.json({
      message: "pdf processing workflow completed.",
      extractedData: extractedData.substring(0, 500) + (extractedData.length > 500 ? "..." : ""), // Return snippet
      spreadsheetData: spreadsheetData
    });

  } catch (error: any) {
    console.error("pdf processing automation error:", error);
    let errorMessage = "an unknown error occurred during pdf processing.";
    if (error instanceof ApiError) {
      errorMessage = `terminator api error: ${error.message} (Status: ${error.status ?? 'N/A'})`;
    } else if (error instanceof Error) {
      errorMessage = `automation script error: ${error.message}`;
    }

    // Clean up downloaded file on error if it exists
    try {
        await fs.access(PDF_PATH); // Check if file exists before unlinking
        await fs.unlink(PDF_PATH);
        console.log(`cleaned up downloaded file: ${PDF_PATH}`);
    } catch (cleanupError: any) {
         // Ignore 'ENOENT' (file not found) errors, log others
        if (cleanupError.code !== 'ENOENT') {
             console.error(`failed to cleanup downloaded file ${PDF_PATH}:`, cleanupError);
        }
    }
    return NextResponse.json({ message: errorMessage }, { status: 500 });
  }
} 