use super::*;

pub(super) fn generate_table(
    out: &mut String,
    table: &Table,
    ctx: &mut GenCtx,
) -> Result<(), ConvertError> {
    ctx.table_depth += 1;
    let result = match table.alignment {
        Some(Alignment::Center) => {
            out.push_str("#align(center)[\n");
            let result = generate_table_inner(out, table, ctx);
            out.push_str("]\n");
            result
        }
        Some(Alignment::Right) => {
            out.push_str("#align(right)[\n");
            let result = generate_table_inner(out, table, ctx);
            out.push_str("]\n");
            result
        }
        _ => generate_table_inner(out, table, ctx),
    };
    ctx.table_depth -= 1;
    result
}

fn generate_table_inner(
    out: &mut String,
    table: &Table,
    ctx: &mut GenCtx,
) -> Result<(), ConvertError> {
    out.push_str("#table(\n");

    // Only explicitly set borders render: Excel does not print gridlines,
    // and Word/PowerPoint borderless tables have none either. Typst's
    // default 1pt grid painted spurious borders on every unbordered table.
    out.push_str("  stroke: none,\n");

    if let Some(ref default_vertical_align) = table.default_vertical_align {
        let align_str: &str = match default_vertical_align {
            CellVerticalAlign::Top => "top",
            CellVerticalAlign::Center => "horizon",
            CellVerticalAlign::Bottom => "bottom",
        };
        let _ = writeln!(out, "  align: {align_str},");
    }

    if let Some(padding) = table.default_cell_padding {
        let _ = writeln!(out, "  inset: {},", format_insets(&padding));
    }

    let num_cols = if !table.column_widths.is_empty() {
        table.column_widths.len()
    } else {
        table.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0)
    };

    if !table.column_widths.is_empty() {
        out.push_str("  columns: (");
        for (i, w) in table.column_widths.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{}pt", format_f64(*w));
        }
        out.push_str("),\n");
    } else if num_cols > 1 {
        let _ = writeln!(out, "  columns: {num_cols},");
    }

    if !table.use_content_driven_row_heights && table.rows.iter().any(|row| row.height.is_some()) {
        out.push_str("  rows: (");
        for (i, row) in table.rows.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            match row.height {
                Some(height) => {
                    let _ = write!(out, "{}pt", format_f64(height));
                }
                None => out.push_str("auto"),
            }
        }
        out.push_str("),\n");
    }

    let mut rowspan_remaining = vec![0usize; num_cols];
    let header_row_count = table.header_row_count.min(table.rows.len());
    let default_cell_padding = table.default_cell_padding.unwrap_or(Insets {
        top: 5.0,
        right: 5.0,
        bottom: 5.0,
        left: 5.0,
    });

    let fixed_row_heights = !table.use_content_driven_row_heights;

    // Rows above a print-title range belong to the header block but print only
    // once, so they go in a `repeat: false` header. The repeating title rows
    // then need a higher level to keep repeating alongside it.
    let lead_row_count = table
        .non_repeating_header_row_count
        .min(table.rows.len().saturating_sub(header_row_count));
    if lead_row_count > 0 {
        out.push_str("  table.header(repeat: false,\n");
        generate_table_rows(
            out,
            &table.rows[..lead_row_count],
            num_cols,
            &mut rowspan_remaining,
            "    ",
            default_cell_padding,
            fixed_row_heights,
            ctx,
        )?;
        out.push_str("  ),\n");
    }

    if header_row_count > 0 {
        if lead_row_count > 0 {
            out.push_str("  table.header(level: 2,\n");
        } else {
            out.push_str("  table.header(\n");
        }
        generate_table_rows(
            out,
            &table.rows[lead_row_count..lead_row_count + header_row_count],
            num_cols,
            &mut rowspan_remaining,
            "    ",
            default_cell_padding,
            fixed_row_heights,
            ctx,
        )?;
        out.push_str("  ),\n");
    }

    generate_table_rows(
        out,
        &table.rows[lead_row_count + header_row_count..],
        num_cols,
        &mut rowspan_remaining,
        "  ",
        default_cell_padding,
        fixed_row_heights,
        ctx,
    )?;

    out.push_str(")\n");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_table_rows(
    out: &mut String,
    rows: &[TableRow],
    num_cols: usize,
    rowspan_remaining: &mut [usize],
    indent: &str,
    default_cell_padding: Insets,
    fixed_row_heights: bool,
    ctx: &mut GenCtx,
) -> Result<(), ConvertError> {
    // A nested table decides its own rows; restore the enclosing row's answer
    // so the outer cells that follow keep sharing their baseline.
    let enclosing_row_has_east_asian_text: bool = ctx.row_has_east_asian_text;
    for row in rows {
        for rs in rowspan_remaining.iter_mut() {
            if *rs > 0 {
                *rs -= 1;
            }
        }

        // Word sizes a row's lines from the whole row: if any cell holds East
        // Asian text, every cell in it takes the East Asian line height, and a
        // snapping grid applies to all of them. Asking each cell separately
        // split mixed-script rows across two baselines (issue #498).
        ctx.row_has_east_asian_text = row_has_east_asian_text(row);

        let mut col_pos: usize = 0;
        for cell in &row.cells {
            if cell.col_span == 0 || cell.row_span == 0 {
                continue;
            }

            while col_pos < num_cols && rowspan_remaining[col_pos] > 0 {
                col_pos += 1;
            }
            if col_pos >= num_cols {
                break;
            }

            let remaining = num_cols - col_pos;
            let clamped_colspan = (cell.col_span as usize).min(remaining).max(1) as u32;
            generate_table_cell(
                out,
                cell,
                clamped_colspan,
                indent,
                default_cell_padding,
                row.height.filter(|_| fixed_row_heights),
                ctx,
            )?;

            if cell.row_span > 1 {
                for rs in rowspan_remaining
                    .iter_mut()
                    .skip(col_pos)
                    .take(clamped_colspan as usize)
                {
                    *rs = cell.row_span as usize;
                }
            }
            col_pos += clamped_colspan as usize;
        }

        while col_pos < num_cols {
            if rowspan_remaining[col_pos] == 0 {
                let _ = writeln!(out, "{indent}[],");
            }
            col_pos += 1;
        }
    }
    ctx.row_has_east_asian_text = enclosing_row_has_east_asian_text;

    Ok(())
}

