# DOCX Creation, Editing, and Analysis

## Overview

A user may ask you to create, edit, or analyze the contents of a .docx file. A .docx file is essentially a ZIP archive containing XML files and other resources that you can read or edit.

## Workflow Decision Tree

### Reading/Analyzing Content

Use "Text extraction" or "Raw XML access" sections below

### Creating New Document

Use "Creating a new Word document" workflow

### Editing Existing Document

- **Your own document + simple changes**: Use "Basic OOXML editing" workflow
- **Someone else's document**: Use **"Redlining workflow"** (recommended default)
- **Legal, academic, business, or government docs**: Use **"Redlining workflow"** (required)

## Reading and Analyzing Content

### Text Extraction

If you just need to read the text contents of a document, convert the document to markdown using pandoc:

```bash
# Convert document to markdown with tracked changes
pandoc --track-changes=all path-to-file.docx -o output.md
# Options: --track-changes=accept/reject/all
```

### Raw XML Access

You need raw XML access for: comments, complex formatting, document structure, embedded media, and metadata.

#### Unpacking a file

```bash
# Unzip the .docx to access raw XML
unzip path-to-file.docx -d unpacked_doc
```

#### Key file structures

- `word/document.xml` - Main document contents
- `word/comments.xml` - Comments referenced in document.xml
- `word/media/` - Embedded images and media files
- Tracked changes use `<w:ins>` (insertions) and `<w:del>` (deletions) tags

## Creating a New Word Document

When creating a new Word document from scratch, use **docx** npm package which allows you to create Word documents using JavaScript/TypeScript.

### Workflow

1. Create a JavaScript/TypeScript file using Document, Paragraph, TextRun components
2. Export as .docx using Packer.toBuffer()

### Example

```typescript
import { Document, Paragraph, TextRun, Packer } from "docx";
import * as fs from "fs";

const doc = new Document({
  sections: [{
    properties: {},
    children: [
      new Paragraph({
        children: [
          new TextRun({ text: "Hello World", bold: true }),
          new TextRun({ text: " - This is a document" }),
        ],
      }),
    ],
  }],
});

Packer.toBuffer(doc).then((buffer) => {
  fs.writeFileSync("output.docx", buffer);
});
```

## Editing an Existing Word Document

When editing an existing Word document, you need to work with the raw Office Open XML (OOXML) format.

### Workflow

1. Unpack the document: `unzip file.docx -d unpacked`
2. Edit the XML files (primarily `word/document.xml`)
3. Pack the final document: `cd unpacked && zip -r ../modified.docx .`

## Redlining Workflow for Document Review

This workflow allows you to plan comprehensive tracked changes using markdown before implementing them in OOXML.

**Principle: Minimal, Precise Edits**
When implementing tracked changes, only mark text that actually changes. Break replacements into: [unchanged text] + [deletion] + [insertion] + [unchanged text].

Example - Changing "30 days" to "60 days" in a sentence:

```xml
<!-- BAD - Replaces entire sentence -->
<w:del><w:r><w:delText>The term is 30 days.</w:delText></w:r></w:del>
<w:ins><w:r><w:t>The term is 60 days.</w:t></w:r></w:ins>

<!-- GOOD - Only marks what changed -->
<w:r><w:t>The term is </w:t></w:r>
<w:del><w:r><w:delText>30</w:delText></w:r></w:del>
<w:ins><w:r><w:t>60</w:t></w:r></w:ins>
<w:r><w:t> days.</w:t></w:r>
```

### Tracked Changes Workflow

1. **Get markdown representation**: Convert document to markdown with tracked changes preserved:
   ```bash
   pandoc --track-changes=all path-to-file.docx -o current.md
   ```

2. **Identify and group changes**: Review the document and identify ALL changes needed

3. **Implement changes**: Edit the XML files with proper tracked change markup

4. **Pack the document**: Convert the unpacked directory back to .docx:
   ```bash
   cd unpacked && zip -r ../reviewed-document.docx .
   ```

5. **Final verification**: Do a comprehensive check of the complete document

## Converting Documents to Images

To visually analyze Word documents, convert them to images using a two-step process:

1. **Convert DOCX to PDF**:
   ```bash
   soffice --headless --convert-to pdf document.docx
   ```

2. **Convert PDF pages to JPEG images**:
   ```bash
   pdftoppm -jpeg -r 150 document.pdf page
   ```

This creates files like `page-1.jpg`, `page-2.jpg`, etc.

## Dependencies

Required dependencies:

- **pandoc**: `sudo apt-get install pandoc` (for text extraction)
- **docx**: `npm install docx` (for creating new documents)
- **LibreOffice**: `sudo apt-get install libreoffice` (for PDF conversion)
- **Poppler**: `sudo apt-get install poppler-utils` (for pdftoppm)
