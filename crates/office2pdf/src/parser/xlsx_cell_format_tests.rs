use super::*;

// ----- Cell merging tests (US-015) -----

/// Helper: build XLSX with merge ranges.
fn build_xlsx_with_merges(sheet_name: &str, cells: &[(&str, &str)], merges: &[&str]) -> Vec<u8> {
    let mut book = umya_spreadsheet::new_file();
    {
        let sheet = book.get_sheet_mut(&0).unwrap();
        sheet.set_name(sheet_name);
        for &(coord, value) in cells {
            sheet.get_cell_mut(coord).set_value(value);
        }
        for &merge_range in merges {
            sheet.add_merge_cells(merge_range);
        }
    }
    let mut cursor = Cursor::new(Vec::new());
    umya_spreadsheet::writer::xlsx::write_writer(&book, &mut cursor).unwrap();
    cursor.into_inner()
}

#[test]
fn test_merge_colspan_basic() {
    let data = build_xlsx_with_merges("Sheet1", &[("A1", "Merged")], &["A1:B1"]);
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].cells.len(),
        1,
        "Merged cells should produce 1 cell"
    );
    assert_eq!(tp.table.rows[0].cells[0].col_span, 2);
    assert_eq!(tp.table.rows[0].cells[0].row_span, 1);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "Merged");
}

#[test]
fn test_merge_rowspan_basic() {
    let data = build_xlsx_with_merges("Sheet1", &[("A1", "Tall")], &["A1:A2"]);
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].cells.len(), 1);
    assert_eq!(tp.table.rows[0].cells[0].row_span, 2);
    assert_eq!(tp.table.rows[0].cells[0].col_span, 1);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "Tall");
    assert_eq!(tp.table.rows[1].cells.len(), 0);
}

#[test]
fn test_merge_colspan_and_rowspan() {
    let data = build_xlsx_with_merges(
        "Sheet1",
        &[("A1", "Big"), ("C1", "Right"), ("C2", "Below")],
        &["A1:B2"],
    );
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].cells.len(), 2);
    assert_eq!(tp.table.rows[0].cells[0].col_span, 2);
    assert_eq!(tp.table.rows[0].cells[0].row_span, 2);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "Big");
    assert_eq!(cell_text(&tp.table.rows[0].cells[1]), "Right");
    assert_eq!(tp.table.rows[1].cells.len(), 1);
    assert_eq!(cell_text(&tp.table.rows[1].cells[0]), "Below");
}

#[test]
fn test_merge_content_in_top_left_only() {
    let data = build_xlsx_with_merges(
        "Sheet1",
        &[("A1", "TopLeft"), ("B1", "should be ignored")],
        &["A1:B1"],
    );
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].cells.len(), 1);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "TopLeft");
}

#[test]
fn test_merge_multiple_ranges() {
    let data = build_xlsx_with_merges(
        "Sheet1",
        &[("A1", "Wide"), ("A2", "Tall"), ("B2", "B2"), ("B3", "B3")],
        &["A1:B1", "A2:A3"],
    );
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].cells.len(), 1);
    assert_eq!(tp.table.rows[0].cells[0].col_span, 2);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "Wide");
    assert_eq!(tp.table.rows[1].cells.len(), 2);
    assert_eq!(tp.table.rows[1].cells[0].row_span, 2);
    assert_eq!(cell_text(&tp.table.rows[1].cells[0]), "Tall");
    assert_eq!(cell_text(&tp.table.rows[1].cells[1]), "B2");
    assert_eq!(tp.table.rows[2].cells.len(), 1);
    assert_eq!(cell_text(&tp.table.rows[2].cells[0]), "B3");
}

#[test]
fn test_merge_no_merges_unchanged() {
    let data = build_xlsx_bytes("Sheet1", &[("A1", "X"), ("B1", "Y")]);
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].cells.len(), 2);
    for cell in &tp.table.rows[0].cells {
        assert_eq!(cell.col_span, 1);
        assert_eq!(cell.row_span, 1);
    }
}

#[test]
fn test_merge_wide_colspan() {
    let data = build_xlsx_with_merges("Sheet1", &[("A1", "Title")], &["A1:D1"]);
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].cells.len(), 1);
    assert_eq!(tp.table.rows[0].cells[0].col_span, 4);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "Title");
}

// ----- US-027: Cell formatting tests -----

/// Helper: build XLSX with formatted cells.
fn build_xlsx_formatted(setup: impl FnOnce(&mut umya_spreadsheet::Worksheet)) -> Vec<u8> {
    let mut book = umya_spreadsheet::new_file();
    {
        let sheet = book.get_sheet_mut(&0).unwrap();
        sheet.set_name("Sheet1");
        setup(sheet);
    }
    let mut cursor = Cursor::new(Vec::new());
    umya_spreadsheet::writer::xlsx::write_writer(&book, &mut cursor).unwrap();
    cursor.into_inner()
}