/// Whether any cell in the row carries East Asian text.
///
/// Nested tables are excluded: they run their own row loop and decide each of
/// their rows on their own content.
fn row_has_east_asian_text(row: &TableRow) -> bool {
    row.cells
        .iter()
        .flat_map(|cell| cell.content.iter())
        .any(block_has_east_asian_text)
}

fn block_has_east_asian_text(block: &Block) -> bool {
    match block {
        Block::Paragraph(paragraph) => paragraph
            .runs
            .iter()
            .any(|run| run.text.chars().any(is_cjk_like)),
        Block::List(list) => {
            list.items
                .iter()
                .flat_map(|item| item.content.iter())
                .any(|paragraph| {
                    paragraph
                        .runs
                        .iter()
                        .any(|run| run.text.chars().any(is_cjk_like))
                })
        }
        _ => false,
    }
}

/// Excel does not fill the cell with a data bar: it insets the bar from the
/// row's top and bottom edges. Native Excel PDF exports of the business corpus
/// print a 10 pt bar in every 14 pt row, which is 2 pt of clearance per side.
const DATA_BAR_VERTICAL_INSET_PT: f64 = 2.0;

/// Floor for rows shorter than the inset, so a bar never vanishes or inverts.
const DATA_BAR_MIN_HEIGHT_PT: f64 = 1.0;
/// Excel's arrow icon sets are drawn shapes, not characters. Native Excel PDFs
/// print an arrow about 10 pt tall in a 14 pt row, filled in the band color and
/// outlined a shade darker.
const ARROW_ICON_LENGTH_PT: f64 = 10.0;
/// Across the shaft the arrow is narrower than it is long.
const ARROW_ICON_BREADTH_PT: f64 = 8.0;

