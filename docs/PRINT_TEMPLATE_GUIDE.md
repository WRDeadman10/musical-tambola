# Musical Tambola - Custom Ticket Print Template Guide

This guide details the specifications, dimensions, layout coordinates, and operational procedures for designing and applying custom ticket templates in Media Tambola / Musical Tambola.

---

## 1. Canvas Dimensions & Specifications

Custom print templates are loaded as full-bleed background images behind the generated ticket data (grid, event title, ticket code, and footer).

- **Base Canvas Dimensions**: `1800 x 900` pixels (width x height)
- **Aspect Ratio**: Exactly **2:1** (tolerance: ±0.02)
- **Maximum File Size**: **5 MB**
- **Supported Formats**: PNG, JPG / JPEG, WebP

### Validation Rules (from `src/features/ticket-printing/templateRegions.ts`)
1. **Aspect Ratio**: The image aspect ratio `width / height` must equal `2.0` (±0.02). Non-2:1 images will be rejected with `"Template background must use a 2:1 aspect ratio."`
2. **File Size**: Images over 5 MB (5,242,880 bytes) will be rejected with `"Template background must be 5 MB or smaller."`
3. **Print Resolution**: Recommended resolution is 300 DPI (approx. 6" x 3" printed ticket).

---

## 2. Template Coordinate Regions

All template overlays are computed as percentages relative to the 1800x900 canvas:

| Region Key | X (%) | Y (%) | Width (%) | Height (%) | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `canvas` | `0%` | `0%` | `100%` (`1800px`) | `100%` (`900px`) | Full ticket surface boundary. |
| `header` | `0%` | `0%` | `100%` | `18%` | Full banner area across the top of the ticket. |
| `eventName` | `4%` | `3%` | `66%` | `12%` | Event / Organization title text area (left side of header). |
| `ticketCode`| `72%` | `3%` | `24%` | `12%` | Unique ticket identifier (e.g., `T-000042`) and optional barcode area. |
| `grid` | `4%` | `20%` | `92%` | `68%` | 3-row x 9-column Tambola ticket numbers/songs grid overlay. |
| `footer` | `4%` | `90%` | `92%` | `8%` | Terms, disclaimers, sponsor logo, or contact info area. |

---

## 3. Graphic Design Recommendations

When creating template background artwork in Photoshop, Canva, Illustrator, or Figma:

1. **Keep the Grid Zone Legible (`X: 4%, Y: 20%, W: 92%, H: 68%`)**:
   - The game renders semi-transparent or white cell boxes with dark text. Avoid high-contrast dark-and-light patterns directly behind the cells.
   - Watermarks should have low opacity (10-15%).
2. **Leave Clear Header Real Estate (`X: 4% to 96%, Y: 3% to 15%`)**:
   - Keep the upper-left (event name) and upper-right (ticket code) clear of distracting graphic elements.
3. **Safety Margins**:
   - Leave at least 3-4% margin from canvas borders to prevent cut-off during commercial printing or cutting.
4. **Color Profile**:
   - Export images in standard **sRGB** color space.

---

## 4. How to Upload and Use Templates

1. Open **Musical Tambola** and navigate to **Ticket Printing** from the sidebar navigation.
2. Under the **Print Settings** or **Template** panel:
   - Click **Browse Template Image** (or drag and drop your `1800x900` image).
   - The system validates file size (≤ 5MB) and aspect ratio (2:1). If validation passes, a live ticket preview is rendered.
3. **Specify Ticket Range**:
   - Filter by start and end ticket codes (e.g., `T-000001` to `T-000100`).
4. **Export / Print**:
   - Select **Print** to open the system print dialog or **Save as PDF** for batch multi-ticket sheet layout (e.g., 2, 4, or 6 tickets per A4 page).