#[test]
fn test_cell_bold_text() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Bold");
        cell.get_style_mut().get_font_mut().set_bold(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let style = first_run_style(&tp.table.rows[0].cells[0]);
    assert_eq!(style.bold, Some(true));
}

#[test]
fn test_cell_italic_text() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Italic");
        cell.get_style_mut().get_font_mut().set_italic(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let style = first_run_style(&tp.table.rows[0].cells[0]);
    assert_eq!(style.italic, Some(true));
}

#[test]
fn test_cell_font_color() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Red");
        cell.get_style_mut()
            .get_font_mut()
            .get_color_mut()
            .set_argb("FFFF0000");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let style = first_run_style(&tp.table.rows[0].cells[0]);
    assert_eq!(style.color, Some(Color::new(255, 0, 0)));
}

#[test]
fn test_cell_font_name_and_size() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Styled");
        let font = cell.get_style_mut().get_font_mut();
        font.set_name("Arial");
        font.set_size(14.0);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let style = first_run_style(&tp.table.rows[0].cells[0]);
    assert_eq!(style.font_family.as_deref(), Some("Arial"));
    assert_eq!(style.font_size, Some(14.0));
}

#[test]
fn test_cell_background_fill() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Yellow BG");
        cell.get_style_mut().set_background_color("FFFFFF00");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = &tp.table.rows[0].cells[0];
    assert_eq!(cell.background, Some(Color::new(255, 255, 0)));
}

#[test]
fn test_cell_borders() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Bordered");
        let borders = cell.get_style_mut().get_borders_mut();
        borders
            .get_bottom_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_MEDIUM);
        borders
            .get_bottom_mut()
            .get_color_mut()
            .set_argb("FF000000");
        borders
            .get_top_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_THIN);
        borders.get_top_mut().get_color_mut().set_argb("FFFF0000");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = &tp.table.rows[0].cells[0];
    let border = cell.border.as_ref().expect("Expected border");
    let bottom = border.bottom.as_ref().expect("Expected bottom border");
    // `medium`, against the measured `thin` of 1.0pt (issue #487).
    assert!((bottom.width - 1.75).abs() < 0.01);
    assert_eq!(bottom.color, Color::new(0, 0, 0));
    let top = border.top.as_ref().expect("Expected top border");
    // `thin`: 2px at 150 DPI on a native Excel export.
    assert!((top.width - 1.0).abs() < 0.01);
    assert_eq!(top.color, Color::new(255, 0, 0));
}

#[test]
fn test_cell_border_styles() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Styled borders");
        let borders = cell.get_style_mut().get_borders_mut();
        borders
            .get_top_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_DASHED);
        borders.get_top_mut().get_color_mut().set_argb("FF000000");
        borders
            .get_bottom_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_DOTTED);
        borders
            .get_bottom_mut()
            .get_color_mut()
            .set_argb("FF000000");
        borders
            .get_left_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_DASHDOT);
        borders.get_left_mut().get_color_mut().set_argb("FF000000");
        borders
            .get_right_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_DOUBLE);
        borders.get_right_mut().get_color_mut().set_argb("FF000000");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = &tp.table.rows[0].cells[0];
    let border = cell.border.as_ref().expect("Expected border");

    let top = border.top.as_ref().expect("Expected top border");
    assert_eq!(top.style, BorderLineStyle::Dashed, "Top should be dashed");

    let bottom = border.bottom.as_ref().expect("Expected bottom border");
    assert_eq!(
        bottom.style,
        BorderLineStyle::Dotted,
        "Bottom should be dotted"
    );

    let left = border.left.as_ref().expect("Expected left border");
    assert_eq!(
        left.style,
        BorderLineStyle::DashDot,
        "Left should be dashDot"
    );

    let right = border.right.as_ref().expect("Expected right border");
    assert_eq!(
        right.style,
        BorderLineStyle::Double,
        "Right should be double"
    );
}

#[test]
fn test_cell_border_medium_dashed() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("MedDash");
        let borders = cell.get_style_mut().get_borders_mut();
        borders
            .get_top_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_MEDIUMDASHED);
        borders.get_top_mut().get_color_mut().set_argb("FF000000");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = &tp.table.rows[0].cells[0];
    let border = cell.border.as_ref().expect("Expected border");
    let top = border.top.as_ref().expect("Expected top border");
    assert_eq!(top.style, BorderLineStyle::Dashed);
    // `mediumDashed` shares the `medium` weight (issue #487).
    assert!((top.width - 1.75).abs() < 0.01);
}

#[test]
fn test_row_height() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("Tall row");
        sheet.get_row_dimension_mut(&1).set_height(30.0);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let row = &tp.table.rows[0];
    assert_eq!(row.height, Some(28.0));
}

