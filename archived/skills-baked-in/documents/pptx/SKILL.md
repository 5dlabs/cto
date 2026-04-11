# PPTX Creation, Editing, and Analysis

## Overview

A user may ask you to create, edit, or analyze the contents of a .pptx file. A .pptx file is essentially a ZIP archive containing XML files and other resources that you can read or edit.

## Reading and Analyzing Content

### Text Extraction

If you just need to read the text contents of a presentation:

```bash
# Using python-pptx or markitdown
python -m markitdown path-to-file.pptx
```

### Raw XML Access

You need raw XML access for: comments, speaker notes, slide layouts, animations, design elements, and complex formatting.

#### Unpacking a file

```bash
unzip presentation.pptx -d unpacked_pptx
```

#### Key file structures

- `ppt/presentation.xml` - Main presentation metadata and slide references
- `ppt/slides/slide{N}.xml` - Individual slide contents (slide1.xml, slide2.xml, etc.)
- `ppt/notesSlides/notesSlide{N}.xml` - Speaker notes for each slide
- `ppt/slideLayouts/` - Layout templates for slides
- `ppt/slideMasters/` - Master slide templates
- `ppt/theme/` - Theme and styling information
- `ppt/media/` - Images and other media files

## Creating a New PowerPoint Presentation

When creating a new PowerPoint presentation from scratch, use **PptxGenJS** or **python-pptx**.

### Using python-pptx

```python
from pptx import Presentation
from pptx.util import Inches, Pt

prs = Presentation()

# Add a slide
slide_layout = prs.slide_layouts[0]  # Title slide
slide = prs.slides.add_slide(slide_layout)

# Add title
title = slide.shapes.title
title.text = "My Presentation"

# Add subtitle
subtitle = slide.placeholders[1]
subtitle.text = "Created with python-pptx"

# Save
prs.save('output.pptx')
```

### Design Principles

**CRITICAL**: Before creating any presentation, analyze the content and choose appropriate design elements:

1. **Consider the subject matter**: What is this presentation about?
2. **Check for branding**: If the user mentions a company/organization, consider their brand colors
3. **Match palette to content**: Select colors that reflect the subject
4. **State your approach**: Explain your design choices before writing code

## Editing an Existing PowerPoint Presentation

When editing slides in an existing PowerPoint presentation, you need to work with the raw OOXML format or use python-pptx.

### Using python-pptx for Editing

```python
from pptx import Presentation

prs = Presentation('existing.pptx')

# Access slides
for slide in prs.slides:
    for shape in slide.shapes:
        if shape.has_text_frame:
            for paragraph in shape.text_frame.paragraphs:
                for run in paragraph.runs:
                    print(run.text)

# Modify text
slide = prs.slides[0]
title = slide.shapes.title
title.text = "New Title"

prs.save('modified.pptx')
```

### Raw XML Workflow

1. Unpack the presentation: `unzip file.pptx -d unpacked`
2. Edit the XML files (primarily `ppt/slides/slide{N}.xml`)
3. Pack the final presentation: `cd unpacked && zip -r ../modified.pptx .`

## Converting Slides to Images

To visually analyze PowerPoint slides, convert them to images:

1. **Convert PPTX to PDF**:
   ```bash
   soffice --headless --convert-to pdf template.pptx
   ```

2. **Convert PDF pages to JPEG images**:
   ```bash
   pdftoppm -jpeg -r 150 template.pdf slide
   ```

This creates files like `slide-1.jpg`, `slide-2.jpg`, etc.

## Color Palette Examples

Use these to spark creativity for presentation design:

1. **Classic Blue**: Deep navy (#1C2833), slate gray (#2E4053), silver (#AAB7B8)
2. **Teal & Coral**: Teal (#5EA8A7), coral (#FE4447), white (#FFFFFF)
3. **Bold Red**: Red (#C0392B), orange (#F39C12), green (#2ECC71)
4. **Warm Blush**: Mauve (#A49393), blush (#EED6D3), cream (#FAF7F2)
5. **Black & Gold**: Gold (#BF9A4A), black (#000000), cream (#F4F6F6)

## Layout Tips

**When creating slides with charts or tables:**

- **Two-column layout (PREFERRED)**: Header spanning full width, then two columns below
- **Full-slide layout**: Let featured content take up entire slide for maximum impact
- **NEVER vertically stack**: Do not place charts/tables below text in single column

## Dependencies

Required dependencies:

- **python-pptx**: `pip install python-pptx` (for Python-based manipulation)
- **pptxgenjs**: `npm install pptxgenjs` (for JavaScript-based creation)
- **markitdown**: `pip install markitdown[pptx]` (for text extraction)
- **LibreOffice**: `sudo apt-get install libreoffice` (for PDF conversion)
- **Poppler**: `sudo apt-get install poppler-utils` (for pdftoppm)