/// Diameter of a circular icon-set icon, in points.
///
/// Measured from Excel's export of the audited workbook: 6.72pt printed at
/// that sheet's 75% scale, so 8.96pt at 100%. The `●` character it used to
/// print is a little over half that (#536).
const CIRCLE_ICON_DIAMETER_PT: f64 = 8.96;

/// The drawn shape for an icon-set glyph, or `None` for the sets that stay
/// characters — symbols, flags, stars.
fn icon_shape(glyph: &str, color: Option<Color>) -> Option<String> {
    if glyph == crate::ir::ICON_CIRCLE {
        let radius: f64 = CIRCLE_ICON_DIAMETER_PT / 2.0;
        let paint: String = color
            .map(|c| rgb(&c))
            .unwrap_or_else(|| "black".to_string());
        return Some(format!(
            "circle(radius: {}pt, fill: {paint}, stroke: none)",
            format_f64(radius)
        ));
    }
    arrow_icon_polygon(glyph, color)
}

/// Build the Typst `polygon` for one of the arrow icon-set glyphs, or `None`
/// for any other glyph.
fn arrow_icon_polygon(glyph: &str, color: Option<Color>) -> Option<String> {
    // Head half-width, shaft half-width, and where the head meets the shaft,
    // as fractions of the arrow's breadth and length.
    let breadth: f64 = ARROW_ICON_BREADTH_PT;
    let length: f64 = ARROW_ICON_LENGTH_PT;
    let shaft: f64 = breadth * 0.28;
    let neck: f64 = length * 0.45;

    // Points of an up arrow, clockwise from the tip.
    let up: Vec<(f64, f64)> = vec![
        (breadth / 2.0, 0.0),
        (breadth, neck),
        (breadth / 2.0 + shaft, neck),
        (breadth / 2.0 + shaft, length),
        (breadth / 2.0 - shaft, length),
        (breadth / 2.0 - shaft, neck),
        (0.0, neck),
    ];
    let flip_y = |points: &[(f64, f64)]| -> Vec<(f64, f64)> {
        points.iter().map(|(x, y)| (*x, length - *y)).collect()
    };
    let transpose = |points: &[(f64, f64)]| -> Vec<(f64, f64)> {
        points.iter().map(|(x, y)| (length - *y, *x)).collect()
    };

    let (points, rotation): (Vec<(f64, f64)>, Option<i32>) = match glyph {
        crate::ir::ICON_ARROW_UP => (up, None),
        crate::ir::ICON_ARROW_DOWN => (flip_y(&up), None),
        crate::ir::ICON_ARROW_RIGHT => (transpose(&up), None),
        crate::ir::ICON_ARROW_UP_RIGHT => (up, Some(45)),
        crate::ir::ICON_ARROW_DOWN_RIGHT => (flip_y(&up), Some(-45)),
        _ => return None,
    };

    let coordinates: String = points
        .iter()
        .map(|(x, y)| format!("({}pt, {}pt)", format_f64(*x), format_f64(*y)))
        .collect::<Vec<String>>()
        .join(", ");
    let paint: String = color
        .map(|c| rgb(&c))
        .unwrap_or_else(|| "black".to_string());
    let shape: String =
        format!("polygon(fill: {paint}, stroke: 0.4pt + {paint}.darken(30%), {coordinates})");
    Some(match rotation {
        Some(degrees) => format!("rotate({degrees}deg, {shape})"),
        None => shape,
    })
}