#[test]
fn test_cell_no_formatting_defaults() {
    let data = build_xlsx_bytes("Sheet1", &[("A1", "Plain")]);
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = &tp.table.rows[0].cells[0];
    let style = first_run_style(cell);
    assert!(style.bold.is_none() || style.bold == Some(false));
    assert!(style.italic.is_none() || style.italic == Some(false));
    assert!(cell.border.is_none());
    assert!(cell.background.is_none());
}

// ----- US-028: Number format tests -----

#[test]
fn test_number_format_currency() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(1234.56f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code(umya_spreadsheet::NumberingFormat::FORMAT_CURRENCY_USD_SIMPLE);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let text = cell_text(&tp.table.rows[0].cells[0]);
    assert!(
        text.contains('$') && text.contains("1,234.56"),
        "Expected currency format with $ and 1,234.56, got: {text}"
    );
}

#[test]
fn test_number_format_keeps_quoted_currency_suffix() {
    // The quoted euro literal after the digits was dropped, printing
    // "1,240.00" instead of Excel's "1,240.00 €" (issue #365).
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(1240.0f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code("#,##0.00\" €\"");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "1,240.00 €");
}

#[test]
fn test_number_format_rounds_half_away_from_zero() {
    // Excel rounds display values; the formatter truncated 107310.6 with
    // #,##0 to 107,310 (issue #363).
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(107310.6f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code("#,##0");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "107,311");
}

#[test]
fn test_number_format_pads_short_fractions_to_two_decimals() {
    // 39.1 with a two-decimal format rendered "39.100" (issue #364).
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(39.1f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code("0.00");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "39.10");
}

#[test]
fn test_percentage_format_rounds_decimal_tie_like_excel() {
    // 21,300 / 20,000 = 1.065 displays as 107% in Excel: the display value
    // is rounded as a decimal, not as the binary double 106.4999… (#363).
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(1.065f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code(umya_spreadsheet::NumberingFormat::FORMAT_PERCENTAGE);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "107%");
}

#[test]
fn test_number_format_percentage() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(0.456f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code(umya_spreadsheet::NumberingFormat::FORMAT_PERCENTAGE);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let text = cell_text(&tp.table.rows[0].cells[0]);
    assert!(
        text.contains('%'),
        "Expected percentage format with %, got: {text}"
    );
}

#[test]
fn test_number_format_percentage_with_decimals() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(0.5f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code(umya_spreadsheet::NumberingFormat::FORMAT_PERCENTAGE_00);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let text = cell_text(&tp.table.rows[0].cells[0]);
    assert!(
        text.contains('%') && text.contains("50.00"),
        "Expected 50.00%, got: {text}"
    );
}

#[test]
fn test_number_format_date() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(45306f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code(umya_spreadsheet::NumberingFormat::FORMAT_DATE_YYYYMMDD);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let text = cell_text(&tp.table.rows[0].cells[0]);
    assert!(
        text.contains('-') && !text.contains("45306"),
        "Expected date format yyyy-mm-dd, got: {text}"
    );
}

#[test]
fn test_number_format_thousands_separator() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(1234567f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code("#,##0");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let text = cell_text(&tp.table.rows[0].cells[0]);
    assert_eq!(text, "1,234,567", "Expected thousands separator formatting");
}

#[test]
fn test_number_format_general_unchanged() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("42");
        sheet.get_cell_mut("B1").set_value("3.14");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(cell_text(&tp.table.rows[0].cells[0]), "42");
    assert_eq!(cell_text(&tp.table.rows[0].cells[1]), "3.14");
}

#[test]
fn test_number_format_builtin_id() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(1234.5f64);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_number_format_id(4);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let text = cell_text(&tp.table.rows[0].cells[0]);
    assert!(
        text.contains("1,234") && text.contains("50"),
        "Expected #,##0.00 formatting via ID 4, got: {text}"
    );
}

#[test]
fn test_number_format_custom_format_string() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(std::f64::consts::PI);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code("0.000");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let text = cell_text(&tp.table.rows[0].cells[0]);
    assert_eq!(text, "3.142", "Expected 3 decimal places formatting");
}

