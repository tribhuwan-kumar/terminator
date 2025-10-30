#!/usr/bin/env tsx
/**
 * Finance Data Entry Workflow
 *
 * Demonstrates:
 * - Creating sample finance data in Notepad
 * - Opening Excel and creating a blank workbook
 * - Reading text data from Notepad
 * - Parsing CSV-like data
 * - Entering structured data into Excel cells
 */

import { createStep, createWorkflow, z } from '../../packages/terminator-workflow/src';

// ============================================================================
// Input Schema
// ============================================================================

const InputSchema = z.object({
  includeHeaders: z
    .boolean()
    .default(true)
    .describe('Include column headers in Excel'),
});

type Input = z.infer<typeof InputSchema>;

// ============================================================================
// Helper: Parse finance data
// ============================================================================

interface InvoiceRecord {
  invoiceNumber: string;
  client: string;
  amount: string;
  date: string;
}

function parseFinanceData(text: string): InvoiceRecord[] {
  // Split by \r, \n, or \r\n to handle all line ending types
  const lines = text.split(/\r\n|\r|\n/).filter(line => line.trim().length > 0);
  const records: InvoiceRecord[] = [];

  for (const line of lines) {
    // Parse format: "Invoice #1001, Client: Acme Corp, Amount: $1500.00, Date: 2025-01-15"
    const invoiceMatch = line.match(/Invoice #(\d+)/);
    const clientMatch = line.match(/Client: ([^,]+)/);
    const amountMatch = line.match(/Amount: (\$[\d,]+\.?\d*)/);
    const dateMatch = line.match(/Date: ([\d-]+)/);

    if (invoiceMatch && clientMatch && amountMatch && dateMatch) {
      records.push({
        invoiceNumber: invoiceMatch[1],
        client: clientMatch[1].trim(),
        amount: amountMatch[1],
        date: dateMatch[1],
      });
    }
  }

  return records;
}

// ============================================================================
// Steps
// ============================================================================

const createFinanceData = createStep({
  id: 'create-finance-data',
  name: 'Create Sample Finance Data',
  description: 'Opens Notepad and creates sample invoice data',

  execute: async ({ desktop, logger }) => {
    logger.info('📝 Opening Notepad...');
    await desktop.openApplication('notepad');
    await desktop.delay(2000);

    // Check if Notepad has existing content - if so, create a new tab
    logger.info('🔍 Checking if Notepad has existing content...');
    const textEditor = await desktop.locator('role:Document|name:Text editor').first(5000);
    const existingText = await textEditor.text();

    if (existingText.trim().length > 0) {
      logger.info('📄 Notepad has existing content - creating new tab...');
      const addTabButton = await desktop.locator('role:Button|name:Add New Tab').first(2000);
      await addTabButton.click();
      await desktop.delay(500);
      logger.info('✅ New tab created');
    }

    logger.info('💰 Creating sample finance data...');
    const textEditorNow = await desktop.locator('role:Document|name:Text editor').first(5000);

    const sampleData = [
      'Invoice #1001, Client: Acme Corp, Amount: $1500.00, Date: 2025-01-15',
      'Invoice #1002, Client: Tech Solutions, Amount: $2800.50, Date: 2025-01-16',
      'Invoice #1003, Client: Global Industries, Amount: $950.75, Date: 2025-01-17',
      'Invoice #1004, Client: Mega Systems, Amount: $4200.00, Date: 2025-01-18',
      'Invoice #1005, Client: Small Business Inc, Amount: $675.25, Date: 2025-01-19',
    ].join('\n');

    await textEditorNow.typeText(sampleData);
    logger.info(`✅ Created ${sampleData.split('\n').length} invoice records`);

    return { recordCount: sampleData.split('\n').length };
  },
});

const openExcelWorkbook = createStep({
  id: 'open-excel',
  name: 'Open Excel Workbook',
  description: 'Opens Excel and creates a blank workbook',

  execute: async ({ desktop, logger }) => {
    logger.info('📊 Opening Excel...');
    await desktop.openApplication('excel');
    await desktop.delay(3000);

    logger.info('📄 Creating blank workbook...');
    const blankWorkbook = await desktop.locator('role:ListItem|name:Blank workbook').first(5000);
    await blankWorkbook.click();
    await desktop.delay(3000);

    logger.info('✅ Excel workbook ready');
  },
});

const readNotepadData = createStep({
  id: 'read-notepad-data',
  name: 'Read Finance Data from Notepad',
  description: 'Reads and parses invoice data from Notepad',

  execute: async ({ desktop, logger }) => {
    logger.info('📖 Reading data from Notepad...');

    // Activate Notepad window
    const notepadWindow = await desktop.locator('role:Window|name:Notepad').first(5000);
    await notepadWindow.activateWindow();
    await desktop.delay(500);

    // Get the text content from the Document element
    const textEditor = await desktop.locator('role:Document|name:Text editor').first(5000);
    const text = await textEditor.text();

    logger.info(`📄 Read ${text.length} characters`);

    // Parse the data
    const records = parseFinanceData(text);
    logger.info(`✅ Parsed ${records.length} invoice records`);

    return { records };
  },
});

const enterDataIntoExcel = createStep({
  id: 'enter-data-excel',
  name: 'Enter Data into Excel',
  description: 'Enters parsed invoice data into Excel spreadsheet',

  execute: async ({ desktop, input, logger, stepResults }) => {
    logger.info('📝 Entering data into Excel...');

    // Get the parsed records from previous step
    const readResult = stepResults['read-notepad-data'];
    if (!readResult || !readResult.records) {
      throw new Error('No invoice records found from previous step');
    }

    const records = readResult.records as InvoiceRecord[];

    // Activate Excel window
    const excelWindow = await desktop.locator('role:Window|name:Excel').first(5000);
    await excelWindow.activateWindow();
    await desktop.delay(500);

    // Click on cell A1 to start
    const cellA1 = await desktop.locator('role:DataItem|name:A1').first(5000);
    await cellA1.click();
    await desktop.delay(300);

    // Enter headers if requested
    let currentRow = 1;
    if (input.includeHeaders) {
      logger.info('📋 Writing headers...');
      await desktop.pressKey('Invoice #{Tab}');
      await desktop.pressKey('Client{Tab}');
      await desktop.pressKey('Amount{Tab}');
      await desktop.pressKey('Date{Enter}');
      currentRow = 2;
    }

    // Enter each record
    logger.info(`📊 Writing ${records.length} records...`);
    for (let i = 0; i < records.length; i++) {
      const record = records[i];

      // Type the data with Tab between columns, Enter at end of row
      await desktop.pressKey(`${record.invoiceNumber}{Tab}`);
      await desktop.pressKey(`${record.client}{Tab}`);
      await desktop.pressKey(`${record.amount}{Tab}`);
      await desktop.pressKey(`${record.date}{Enter}`);

      logger.info(`  ✓ Row ${currentRow}: Invoice #${record.invoiceNumber}`);
      currentRow++;
    }

    logger.info('✅ All data entered successfully');
    return { rowsEntered: records.length + (input.includeHeaders ? 1 : 0) };
  },
});

// ============================================================================
// Workflow
// ============================================================================

const workflow = createWorkflow({
  name: 'Finance Data Entry',
  description: 'Automated invoice data entry from Notepad to Excel',
  version: '1.0.0',
  input: InputSchema,
})
  .step(createFinanceData)
  .step(openExcelWorkbook)
  .step(readNotepadData)
  .step(enterDataIntoExcel)
  .build();

// ============================================================================
// Execute (CLI)
// ============================================================================

if (require.main === module) {
  const input: Input = {
    includeHeaders: true,
  };

  workflow.run(input).catch(error => {
    console.error('\n❌ Workflow execution failed:', error);
    process.exit(1);
  });
}

export default workflow;