fn generate_table_cell(
    out: &mut String,
    cell: &TableCell,
    clamped_colspan: u32,
    indent: &str,
    default_cell_padding: Insets,
    row_height: Option<f64>,
    ctx: &mut GenCtx,
) -> Result<(), ConvertError> {
    // The row's declared height less the padding this cell actually uses is
    // what one line has to fit inside (issue #608). A cell spanning rows is
    // excluded: its content is sized by the rows together, not by this one.
    let padding: Insets = cell.padding.unwrap_or(default_cell_padding);
    let enclosing_sheet_row_line_pt: Option<f64> = ctx.sheet_row_line_pt;
    ctx.sheet_row_line_pt = row_height
        .filter(|_| cell.row_span <= 1)
        .map(|height| height - padding.top - padding.bottom)
        .filter(|available| *available > 0.0);

    let needs_cell_fn = clamped_colspan > 1
        || cell.row_span > 1
        || cell.border.is_some()
        || cell.background.is_some()
        || cell.vertical_align.is_some()
        || cell.padding.is_some();

    if needs_cell_fn {
        out.push_str(indent);
        out.push_str("table.cell(");
        write_cell_params(out, cell, clamped_colspan, default_cell_padding);
        out.push_str(")[");
    } else {
        out.push_str(indent);
        out.push('[');
    }

    if let Some(border) = &cell.border {
        write_double_border_overlays(out, border, cell.padding.unwrap_or(default_cell_padding));
    }

    if let Some(ref db) = cell.data_bar {
        // Excel draws the bar behind the value on the same line (no track),
        // with a horizontal fade of the bar color; #place keeps it out of
        // layout so the value renders on top at its normal position. The bar
        // height must be concrete: in auto-height rows a relative height has
        // no cell frame to resolve against and blows up to the page height,
        // smearing over neighboring rows (issue #362).
        let pct = db.fill_pct.clamp(0.0, 100.0);
        let bar_height: String = match row_height {
            Some(height) => {
                let inset_height =
                    (height - 2.0 * DATA_BAR_VERTICAL_INSET_PT).max(DATA_BAR_MIN_HEIGHT_PT);
                format!("{}pt", format_f64(inset_height))
            }
            // Excel sizes default rows to the font's line box; 1.2em tracks
            // that for single-line numeric cells, less the same inset.
            None => format!("1.2em - {}pt", format_f64(2.0 * DATA_BAR_VERTICAL_INSET_PT)),
        };
        let _ = write!(
            out,
            "#place(left + horizon, box(width: {}%, height: {}, fill: gradient.linear({}, {}.lighten(70%))))",
            format_f64(pct),
            bar_height,
            rgb(&db.color),
            rgb(&db.color),
        );
    }

    if let Some(ref icon) = cell.icon_text {
        // Excel draws icon set glyphs in their band color, independent of
        // the cell's font color, anchored at the cell's left edge on the
        // value's own line. Placing the icon out of layout keeps narrow
        // cells from wrapping the value onto a second line, which doubled
        // the row height (issue #367).
        // Excel's arrow sets are drawn shapes rather than characters: a shaft
        // with a triangular head, outlined and filling most of the row. The
        // triangle characters the parser records are only a third that size,
        // so arrows are re-drawn as polygons.
        // The circle sets are drawn discs for the same reason (#536).
        match (icon_shape(icon, cell.icon_color), cell.icon_color) {
            (Some(polygon), _) => {
                let _ = write!(out, "#place(left + horizon, {polygon})");
            }
            (None, Some(color)) => {
                let _ = write!(
                    out,
                    "#place(left + horizon, text(fill: {}, weight: \"bold\")[{}])",
                    rgb(&color),
                    icon
                );
            }
            (None, None) => {
                let _ = write!(
                    out,
                    "#place(left + horizon, text(weight: \"bold\")[{icon}])"
                );
            }
        }
    }

    if let Some(spill_width) = cell.spill_width {
        // Excel paints unwrapped text across empty right neighbors without
        // growing the row: lay the content out on one clipped line via
        // #place (out of layout) and hold the row height with a zero-width
        // strut.
        let _ = write!(
            out,
            "#place(left + horizon, box(width: {}pt, height: 1.3em, clip: true)[",
            format_f64(spill_width),
        );
        generate_cell_content(out, &cell.content, ctx)?;
        out.push_str("])#box(width: 0pt, height: 1.3em)");
    } else {
        generate_cell_content(out, &cell.content, ctx)?;
    }
    out.push_str("],\n");
    ctx.sheet_row_line_pt = enclosing_sheet_row_line_pt;
    Ok(())
}