#[test]
fn test_cell_combined_formatting() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Full");
        let style = cell.get_style_mut();
        let font = style.get_font_mut();
        font.set_bold(true);
        font.set_size(16.0);
        font.set_name("Helvetica");
        font.get_color_mut().set_argb("FF0000FF");
        style.set_background_color("FFFFCC00");
        let borders = style.get_borders_mut();
        borders
            .get_left_mut()
            .set_border_style(umya_spreadsheet::Border::BORDER_THICK);
        borders.get_left_mut().get_color_mut().set_argb("FF00FF00");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = &tp.table.rows[0].cells[0];
    let style = first_run_style(cell);
    assert_eq!(style.bold, Some(true));
    assert_eq!(style.font_size, Some(16.0));
    assert_eq!(style.font_family.as_deref(), Some("Helvetica"));
    assert_eq!(style.color, Some(Color::new(0, 0, 255)));
    assert_eq!(cell.background, Some(Color::new(255, 204, 0)));
    let border = cell.border.as_ref().expect("Expected border");
    let left = border.left.as_ref().expect("Expected left border");
    // `thick` (issue #487).
    assert!((left.width - 2.5).abs() < 0.01);
    assert_eq!(left.color, Color::new(0, 255, 0));
}

#[test]
fn test_cell_without_underline_style_is_not_underlined() {
    // A font entry with other properties (e.g. bold) but no <u> element must
    // not inherit a spurious underline from the library's enum default.
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Plain");
        cell.get_style_mut().get_font_mut().set_bold(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let style = first_run_style(&tp.table.rows[0].cells[0]);
    assert_eq!(style.underline, None);
}

#[test]
fn test_cell_explicit_underline_is_applied() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Underlined");
        cell.get_style_mut().get_font_mut().set_underline("single");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let style = first_run_style(&tp.table.rows[0].cells[0]);
    assert_eq!(style.underline, Some(true));
}

#[test]
fn test_cell_underline_none_is_not_underlined() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("NoUnderline");
        cell.get_style_mut().get_font_mut().set_underline("none");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let style = first_run_style(&tp.table.rows[0].cells[0]);
    assert_eq!(style.underline, None);
}

#[test]
fn test_cell_horizontal_center_alignment_applied() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Centered");
        cell.get_style_mut()
            .get_alignment_mut()
            .set_horizontal(umya_spreadsheet::HorizontalAlignmentValues::Center);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = &tp.table.rows[0].cells[0];
    let Block::Paragraph(paragraph) = &cell.content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(paragraph.style.alignment, Some(Alignment::Center));
}

#[test]
fn test_cell_horizontal_right_alignment_applied() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Right");
        cell.get_style_mut()
            .get_alignment_mut()
            .set_horizontal(umya_spreadsheet::HorizontalAlignmentValues::Right);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let Block::Paragraph(paragraph) = &tp.table.rows[0].cells[0].content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(paragraph.style.alignment, Some(Alignment::Right));
}

#[test]
fn test_cell_vertical_center_alignment_applied() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("Middle");
        cell.get_style_mut()
            .get_alignment_mut()
            .set_vertical(umya_spreadsheet::VerticalAlignmentValues::Center);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].cells[0].vertical_align,
        Some(CellVerticalAlign::Center)
    );
}

#[test]
fn test_cell_without_alignment_keeps_default() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("Plain");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let Block::Paragraph(paragraph) = &tp.table.rows[0].cells[0].content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(paragraph.style.alignment, None);
    assert_eq!(tp.table.rows[0].cells[0].vertical_align, None);
}

#[test]
fn test_percent_format_keeps_decimal_precision() {
    // A cached formula ratio formatted as "0.0%" must not round to an
    // integer first (0.17309... rendered "17.0%" instead of "17.3%").
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(0.1730909090909091);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code("0.0%");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let Block::Paragraph(paragraph) = &tp.table.rows[0].cells[0].content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(paragraph.runs[0].text, "17.3%");
}

// ----- In-cell rich text runs (issue #275) -----

/// Helper: build a rich text value like the classified workbook's headings —
/// a bold label run followed by a plain continuation run.
fn build_rich_text_cell(setup: impl FnOnce(&mut umya_spreadsheet::Worksheet)) -> Vec<u8> {
    build_xlsx_formatted(setup)
}