fn write_double_border_overlays(out: &mut String, border: &CellBorder, padding: Insets) {
    if let Some(side) = border
        .top
        .as_ref()
        .filter(|side| side.style == BorderLineStyle::Double)
    {
        write_horizontal_double_border(out, side, padding, true);
    }
    if let Some(side) = border
        .bottom
        .as_ref()
        .filter(|side| side.style == BorderLineStyle::Double)
    {
        write_horizontal_double_border(out, side, padding, false);
    }
    if let Some(side) = border
        .left
        .as_ref()
        .filter(|side| side.style == BorderLineStyle::Double)
    {
        write_vertical_double_border(out, side, padding, true);
    }
    if let Some(side) = border
        .right
        .as_ref()
        .filter(|side| side.style == BorderLineStyle::Double)
    {
        write_vertical_double_border(out, side, padding, false);
    }
}

fn write_horizontal_double_border(
    out: &mut String,
    side: &BorderSide,
    padding: Insets,
    is_top: bool,
) {
    let align = if is_top {
        "top + left"
    } else {
        "bottom + left"
    };
    let first_dy = if is_top {
        -padding.top - side.width
    } else {
        padding.bottom - side.width
    };
    let second_dy = if is_top {
        -padding.top + side.width
    } else {
        padding.bottom + side.width
    };
    let dx = -padding.left;
    let length_extra = padding.left + padding.right;
    write_double_border_line(out, align, dx, first_dy, "0deg", length_extra, side);
    write_double_border_line(out, align, dx, second_dy, "0deg", length_extra, side);
}

fn write_vertical_double_border(
    out: &mut String,
    side: &BorderSide,
    padding: Insets,
    is_left: bool,
) {
    let align = if is_left { "top + left" } else { "top + right" };
    let first_dx = if is_left {
        -padding.left - side.width
    } else {
        padding.right - side.width
    };
    let second_dx = if is_left {
        -padding.left + side.width
    } else {
        padding.right + side.width
    };
    let dy = -padding.top;
    let length_extra = padding.top + padding.bottom;
    write_double_border_line(out, align, first_dx, dy, "90deg", length_extra, side);
    write_double_border_line(out, align, second_dx, dy, "90deg", length_extra, side);
}

fn write_double_border_line(
    out: &mut String,
    align: &str,
    dx: f64,
    dy: f64,
    angle: &str,
    length_extra: f64,
    side: &BorderSide,
) {
    let _ = write!(
        out,
        "#place({align}, dx: {}pt, dy: {}pt, line(length: 100% + {}pt, angle: {angle}, stroke: {}pt + {}))",
        format_geometry(dx),
        format_geometry(dy),
        format_geometry(length_extra),
        format_geometry(side.width),
        rgb(&side.color),
    );
}

fn format_geometry(value: f64) -> String {
    let rounded = (value * 1_000.0).round() / 1_000.0;
    format_f64(if rounded == -0.0 { 0.0 } else { rounded })
}