#[test]
fn test_rich_text_runs_keep_per_run_formatting() {
    let data = build_rich_text_cell(|sheet| {
        let mut rich = umya_spreadsheet::RichText::default();

        let mut bold_run = umya_spreadsheet::TextElement::default();
        bold_run.set_text("지원율 ");
        {
            let font = bold_run.get_run_properties_mut();
            font.set_bold(true);
            font.set_size(14.0);
            font.get_color_mut().set_argb("FFC00000");
            font.set_name("Arial");
        }
        rich.add_rich_text_elements(bold_run);

        let mut plain_run = umya_spreadsheet::TextElement::default();
        plain_run.set_text("(최근 3년)");
        rich.add_rich_text_elements(plain_run);

        sheet
            .get_cell_mut("A1")
            .get_cell_value_mut()
            .set_rich_text(rich);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let Block::Paragraph(paragraph) = &tp.table.rows[0].cells[0].content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(
        paragraph.runs.len(),
        2,
        "each rich text run must become its own IR run"
    );

    let bold_run = &paragraph.runs[0];
    assert_eq!(bold_run.text, "지원율 ");
    assert_eq!(bold_run.style.bold, Some(true));
    assert_eq!(bold_run.style.font_size, Some(14.0));
    assert_eq!(bold_run.style.font_family.as_deref(), Some("Arial"));
    assert_eq!(
        bold_run.style.color,
        Some(Color {
            r: 0xC0,
            g: 0x00,
            b: 0x00
        })
    );

    let plain_run = &paragraph.runs[1];
    assert_eq!(plain_run.text, "(최근 3년)");
    assert_eq!(plain_run.style.bold, None, "unstyled run stays regular");
    // The run keeps the cell's inherited workbook Normal font rather than the
    // styled run's 14pt; that font is now carried explicitly (issue #462).
    assert_eq!(plain_run.style.font_size, Some(11.0));
}

#[test]
fn test_rich_text_unstyled_run_inherits_cell_style() {
    // Cell-level style is 12pt green italic; a rich run without its own
    // properties must inherit it, while a styled run overrides per-property.
    let data = build_rich_text_cell(|sheet| {
        let mut rich = umya_spreadsheet::RichText::default();

        let mut styled_run = umya_spreadsheet::TextElement::default();
        styled_run.set_text("34.8%");
        // Excel writes minimal <rPr> with only the changed property — build the
        // font from empty instead of get_run_properties_mut(), which seeds the
        // library's full default font (explicit sz=11/Calibri).
        let mut bold_only_font = umya_spreadsheet::Font::default();
        bold_only_font.set_bold(true);
        styled_run.set_run_properties(bold_only_font);
        rich.add_rich_text_elements(styled_run);

        let mut plain_run = umya_spreadsheet::TextElement::default();
        plain_run.set_text(" 달성");
        rich.add_rich_text_elements(plain_run);

        let cell = sheet.get_cell_mut("B2");
        cell.get_cell_value_mut().set_rich_text(rich);
        let font = cell.get_style_mut().get_font_mut();
        font.set_size(12.0);
        font.set_italic(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();

    let tp = get_sheet_page(&doc, 0);
    let cell = tp
        .table
        .rows
        .iter()
        .flat_map(|r| r.cells.iter())
        .find(|c| !c.content.is_empty())
        .expect("cell with content");
    let Block::Paragraph(paragraph) = &cell.content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(paragraph.runs.len(), 2);

    let styled_run = &paragraph.runs[0];
    assert_eq!(styled_run.style.bold, Some(true));
    assert_eq!(
        styled_run.style.font_size,
        Some(12.0),
        "run without explicit size keeps the cell size"
    );
    assert_eq!(styled_run.style.italic, Some(true));

    let plain_run = &paragraph.runs[1];
    assert_eq!(plain_run.style.font_size, Some(12.0));
    assert_eq!(plain_run.style.italic, Some(true));
    assert_eq!(plain_run.style.bold, None);
}

// ----- Text spill into adjacent empty cells (issue #293) -----

#[test]
fn test_long_text_spills_over_empty_neighbors() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value(
            "전 직원 통일 방식으로 운영 시, 최소 주 2회 이상의 수업을 제공하는 기관과 제휴",
        );
        // B1..C1 empty, D1 occupied — the spill must stop before D1.
        sheet.get_cell_mut("D1").set_value("차단");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);

    let cell = &tp.table.rows[0].cells[0];
    let spill_width = cell
        .spill_width
        .expect("long unwrapped text with empty right neighbors should spill");
    let own_width = tp.table.column_widths[0];
    let three_columns: f64 = tp.table.column_widths[..3].iter().sum();
    assert!(
        (spill_width - three_columns).abs() < 0.5,
        "spill should cover A..C ({three_columns}pt), got {spill_width}pt (own {own_width}pt)"
    );
}

#[test]
fn test_short_text_does_not_spill() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("짧음");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].cells[0].spill_width, None);
}

#[test]
fn test_wrap_text_disables_spill() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value(
            "전 직원 통일 방식으로 운영 시, 최소 주 2회 이상의 수업을 제공하는 기관과 제휴",
        );
        cell.get_style_mut().get_alignment_mut().set_wrap_text(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].cells[0].spill_width, None,
        "explicit wrapText must wrap inside the cell, not spill"
    );
}

#[test]
fn test_occupied_neighbor_blocks_spill() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value(
            "전 직원 통일 방식으로 운영 시, 최소 주 2회 이상의 수업을 제공하는 기관과 제휴",
        );
        sheet.get_cell_mut("B1").set_value("옆");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].cells[0].spill_width, None,
        "an occupied right neighbor leaves nothing to spill into"
    );
}

#[test]
fn test_merged_cell_clips_at_merge_edge_instead_of_wrapping() {
    let data = build_xlsx_with_merges(
        "Sheet1",
        &[(
            "A1",
            "전 직원 통일 방식으로 운영 시, 최소 주 2회 이상의 수업을 제공하는 기관과 제휴",
        )],
        &["A1:C1"],
    );
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);

    let cell = &tp.table.rows[0].cells[0];
    let merged_width: f64 = tp.table.column_widths[..3].iter().sum();
    let spill_width = cell
        .spill_width
        .expect("overflowing unwrapped text in a merge should clip, not wrap");
    assert!(
        (spill_width - merged_width).abs() < 0.5,
        "clip width should equal the merged width {merged_width}pt, got {spill_width}pt"
    );
}

// ----- Default bottom vertical alignment (issue #298) -----

#[test]
fn test_sheet_table_defaults_to_bottom_vertical_alignment() {
    // Excel's default cell vertical alignment is bottom; sheets must carry
    // that down to the renderer so text sits at the row bottom like Excel
    // prints it.
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("바닥 정렬");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.default_vertical_align,
        Some(crate::ir::CellVerticalAlign::Bottom)
    );
}

// ----- Explicit print margins (issue #300) -----

#[test]
fn test_explicit_page_margins_are_used() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("여백 테스트");
        let margins = sheet.get_page_margins_mut();
        margins.set_top(1.0);
        margins.set_bottom(1.0);
        margins.set_left(0.75);
        margins.set_right(0.75);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.margins.top, 72.0, "1in top margin must be honored");
    assert_eq!(tp.margins.bottom, 72.0);
    assert_eq!(tp.margins.left, 54.0);
    assert_eq!(tp.margins.right, 54.0);
}

#[test]
fn test_absent_page_margins_fall_back_to_excel_defaults() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("기본 여백");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.margins.top, 54.0, "Excel default 0.75in top");
    assert_eq!(tp.margins.left, 50.4, "Excel default 0.7in left");
}

// ----- Row heights without customHeight (issue #303) -----

#[test]
fn test_row_height_used_even_without_custom_height_flag() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("행 높이");
        let row = sheet.get_row_dimension_mut(&1);
        row.set_height(20.0);
        row.set_custom_height(false);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].height,
        Some(18.0),
        "recorded ht is calibrated to the native PDF track even without customHeight"
    );
}

#[test]
fn test_row_without_dimension_uses_sheet_default_height() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("첫째 줄");
        sheet.get_cell_mut("A2").set_value("둘째 줄");
        sheet
            .get_sheet_format_properties_mut()
            .set_default_row_height(18.0);
        sheet.get_row_dimension_mut(&1).set_height(24.0);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].height, Some(22.0));
    assert_eq!(
        tp.table.rows[1].height,
        Some(17.0),
        "rows without their own ht use the calibrated sheet defaultRowHeight"
    );
}

#[test]
fn test_wrapping_row_uses_the_height_excel_recorded() {
    // A wrapping row used to be left content-driven whenever `customHeight`
    // was false, on the theory that Excel re-grows it on open and a fixed
    // height could clip a line. A native Excel export of
    // `office2pdf_repository_workbook.xlsx` disproves that: its data rows all
    // carry `ht="15"` with `customHeight="false"` and wrapped cells, and Excel
    // prints them at exactly that height. Leaving them auto printed the
    // workbook on 25 pages against Excel's 23 (issue #608).
    //
    // Excel writes back the height it auto-fitted, so the recorded value
    // already covers however many lines the cell wraps to; the renderer
    // divides it by that line count rather than clipping.
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("줄바꿈이 있는 긴 텍스트가 이 셀에 들어 있습니다");
        cell.get_style_mut().get_alignment_mut().set_wrap_text(true);
        let row = sheet.get_row_dimension_mut(&1);
        row.set_height(30.0);
        row.set_custom_height(false);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].height,
        Some(28.0),
        "a wrapping row prints at the height Excel recorded for it"
    );
}

#[test]
fn test_row_without_recorded_height_and_wrapping_cell_stays_auto() {
    // The auto path still applies where it is the only sensible answer: no
    // recorded `ht` at all, so there is nothing to honour and the content has
    // to decide.
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("줄바꿈이 있는 긴 텍스트가 이 셀에 들어 있습니다");
        cell.get_style_mut().get_alignment_mut().set_wrap_text(true);
        let row = sheet.get_row_dimension_mut(&1);
        row.set_height(0.0);
        row.set_custom_height(false);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].height, None,
        "a wrapping row with no recorded height stays content-driven"
    );
}

#[test]
fn test_wrapping_row_with_custom_height_stays_fixed() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("줄바꿈이 있는 긴 텍스트가 이 셀에 들어 있습니다");
        cell.get_style_mut().get_alignment_mut().set_wrap_text(true);
        let row = sheet.get_row_dimension_mut(&1);
        row.set_height(30.0);
        row.set_custom_height(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.rows[0].height, Some(28.0));
}