/// The cell's inset, with the layout space its horizontal borders occupy.
///
/// Typst draws our per-cell strokes without reserving room for them, but Word
/// counts a border's width in the row height. Each horizontal border is shared
/// between the rows above and below it, so each cell takes half (issues #500,
/// #503).
fn cell_inset_with_border(cell: &TableCell, default_cell_padding: Insets) -> Insets {
    let padding: Insets = cell.padding.unwrap_or(default_cell_padding);
    let Some(border) = &cell.border else {
        return padding;
    };
    let half = |side: &Option<BorderSide>| side.as_ref().map_or(0.0, |s| s.width / 2.0);
    Insets {
        top: padding.top + half(&border.top),
        bottom: padding.bottom + half(&border.bottom),
        ..padding
    }
}

fn write_cell_params(
    out: &mut String,
    cell: &TableCell,
    clamped_colspan: u32,
    default_cell_padding: Insets,
) {
    let mut first = true;

    if clamped_colspan > 1 {
        write_param(out, &mut first, &format!("colspan: {clamped_colspan}"));
    }
    if cell.row_span > 1 {
        write_param(out, &mut first, &format!("rowspan: {}", cell.row_span));
    }
    if let Some(ref bg) = cell.background {
        write_param(out, &mut first, &format_color(bg));
    }
    let inset: Insets = cell_inset_with_border(cell, default_cell_padding);
    if cell.padding.is_some() || cell.border.is_some() {
        write_param(
            out,
            &mut first,
            &format!("inset: {}", format_insets(&inset)),
        );
    }
    if let Some(ref border) = cell.border {
        let stroke = format_cell_stroke(border);
        if !stroke.is_empty() {
            write_param(out, &mut first, &stroke);
        }
    }
    if let Some(ref va) = cell.vertical_align {
        let align_str: &str = match va {
            CellVerticalAlign::Top => "top",
            CellVerticalAlign::Center => "horizon",
            CellVerticalAlign::Bottom => "bottom",
        };
        write_param(out, &mut first, &format!("align: {align_str}"));
    }
}

fn format_cell_stroke(border: &CellBorder) -> String {
    let mut parts = Vec::with_capacity(4);

    if let Some(ref side) = border.top
        && side.style != BorderLineStyle::Double
    {
        parts.push(format!("top: {}", format_border_side(side)));
    }
    if let Some(ref side) = border.bottom
        && side.style != BorderLineStyle::Double
    {
        parts.push(format!("bottom: {}", format_border_side(side)));
    }
    if let Some(ref side) = border.left
        && side.style != BorderLineStyle::Double
    {
        parts.push(format!("left: {}", format_border_side(side)));
    }
    if let Some(ref side) = border.right
        && side.style != BorderLineStyle::Double
    {
        parts.push(format!("right: {}", format_border_side(side)));
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("stroke: ({})", parts.join(", "))
    }
}

fn format_border_side(side: &BorderSide) -> String {
    stroke_value(side, true)
}

fn generate_cell_content(
    out: &mut String,
    blocks: &[Block],
    ctx: &mut GenCtx,
) -> Result<(), ConvertError> {
    for (i, block) in blocks.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        match block {
            // A `TOC` field inside a table cell is not a shape Word produces.
            Block::TableOfContents(_) => {}
            Block::Caption(caption) => generate_cell_paragraph(
                out,
                &caption.paragraph,
                ctx.default_tab_width_pt,
                ctx.line_grid_pitch,
                ctx.row_has_east_asian_text,
                ctx.sheet_row_line_pt,
            ),
            Block::Paragraph(para) => generate_cell_paragraph(
                out,
                para,
                ctx.default_tab_width_pt,
                ctx.line_grid_pitch,
                ctx.row_has_east_asian_text,
                ctx.sheet_row_line_pt,
            ),
            Block::Table(table) => {
                if ctx.table_depth < MAX_TABLE_DEPTH {
                    generate_table(out, table, ctx)?;
                }
            }
            Block::Image(img) => generate_image(out, img, ctx),
            Block::InlineImages(images) => {
                for image in images {
                    generate_image(out, image, ctx);
                }
            }
            Block::FloatingImage(fi) => generate_floating_image(out, fi, ctx),
            Block::FloatingTextBox(ftb) => generate_floating_text_box(out, ftb, ctx)?,
            Block::FloatingShape(fs) => generate_floating_shape(out, fs),
            Block::List(list) => {
                if can_render_fixed_text_list_inline(list) {
                    generate_fixed_text_list(out, list, true, None)?;
                } else {
                    generate_list(out, list, None)?;
                }
            }
            Block::MathEquation(math) => generate_math_equation(out, math),
            Block::Chart(chart) => generate_chart(out, chart),
            Block::PageBreak | Block::ColumnBreak => {}
        }
    }
    Ok(())
}