#[test]
fn test_native_excel_print_height_calibrates_custom_title_rows() {
    // Native Excel PDF output measures a 25.5pt worksheet row as a 23pt
    // printed track. Keeping the raw OOXML value makes both the title row
    // and the table header too tall and pushes the data block down.
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("Dashboard title");
        let row = sheet.get_row_dimension_mut(&1);
        row.set_height(25.5);
        row.set_custom_height(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].height,
        Some(23.0),
        "25.5pt custom rows must match Excel's 23pt PDF track"
    );
}

#[test]
fn test_native_excel_print_height_calibrates_default_rows() {
    // The same native print path measures the workbook's default 15pt row
    // as a 14pt track. This matters for blank spacer rows because they have
    // no content from which Typst could derive the native height.
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("Title");
        sheet.get_cell_mut("A3").set_value("Header");
        sheet
            .get_sheet_format_properties_mut()
            .set_default_row_height(15.0);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[1].height,
        Some(14.0),
        "blank default rows must match Excel's 14pt PDF track"
    );
}

#[test]
fn test_xlsx_auto_row_padding_matches_native_14pt_track() {
    // Arial 10's full hhea line plus Excel's asymmetric 1pt/1.5pt vertical
    // insets forms the 14pt single-line track measured in native output.
    // Keeping only 1pt below makes every later row drift upward by ~0.5pt.
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("North America");
        cell.get_style_mut().get_alignment_mut().set_wrap_text(true);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let padding = get_sheet_page(&doc, 0)
        .table
        .default_cell_padding
        .expect("XLSX table padding");
    assert_eq!(padding.top, 1.0);
    assert_eq!(padding.bottom, 1.5);
}

// ----- Print titles (issue #234) -----

/// Helper: build a workbook whose sheet declares `_xlnm.Print_Titles`.
fn build_xlsx_with_print_titles(
    address: &str,
    setup: impl FnOnce(&mut umya_spreadsheet::Worksheet),
) -> Vec<u8> {
    build_xlsx_formatted(|sheet| {
        setup(sheet);
        sheet
            .add_defined_name("_xlnm.Print_Titles", address)
            .unwrap();
    })
}

#[test]
fn test_print_title_rows_become_repeating_header() {
    let data = build_xlsx_with_print_titles("Sheet1!$1:$2", |sheet| {
        sheet.get_cell_mut("A1").set_value("제목 1");
        sheet.get_cell_mut("A2").set_value("제목 2");
        sheet.get_cell_mut("A3").set_value("데이터");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.header_row_count, 2,
        "title rows $1:$2 must repeat as the table header"
    );
}

/// Excel repeats only the rows named by `_xlnm.Print_Titles`. When the range
/// starts below the sheet top, the rows above it print once.
#[test]
fn test_print_titles_below_sheet_top_do_not_repeat_the_rows_above() {
    let data = build_xlsx_with_print_titles("Sheet1!$3:$3", |sheet| {
        sheet
            .get_cell_mut("A1")
            .set_value("Warehouse Inventory Snapshot");
        sheet.get_cell_mut("A3").set_value("SKU");
        sheet.get_cell_mut("A4").set_value("SKU-1000");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.header_row_count, 1,
        "only row 3 repeats on later pages"
    );
    assert_eq!(
        tp.table.non_repeating_header_row_count, 2,
        "rows 1-2 lead the table but must not repeat"
    );
}

/// A title range starting at the sheet top has nothing above it to hold back.
#[test]
fn test_print_titles_at_sheet_top_have_no_non_repeating_rows() {
    let data = build_xlsx_with_print_titles("Sheet1!$1:$2", |sheet| {
        sheet.get_cell_mut("A1").set_value("Title");
        sheet.get_cell_mut("A2").set_value("Header");
        sheet.get_cell_mut("A3").set_value("Data");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.header_row_count, 2);
    assert_eq!(tp.table.non_repeating_header_row_count, 0);
}

#[test]
fn test_no_print_titles_means_no_header() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("데이터");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.header_row_count, 0);
}

#[test]
fn test_print_title_columns_repeat_on_overflow_pages() {
    // Column A is a title column; enough wide columns follow to force
    // column pagination. Every overflow page must start with column A.
    let data = build_xlsx_with_print_titles("Sheet1!$A:$A", |sheet| {
        sheet.get_cell_mut("A1").set_value("이름");
        for col in 2..=12u32 {
            let cell = sheet.get_cell_mut((col, 1));
            cell.set_value(format!("값{col}"));
        }
        for col in 1..=12u32 {
            sheet
                .get_column_dimension_by_number_mut(&col)
                .set_width(30.0);
        }
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    assert!(doc.pages.len() >= 2, "wide sheet must paginate by columns");

    for (page_idx, page) in doc.pages.iter().enumerate().skip(1) {
        let Page::Sheet(sp) = page else {
            panic!("expected sheet page");
        };
        let first_cell_text = cell_text(&sp.table.rows[0].cells[0]);
        assert_eq!(
            first_cell_text, "이름",
            "page {page_idx} must repeat the title column"
        );
    }
}

#[test]
fn test_print_titles_with_both_rows_and_columns() {
    // Mirrors `Sheet4!$A:$B,Sheet4!$2:$3`-style definitions with two parts.
    let data = build_xlsx_with_print_titles("Sheet1!$A:$A,Sheet1!$1:$1", |sheet| {
        sheet.get_cell_mut("A1").set_value("이름");
        for col in 2..=12u32 {
            sheet.get_cell_mut((col, 1)).set_value(format!("값{col}"));
        }
        sheet.get_cell_mut("A2").set_value("둘째");
        for col in 1..=12u32 {
            sheet
                .get_column_dimension_by_number_mut(&col)
                .set_width(30.0);
        }
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(tp.table.header_row_count, 1, "row titles parsed");
    assert!(doc.pages.len() >= 2);
    let Page::Sheet(sp) = &doc.pages[1] else {
        panic!("expected sheet page");
    };
    assert_eq!(
        cell_text(&sp.table.rows[0].cells[0]),
        "이름",
        "column titles parsed from the multi-part address"
    );
}

// ----- RTL text presentation (issue #236) -----

#[test]
fn test_rtl_text_right_aligns_under_general_alignment() {
    let data = build_xlsx_formatted(|sheet| {
        sheet.get_cell_mut("A1").set_value("نص");
        sheet.get_cell_mut("A2").set_value("עִבְרִית");
        sheet.get_cell_mut("A3").set_value("text");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);

    let alignment_of = |row: usize| match &tp.table.rows[row].cells[0].content[0] {
        Block::Paragraph(p) => p.style.alignment,
        _ => panic!("expected paragraph"),
    };
    assert_eq!(
        alignment_of(0),
        Some(crate::ir::Alignment::Right),
        "Arabic text under general alignment renders right-aligned in Excel"
    );
    assert_eq!(alignment_of(1), Some(crate::ir::Alignment::Right));
    assert_eq!(alignment_of(2), None, "Latin text keeps the default");
}

#[test]
fn test_explicit_alignment_wins_over_rtl_text() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value("نص");
        cell.get_style_mut()
            .get_alignment_mut()
            .set_horizontal(umya_spreadsheet::HorizontalAlignmentValues::Left);
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    let Block::Paragraph(p) = &tp.table.rows[0].cells[0].content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(p.style.alignment, Some(crate::ir::Alignment::Left));
}

#[test]
fn test_native_digit_locale_format_renders_arabic_indic_digits() {
    let data = build_xlsx_formatted(|sheet| {
        let cell = sheet.get_cell_mut("A1");
        cell.set_value_number(123.0);
        cell.get_style_mut()
            .get_number_format_mut()
            .set_format_code("[$-3000401]0");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    let Block::Paragraph(p) = &tp.table.rows[0].cells[0].content[0] else {
        panic!("expected paragraph");
    };
    assert_eq!(
        p.runs[0].text, "١٢٣",
        "native-digit locale prefix must map to Arabic-Indic digits"
    );
}

// ----- Spill past the used range (issue #309) -----

#[test]
fn test_spill_extends_past_used_range_over_virtual_cells() {
    // Single used column: Excel paints the text across the virtual empty
    // cells to the right instead of wrapping inside column A.
    let data = build_xlsx_formatted(|sheet| {
        sheet
            .get_cell_mut("A1")
            .set_value("text with comment stretching well past column A");
        sheet.get_cell_mut("A2").set_value("둘째");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);

    let cell = &tp.table.rows[0].cells[0];
    let own_width = tp.table.column_widths[0];
    let spill_width = cell
        .spill_width
        .expect("text must spill past the used range");
    assert!(
        spill_width > own_width * 2.0,
        "spill ({spill_width}pt) must extend well past the single {own_width}pt column"
    );
}

#[test]
fn test_spill_still_blocked_by_occupied_neighbor() {
    let data = build_xlsx_formatted(|sheet| {
        sheet
            .get_cell_mut("A1")
            .set_value("text with comment stretching well past column A");
        sheet.get_cell_mut("B1").set_value("차단");
    });
    let parser = XlsxParser;
    let (doc, _warnings) = parser.parse(&data, &ConvertOptions::default()).unwrap();
    let tp = get_sheet_page(&doc, 0);
    assert_eq!(
        tp.table.rows[0].cells[0].spill_width, None,
        "an occupied neighbor still blocks the spill"
    );
}