fn generate_cell_paragraph(
    out: &mut String,
    para: &Paragraph,
    default_tab_width_pt: f64,
    line_grid_pitch: Option<f64>,
    row_has_east_asian_text: bool,
    sheet_row_line_pt: Option<f64>,
) {
    let style: &ParagraphStyle = &para.style;
    let alignment = style.alignment;
    let align_str: Option<&str> = match alignment {
        Some(Alignment::Left) => Some("left"),
        Some(Alignment::Center) => Some("center"),
        Some(Alignment::Right) => Some("right"),
        _ => None,
    };
    // Table-cell text occupies the font's full single-spacing (hhea) line
    // as a fixed box: a single-line cell must fill the whole line height
    // Word gives it rather than only the tighter metric box, or auto-height
    // rows come out short (issue #396). A cell whose *row* holds East Asian
    // text takes 1.3 times that line, like body text, and a snapping grid's
    // pitch above it — decided once per row so every cell in it shares a
    // baseline, the numeric ones included (issues #498, #518).
    // A sheet row states its own total height, so the line is pinned to it
    // rather than derived from the font the way a Word row's is (issue #608).
    let line_height_settings: Option<String> =
        sheet_cell_line_box_settings(&para.runs, style, sheet_row_line_pt).or_else(|| {
            word_cell_line_box_settings(&para.runs, style, line_grid_pitch, row_has_east_asian_text)
        });
    let has_block_wrapper = cell_paragraph_needs_block_wrapper(style)
        || align_str.is_some()
        || line_height_settings.is_some();

    if has_block_wrapper {
        out.push_str("#block(");
        write_cell_paragraph_block_params(out, align_str.is_some());
        out.push_str(")[\n");
        write_line_box_settings(out, style.line_box);
        write_par_settings(out, style);
        if let Some(align_str) = align_str {
            let _ = writeln!(out, "  #set align({align_str})");
        }
        if let Some(ref settings) = line_height_settings {
            out.push_str(settings);
        }
    }

    if let Some(space_before) = style.space_before {
        let _ = writeln!(out, "#v({}pt)", format_f64(space_before));
    }

    generate_runs_with_tabs(
        out,
        &para.runs,
        style.tab_stops.as_deref(),
        default_tab_width_pt,
    );

    // Suppressed when the grid-snapped line box already contains it, or the
    // gap would be counted twice (issues #500, #503).
    if let Some(space_after) = style.space_after
        && !cell_grid_absorbs_space_after(style, line_grid_pitch, row_has_east_asian_text)
    {
        let _ = write!(out, "\n#v({}pt)", format_f64(space_after));
    }

    if has_block_wrapper {
        out.push_str("\n]");
    }
}

fn cell_paragraph_needs_block_wrapper(style: &ParagraphStyle) -> bool {
    style.line_spacing.is_some()
        || style.line_box.is_some()
        || matches!(style.alignment, Some(Alignment::Justify))
        || matches!(style.direction, Some(TextDirection::Rtl))
}

fn write_cell_paragraph_block_params(out: &mut String, needs_full_width: bool) {
    let mut first = true;

    if needs_full_width {
        write_param(out, &mut first, "width: 100%");
    }
}
