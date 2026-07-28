use super::*;

#[test]
fn test_generate_flow_page_with_text_header() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body text")],
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Document Title".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("header:"));
    assert!(output.source.contains("Document Title"));
}

#[test]
fn test_generate_flow_page_with_page_number_footer() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body text")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: Some(35.4),
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![
                    HFInline::Run(Run {
                        text: "Page ".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }),
                    HFInline::PageNumber(TextStyle::default()),
                ],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("footer:"));
    assert!(output.source.contains(r#"counter(page).display("1")"#));
    assert!(output.source.contains("Page "));
    // Word pins the footer's bottom `w:footer` points above the page edge.
    assert!(output.source.contains("footer-descent: 0pt"));
    assert!(output.source.contains("block(width: 100%, height: 36.6pt)"));
    assert!(output.source.contains("place(bottom"));
}

#[test]
fn test_generate_footer_with_compound_border_and_right_positioned_tab() {
    use crate::ir::{
        BorderSide, CellBorder, HFInline, HeaderFooter, HeaderFooterParagraph, PositionedTab,
        PositionedTabAlignment, PositionedTabRelativeTo,
    };

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![
                    HFInline::Run(Run {
                        text: "Left".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }),
                    HFInline::PositionedTab(PositionedTab {
                        alignment: PositionedTabAlignment::Right,
                        relative_to: PositionedTabRelativeTo::Margin,
                        leader: TabLeader::None,
                    }),
                    HFInline::Run(Run {
                        text: "Page ".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }),
                    HFInline::PageNumber(TextStyle::default()),
                ],
                border: Some(CellBorder {
                    top: Some(BorderSide {
                        width: 3.0,
                        color: Color::new(0x62, 0x24, 0x23),
                        style: BorderLineStyle::Double,
                    }),
                    bottom: None,
                    left: None,
                    right: None,
                }),
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("#grid(columns: (1fr, auto)"));
    assert!(output.source.contains("rgb(98, 36, 35)"));
    assert_eq!(output.source.matches("line(length: 100%").count(), 2);
}

#[test]
fn test_generate_page_anchored_footer_frame_in_foreground() {
    use crate::ir::{
        FrameAnchor, HFInline, HeaderFooter, HeaderFooterFrame, HeaderFooterParagraph,
    };

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Framed footer".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: Some(HeaderFooterFrame {
                    x: Some(71.8),
                    y: Some(198.5),
                    width: None,
                    height: None,
                    horizontal_anchor: FrameAnchor::Page,
                    vertical_anchor: FrameAnchor::Page,
                }),
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("foreground: ["));
    assert!(
        output
            .source
            .contains("#place(top + left, dx: 71.8pt, dy: 198.5pt)")
    );
    assert!(!output.source.contains("footer:"));
}

#[test]
fn test_generate_flow_page_with_header_and_footer() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Header".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        footer: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::PageNumber(TextStyle::default())],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("header:") && output.source.contains("footer:"));
}

#[test]
fn test_generate_flow_page_without_header_footer() {
    let doc = make_doc(vec![make_flow_page(vec![make_paragraph("Body")])]);
    let output = generate_typst(&doc).unwrap();
    assert!(!output.source.contains("header:"));
    assert!(!output.source.contains("footer:"));
}

#[test]
fn test_generate_typst_inserts_pagebreak_between_flow_pages() {
    let first = Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("First section")],
        header: None,
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    });
    let second = Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Second section")],
        header: None,
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    });

    let output = generate_typst(&make_doc(vec![first, second])).unwrap();
    let pagebreak_count = output.source.matches("#pagebreak()").count();

    assert_eq!(pagebreak_count, 1);
}

#[test]
fn test_fixed_page_with_background_color() {
    let page = Page::Fixed(FixedPage {
        size: PageSize {
            width: 720.0,
            height: 540.0,
        },
        elements: vec![],
        background_color: Some(Color::new(255, 0, 0)),
        background_gradient: None,
    });
    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("fill: rgb(255, 0, 0)"));
}

#[test]
fn test_fixed_page_without_background_color() {
    let page = Page::Fixed(FixedPage {
        size: PageSize {
            width: 720.0,
            height: 540.0,
        },
        elements: vec![],
        background_color: None,
        background_gradient: None,
    });
    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("fill: white"),
        "Expected fill: white for no-background slide, got:\n{}",
        output.source
    );
}

#[test]
fn test_fixed_page_table_element() {
    let table = Table {
        rows: vec![TableRow {
            cells: vec![
                TableCell {
                    content: vec![Block::Paragraph(Paragraph {
                        style: ParagraphStyle::default(),
                        runs: vec![Run {
                            text: "A1".to_string(),
                            style: TextStyle::default(),
                            href: None,
                            footnote: None,
                        }],
                    })],
                    ..TableCell::default()
                },
                TableCell {
                    content: vec![Block::Paragraph(Paragraph {
                        style: ParagraphStyle::default(),
                        runs: vec![Run {
                            text: "B1".to_string(),
                            style: TextStyle::default(),
                            href: None,
                            footnote: None,
                        }],
                    })],
                    ..TableCell::default()
                },
            ],
            height: None,
        }],
        column_widths: vec![100.0, 100.0],
        ..Table::default()
    };

    let page = Page::Fixed(FixedPage {
        size: PageSize {
            width: 720.0,
            height: 540.0,
        },
        elements: vec![FixedElement {
            x: 50.0,
            y: 100.0,
            width: 200.0,
            height: 50.0,
            kind: FixedElementKind::Table(table),
        }],
        background_color: None,
        background_gradient: None,
    });

    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();

    assert!(
        output
            .source
            .contains("#place(top + left, dx: 50pt, dy: 100pt)")
    );
    assert!(output.source.contains("#table("));
    assert!(output.source.contains("columns: (100pt, 100pt)"));
    assert!(output.source.contains("A1"));
    assert!(output.source.contains("B1"));
}

#[test]
fn test_hyperlink_generates_typst_link() {
    let doc = make_doc(vec![make_flow_page(vec![Block::Paragraph(Paragraph {
        style: ParagraphStyle::default(),
        runs: vec![Run {
            text: "Click me".to_string(),
            style: TextStyle::default(),
            href: Some("https://example.com".to_string()),
            footnote: None,
        }],
    })])]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output
            .source
            .contains(r#"#link("https://example.com")[Click me]"#)
    );
}

#[test]
fn test_hyperlink_with_styled_text() {
    let doc = make_doc(vec![make_flow_page(vec![Block::Paragraph(Paragraph {
        style: ParagraphStyle::default(),
        runs: vec![Run {
            text: "Bold link".to_string(),
            style: TextStyle {
                bold: Some(true),
                ..TextStyle::default()
            },
            href: Some("https://example.com".to_string()),
            footnote: None,
        }],
    })])]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains(r#"#link("https://example.com")["#));
    assert!(output.source.contains("#text(weight: \"bold\")"));
}

#[test]
fn test_hyperlink_mixed_with_plain_text() {
    let doc = make_doc(vec![make_flow_page(vec![Block::Paragraph(Paragraph {
        style: ParagraphStyle::default(),
        runs: vec![
            Run {
                text: "Visit ".to_string(),
                style: TextStyle::default(),
                href: None,
                footnote: None,
            },
            Run {
                text: "Rust".to_string(),
                style: TextStyle::default(),
                href: Some("https://rust-lang.org".to_string()),
                footnote: None,
            },
            Run {
                text: " for more.".to_string(),
                style: TextStyle::default(),
                href: None,
                footnote: None,
            },
        ],
    })])]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("Visit "));
    assert!(
        output
            .source
            .contains(r#"#link("https://rust-lang.org")[Rust]"#)
    );
    assert!(output.source.contains(" for more."));
}

#[test]
fn test_hyperlink_url_with_special_chars_escaped() {
    let doc = make_doc(vec![make_flow_page(vec![Block::Paragraph(Paragraph {
        style: ParagraphStyle::default(),
        runs: vec![Run {
            text: "Link".to_string(),
            style: TextStyle::default(),
            href: Some("https://example.com/path?q=1&r=2".to_string()),
            footnote: None,
        }],
    })])]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output
            .source
            .contains(r#"#link("https://example.com/path?q=1&r=2")[Link]"#)
    );
}

/// A note's content run: a note carries styled runs, not one string.
fn note_run(text: &str) -> Run {
    Run {
        text: text.to_string(),
        style: TextStyle::default(),
        href: None,
        footnote: None,
    }
}

#[test]
fn test_footnote_generates_typst_footnote() {
    let doc = make_doc(vec![make_flow_page(vec![Block::Paragraph(Paragraph {
        style: ParagraphStyle::default(),
        runs: vec![
            Run {
                text: "Some text".to_string(),
                style: TextStyle::default(),
                href: None,
                footnote: None,
            },
            Run {
                text: String::new(),
                style: TextStyle::default(),
                href: None,
                footnote: Some(vec![note_run("This is a footnote.")]),
            },
        ],
    })])]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("#footnote[This is a footnote.]"));
}

#[test]
fn test_footnote_with_special_chars() {
    let doc = make_doc(vec![make_flow_page(vec![Block::Paragraph(Paragraph {
        style: ParagraphStyle::default(),
        runs: vec![Run {
            text: String::new(),
            style: TextStyle::default(),
            href: None,
            footnote: Some(vec![note_run("Note with #special *chars*")]),
        }],
    })])]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output
            .source
            .contains(r"#footnote[Note with \#special \*chars\*]")
    );
}

#[test]
fn test_table_page_with_header() {
    let page = Page::Sheet(SheetPage {
        name: "Sheet1".to_string(),
        size: PageSize::default(),
        margins: Margins::default(),
        table: make_simple_table(vec![vec!["A"]]),
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle {
                    alignment: Some(Alignment::Center),
                    ..ParagraphStyle::default()
                },
                elements: vec![HFInline::Run(Run {
                    text: "My Header".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        footer: None,
        charts: vec![],
        images: Vec::new(),
        text_boxes: Vec::new(),
    });
    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("header: ["));
    assert!(output.source.contains("My Header"));
}

#[test]
fn test_table_page_with_page_number_footer() {
    let page = Page::Sheet(SheetPage {
        name: "Sheet1".to_string(),
        size: PageSize::default(),
        margins: Margins::default(),
        table: make_simple_table(vec![vec!["A"]]),
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle {
                    alignment: Some(Alignment::Center),
                    ..ParagraphStyle::default()
                },
                elements: vec![
                    HFInline::Run(Run {
                        text: "Page ".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }),
                    HFInline::PageNumber(TextStyle::default()),
                    HFInline::Run(Run {
                        text: " of ".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }),
                    HFInline::TotalPages(TextStyle::default()),
                ],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        charts: vec![],
        images: Vec::new(),
        text_boxes: Vec::new(),
    });
    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("footer: context ["));
    assert!(output.source.contains(r#"#counter(page).display("1")"#));
    assert!(output.source.contains("#counter(page).final().first()"));
}

#[test]
fn test_table_page_no_header_footer() {
    let page = Page::Sheet(SheetPage {
        name: "Sheet1".to_string(),
        size: PageSize::default(),
        margins: Margins::default(),
        table: make_simple_table(vec![vec!["A"]]),
        header: None,
        footer: None,
        charts: vec![],
        images: Vec::new(),
        text_boxes: Vec::new(),
    });
    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();
    assert!(!output.source.contains("header:"));
    assert!(!output.source.contains("footer:"));
}

#[test]
fn test_table_page_with_chart_at_row() {
    use crate::ir::{Chart, ChartGrouping, ChartSeries, ChartType, DataLabels, LegendPosition};

    let chart = Chart {
        chart_type: ChartType::Bar,
        title: Some("Sales".to_string()),
        categories: vec!["Q1".to_string(), "Q2".to_string()],
        series: vec![ChartSeries {
            name: Some("Revenue".to_string()),
            values: vec![100.0, 200.0],
            fill: None,
            point_fills: Vec::new(),
            data_labels: DataLabels::default(),
        }],
        grouping: ChartGrouping::Clustered,
        legend_position: LegendPosition::Right,
        category_axis_title: None,
        value_axis_title: None,
    };

    let page = Page::Sheet(SheetPage {
        name: "Sheet1".to_string(),
        size: PageSize::default(),
        margins: Margins::default(),
        table: make_simple_table(vec![
            vec!["Row 1"],
            vec!["Row 2"],
            vec!["Row 3"],
            vec!["Row 4"],
            vec!["Row 5"],
        ]),
        header: None,
        footer: None,
        charts: vec![(2, chart)],
        images: Vec::new(),
        text_boxes: Vec::new(),
    });

    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();
    let src = &output.source;

    assert_eq!(src.matches("#table(").count(), 2);
    assert!(src.contains("Sales"));
}

#[test]
fn test_table_page_with_chart_at_end() {
    use crate::ir::{Chart, ChartGrouping, ChartSeries, ChartType, DataLabels, LegendPosition};

    let chart = Chart {
        chart_type: ChartType::Pie,
        title: Some("Pie".to_string()),
        categories: vec!["A".to_string()],
        series: vec![ChartSeries {
            name: None,
            values: vec![100.0],
            fill: None,
            point_fills: Vec::new(),
            data_labels: DataLabels::default(),
        }],
        grouping: ChartGrouping::Clustered,
        legend_position: LegendPosition::Right,
        category_axis_title: None,
        value_axis_title: None,
    };

    let page = Page::Sheet(SheetPage {
        name: "Sheet1".to_string(),
        size: PageSize::default(),
        margins: Margins::default(),
        table: make_simple_table(vec![vec!["Data"]]),
        header: None,
        footer: None,
        charts: vec![(u32::MAX, chart)],
        images: Vec::new(),
        text_boxes: Vec::new(),
    });

    let doc = make_doc(vec![page]);
    let output = generate_typst(&doc).unwrap();
    let src = &output.source;

    let table_pos = src.find("#table(").unwrap();
    let chart_pos = src.find("Pie").unwrap();
    assert!(table_pos < chart_pos);
}

#[test]
fn test_paper_size_override_letter() {
    use crate::config::PaperSize;

    let doc = make_doc(vec![make_flow_page(vec![make_paragraph("Test")])]);
    let options = ConvertOptions {
        paper_size: Some(PaperSize::Letter),
        ..Default::default()
    };
    let output = generate_typst_with_options(&doc, &options).unwrap();
    assert!(output.source.contains("width: 612pt"));
    assert!(output.source.contains("height: 792pt"));
}

#[test]
fn test_landscape_override_swaps_dimensions() {
    let doc = make_doc(vec![make_flow_page(vec![make_paragraph("Test")])]);
    let options = ConvertOptions {
        landscape: Some(true),
        ..Default::default()
    };
    let output = generate_typst_with_options(&doc, &options).unwrap();
    assert!(output.source.contains("width: 841.89pt"));
    assert!(output.source.contains("height: 595.28pt"));
}

#[test]
fn test_portrait_override_keeps_portrait() {
    let doc = make_doc(vec![make_flow_page(vec![make_paragraph("Test")])]);
    let options = ConvertOptions {
        landscape: Some(false),
        ..Default::default()
    };
    let output = generate_typst_with_options(&doc, &options).unwrap();
    assert!(output.source.contains("width: 595.28pt"));
    assert!(output.source.contains("height: 841.89pt"));
}

#[test]
fn test_paper_size_with_landscape() {
    use crate::config::PaperSize;

    let doc = make_doc(vec![make_flow_page(vec![make_paragraph("Test")])]);
    let options = ConvertOptions {
        paper_size: Some(PaperSize::Letter),
        landscape: Some(true),
        ..Default::default()
    };
    let output = generate_typst_with_options(&doc, &options).unwrap();
    assert!(output.source.contains("width: 792pt"));
    assert!(output.source.contains("height: 612pt"));
}

#[test]
fn test_no_override_uses_original_size() {
    let doc = make_doc(vec![make_flow_page(vec![make_paragraph("Test")])]);
    let options = ConvertOptions::default();
    let output = generate_typst_with_options(&doc, &options).unwrap();
    assert!(output.source.contains("width: 595.28pt"));
}

/// Word letterhead headers commonly carry a `w:pBdr/w:bottom` rule under the
/// header text. The rule must render below the content, not be dropped.
#[test]
fn test_generate_header_with_bottom_border_draws_rule_below_text() {
    use crate::ir::{BorderSide, CellBorder, HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Manual v0.6".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: Some(CellBorder {
                    top: None,
                    bottom: Some(BorderSide {
                        width: 0.5,
                        color: Color::new(0xCC, 0xCC, 0xCC),
                        style: BorderLineStyle::Solid,
                    }),
                    left: None,
                    right: None,
                }),
                border_space: None,
                frame: None,
            }],
        }),
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert_eq!(
        output.source.matches("line(length: 100%").count(),
        1,
        "the bottom rule must be emitted"
    );
    assert!(
        output.source.contains("rgb(204, 204, 204)"),
        "the rule keeps the pBdr color"
    );
    let text_pos = output
        .source
        .find("Manual v0.6")
        .expect("header text present");
    let rule_pos = output
        .source
        .find("line(length: 100%")
        .expect("rule present");
    assert!(
        rule_pos > text_pos,
        "a bottom rule must be drawn after the header text"
    );
}

/// A paragraph carrying both a top and a bottom rule must draw both.
#[test]
fn test_generate_header_with_top_and_bottom_borders_draws_both_rules() {
    use crate::ir::{BorderSide, CellBorder, HFInline, HeaderFooter, HeaderFooterParagraph};

    let rule = |width: f64| {
        Some(BorderSide {
            width,
            color: Color::new(0x33, 0x66, 0x99),
            style: BorderLineStyle::Solid,
        })
    };

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Framed".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: Some(CellBorder {
                    top: rule(1.0),
                    bottom: rule(1.0),
                    left: None,
                    right: None,
                }),
                border_space: None,
                frame: None,
            }],
        }),
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert_eq!(
        output.source.matches("line(length: 100%").count(),
        2,
        "both rules must be emitted"
    );
}

/// Word measures `w:pgMar/@w:footer` from the bottom page edge to the bottom of
/// the footer, so the footer must be pinned to that line and grow upward — not
/// pushed further away from the edge.
#[test]
fn test_flow_page_footer_is_pinned_to_the_word_edge_distance() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins {
            top: 62.35,
            bottom: 62.35,
            left: 70.85,
            right: 70.85,
        },
        content: vec![make_paragraph("Body")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: Some(35.4),
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "- 1 -".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("footer-descent: 0pt"),
        "the footer origin must sit on the bottom margin line"
    );
    assert!(
        output
            .source
            .contains("block(width: 100%, height: 26.95pt)"),
        "the band must span bottom margin minus footer distance, got: {}",
        output.source
    );
    assert!(
        output.source.contains("bottom-edge: \"descender\""),
        "Word measures to the descender line"
    );
    assert!(
        output.source.contains("place(bottom"),
        "the footer grows upward from the pinned bottom"
    );
    assert!(
        !output.source.contains("move(dy: -26.95pt)"),
        "the footer must not be shifted away from the page edge"
    );
}

/// Without a declared footer distance the previous placement is kept, so
/// formats that do not carry the attribute are unaffected.
#[test]
fn test_flow_page_footer_without_edge_distance_keeps_default_placement() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Plain footer".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("footer: ["));
    assert!(!output.source.contains("footer-descent"));
}

/// A footer distance at or beyond the bottom margin leaves no band to draw, so
/// the default placement is used instead of a zero or negative height block.
#[test]
fn test_flow_page_footer_distance_beyond_margin_falls_back() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins {
            top: 72.0,
            bottom: 30.0,
            left: 72.0,
            right: 72.0,
        },
        content: vec![make_paragraph("Body")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: Some(48.0),
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Deep footer".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(!output.source.contains("footer-descent"));
    assert!(output.source.contains("Deep footer"));
}

/// Word keeps `w:spacing w:before` on the very first body paragraph, while
/// Typst drops leading block spacing at a page boundary. The gap has to be
/// emitted as explicit vertical space so the first heading is not pulled up to
/// the top margin.
#[test]
fn test_first_document_paragraph_keeps_its_space_before() {
    let mut heading = make_paragraph("Research Report");
    if let Block::Paragraph(ref mut paragraph) = heading {
        paragraph.style.space_before = Some(14.0);
        paragraph.style.space_after = Some(7.0);
    }

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![heading, make_paragraph("Body")],
        header: None,
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    let spacer = output
        .source
        .find("#v(14pt")
        .expect("explicit leading space emitted");
    let heading_pos = output
        .source
        .find("Research Report")
        .expect("heading present");
    assert!(
        spacer < heading_pos,
        "the space must precede the heading, got: {}",
        output.source
    );
    assert!(
        !output.source[spacer..heading_pos].contains("above: 14pt"),
        "the collapsed block spacing must not be emitted twice"
    );
}

/// Only the document's first paragraph gets the explicit gap. Word suppresses
/// space-before at the top of a page reached by a break, so later paragraphs
/// keep ordinary collapsing block spacing.
#[test]
fn test_later_paragraph_space_before_stays_block_spacing() {
    let mut second = make_paragraph("Second");
    if let Block::Paragraph(ref mut paragraph) = second {
        paragraph.style.space_before = Some(21.0);
    }

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("First"), second],
        header: None,
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        !output.source.contains("#v(21pt"),
        "later paragraphs keep block spacing"
    );
    assert!(output.source.contains("above: 21pt"));
}

/// `w:pBdr` sides declare a `w:space` gap in points between the text and the
/// rule; a header rule must sit that far below its text.
#[test]
fn test_generate_header_border_uses_declared_pbdr_space() {
    use crate::ir::{
        BorderSide, CellBorder, HFInline, HeaderFooter, HeaderFooterParagraph, Insets,
    };

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Manual v0.6".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: Some(CellBorder {
                    top: None,
                    bottom: Some(BorderSide {
                        width: 0.5,
                        color: Color::new(0xCC, 0xCC, 0xCC),
                        style: BorderLineStyle::Solid,
                    }),
                    left: None,
                    right: None,
                }),
                border_space: Some(Insets {
                    top: 0.0,
                    right: 0.0,
                    bottom: 4.0,
                    left: 0.0,
                }),
                frame: None,
            }],
        }),
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("block(height: 4pt)[]"),
        "the declared 4pt gap must separate text and rule, got: {}",
        output.source
    );
    assert!(
        output.source.contains("bottom-edge: \"descender\""),
        "Word measures the gap from the descender line"
    );
}

/// Without `w:space` the rule keeps the previous hairline clearance.
#[test]
fn test_generate_header_border_without_space_keeps_hairline_gap() {
    use crate::ir::{BorderSide, CellBorder, HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Plain".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: Some(CellBorder {
                    top: None,
                    bottom: Some(BorderSide {
                        width: 0.5,
                        color: Color::black(),
                        style: BorderLineStyle::Solid,
                    }),
                    left: None,
                    right: None,
                }),
                border_space: None,
                frame: None,
            }],
        }),
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("block(height: 0.5pt)[]"));
}

/// Word measures `w:pgMar/@w:header` from the top page edge to the top of the
/// header, which then grows downward. Typst anchors headers by their bottom, so
/// the band has to hold the content against the header top.
#[test]
fn test_flow_page_header_is_pinned_to_the_word_edge_distance() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins {
            top: 62.35,
            bottom: 62.35,
            left: 70.85,
            right: 70.85,
        },
        content: vec![make_paragraph("Body")],
        header: Some(HeaderFooter {
            distance_from_edge: Some(35.4),
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Manual v0.6".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("header-ascent: 0pt"),
        "the header origin must sit on the top margin line"
    );
    assert!(
        output
            .source
            .contains("block(width: 100%, height: 26.95pt)"),
        "the band must span top margin minus header distance, got: {}",
        output.source
    );
    assert!(
        output.source.contains("place(top"),
        "the header grows downward from the pinned top"
    );
}

/// Without a declared header distance the previous placement is kept.
#[test]
fn test_flow_page_header_without_edge_distance_keeps_default_placement() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::Run(Run {
                    text: "Plain header".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                })],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains("header: ["));
    assert!(!output.source.contains("header-ascent"));
}

/// Word applies the containing run's properties to a `PAGE` field result, so
/// the number must render in the run's font, size, and color rather than the
/// document default.
#[test]
fn test_page_number_field_uses_its_run_style() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let field_style = TextStyle {
        font_size: Some(8.0),
        color: Some(Color::new(0x88, 0x88, 0x88)),
        ..TextStyle::default()
    };
    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![
                    HFInline::Run(Run {
                        text: "- ".to_string(),
                        style: field_style.clone(),
                        href: None,
                        footnote: None,
                    }),
                    HFInline::PageNumber(field_style.clone()),
                ],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    let counter = output
        .source
        .find(r#"#counter(page).display("1")"#)
        .expect("page counter emitted");
    let prefix = &output.source[..counter];
    let wrapper = prefix
        .rfind("#text(")
        .expect("the counter is wrapped in its run's text properties");
    assert!(
        prefix[wrapper..].contains("size: 8pt"),
        "the field keeps the run's size, got: {}",
        &prefix[wrapper..]
    );
    assert!(
        prefix[wrapper..].contains("rgb(136, 136, 136)"),
        "the field keeps the run's color"
    );
}

/// An unstyled field stays a bare counter, so documents that never style their
/// page numbers are unchanged.
#[test]
fn test_unstyled_page_number_field_stays_bare() {
    use crate::ir::{HFInline, HeaderFooter, HeaderFooterParagraph};

    let doc = make_doc(vec![Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![make_paragraph("Body")],
        header: None,
        footer: Some(HeaderFooter {
            distance_from_edge: None,
            paragraphs: vec![HeaderFooterParagraph {
                style: ParagraphStyle::default(),
                elements: vec![HFInline::PageNumber(TextStyle::default())],
                border: None,
                border_space: None,
                frame: None,
            }],
        }),
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: None,
    })]);

    let output = generate_typst(&doc).unwrap();
    assert!(output.source.contains(r#"#counter(page).display("1")"#));
    assert!(
        !output.source.contains("#text()[#counter"),
        "no empty text wrapper"
    );
}

#[test]
fn test_section_page_numbering_updates_the_counter_and_its_numerals() {
    // Word restarts the counter at the section boundary and renders the
    // numerals w:fmt names; Typst counts from the document start in decimal
    // unless told otherwise (issue #582).
    let Page::Flow(mut flow) = make_flow_page(vec![make_paragraph("front matter")]) else {
        unreachable!()
    };
    flow.footer = Some(crate::ir::HeaderFooter {
        paragraphs: vec![crate::ir::HeaderFooterParagraph {
            style: ParagraphStyle::default(),
            elements: vec![HFInline::PageNumber(TextStyle::default())],
            border: None,
            border_space: None,
            frame: None,
        }],
        distance_from_edge: None,
    });
    flow.page_numbering = Some(crate::ir::PageNumbering {
        start: Some(1),
        format: crate::ir::PageNumberFormat::LowerRoman,
    });

    let output = generate_typst(&make_doc(vec![Page::Flow(flow)])).unwrap();

    assert!(
        output.source.contains("#counter(page).update(1)"),
        "the section restarts the counter: {}",
        output.source
    );
    assert!(
        output.source.contains(r#"#counter(page).display("i")"#),
        "the PAGE field renders the section's numerals: {}",
        output.source
    );
}

#[test]
fn test_contents_block_emits_an_outline_at_its_declared_depth() {
    // The entries, their page numbers, and the leaders between them all come
    // from where the headings land, which only the layout knows (issue #576).
    let doc = make_doc(vec![make_flow_page(vec![
        Block::TableOfContents(crate::ir::TableOfContents::Headings { depth: 3 }),
        Block::Paragraph(Paragraph {
            style: ParagraphStyle {
                heading_level: Some(1),
                ..ParagraphStyle::default()
            },
            runs: vec![Run {
                text: "1. 개요".to_string(),
                style: TextStyle::default(),
                href: None,
                footnote: None,
            }],
        }),
    ])]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("#outline(title: none, depth: 3)"),
        "the contents block resolves against the document's headings: {}",
        output.source
    );
}

#[test]
fn test_caption_list_queries_the_captions_it_collects() {
    // A caption is not a heading, so Typst's outline cannot reach it. Each one
    // drops an invisible marker as it is laid out and the list queries those,
    // so both the entries and their page numbers come from the layout
    // (issue #576).
    let doc = make_doc(vec![make_flow_page(vec![
        Block::TableOfContents(crate::ir::TableOfContents::Captions {
            identifier: "Figure".to_string(),
        }),
        Block::Caption(crate::ir::Caption {
            identifier: "Figure".to_string(),
            entry_text: "변환 파이프라인".to_string(),
            paragraph: Paragraph {
                style: ParagraphStyle::default(),
                runs: vec![Run {
                    text: "그림 1  변환 파이프라인".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                }],
            },
        }),
    ])]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output
            .source
            .contains("#metadata[변환 파이프라인]<o2p-seq-Figure>"),
        "the caption carries the marker its list queries: {}",
        output.source
    );
    assert!(
        output.source.contains("query(<o2p-seq-Figure>)")
            && output.source.contains("let target = entry.location()")
            && output.source.contains("counter(page).at(target)"),
        "the list resolves each entry's page from where it landed: {}",
        output.source
    );
    // The caption's own runs still render; the auto-space marker sits between
    // its number and the Korean that follows, which is why this looks for the
    // two halves rather than the joined string.
    assert!(
        output.source.contains("그림 1") && output.source.matches("변환 파이프라인").count() >= 2,
        "the caption still renders as itself, beside its list entry: {}",
        output.source
    );
}

#[test]
fn test_a_caption_identifier_outside_ascii_still_labels() {
    // A `SEQ` name may be written in any script; a Typst label may not.
    let doc = make_doc(vec![make_flow_page(vec![Block::TableOfContents(
        crate::ir::TableOfContents::Captions {
            identifier: "표".to_string(),
        },
    )])]);

    let output = generate_typst(&doc).unwrap();
    assert!(
        output.source.contains("query(<o2p-seq--d45c>)"),
        "the identifier reduces to label characters: {}",
        output.source
    );
}

/// Word numbers a contents entry the way the section it points into numbers
/// its pages, so an entry landing in roman-numbered front matter reads `i`
/// rather than `1` (issue #605). The format travels with the layout, because
/// only the layout knows which section an entry resolved into.
#[test]
fn test_contents_entries_number_in_the_target_sections_format() {
    use crate::ir::{PageNumberFormat, PageNumbering};

    let front_matter = Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![
            Block::TableOfContents(crate::ir::TableOfContents::Headings { depth: 3 }),
            Block::Paragraph(Paragraph {
                style: ParagraphStyle {
                    heading_level: Some(1),
                    ..ParagraphStyle::default()
                },
                runs: vec![Run {
                    text: "적용 범위".to_string(),
                    style: TextStyle::default(),
                    href: None,
                    footnote: None,
                }],
            }),
        ],
        header: None,
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: Some(PageNumbering {
            start: Some(1),
            format: PageNumberFormat::LowerRoman,
        }),
    });
    let body = Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![Block::Paragraph(Paragraph {
            style: ParagraphStyle {
                heading_level: Some(1),
                ..ParagraphStyle::default()
            },
            runs: vec![Run {
                text: "1. 개요".to_string(),
                style: TextStyle::default(),
                href: None,
                footnote: None,
            }],
        })],
        header: None,
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: Some(PageNumbering {
            start: Some(1),
            format: PageNumberFormat::Decimal,
        }),
    });

    let output = generate_typst(&make_doc(vec![front_matter, body])).unwrap();

    assert!(
        output.source.contains("state(\"o2p-page-format\""),
        "the section's numeral format is recorded where the layout can read it back: {}",
        output.source
    );
    assert!(
        output.source.contains("o2p-page-format.update(\"i\")"),
        "the roman front matter records its format: {}",
        output.source
    );
    assert!(
        output.source.contains("o2p-page-format.update(\"1\")"),
        "the decimal body records its own: {}",
        output.source
    );
    assert!(
        output.source.contains("show outline.entry"),
        "the outline renders each entry's number through the recorded format: {}",
        output.source
    );
}

/// A caption list is numbered the same way a heading outline is: an entry
/// pointing at a table in roman-numbered front matter reads `i`, not `1`
/// (issue #605). The list builds its own rows, so it has to read the format
/// back at the entry's location just as the outline rule does.
#[test]
fn test_caption_list_numbers_in_the_target_sections_format() {
    use crate::ir::{Caption, PageNumberFormat, PageNumbering};

    let page = Page::Flow(FlowPage {
        size: PageSize::default(),
        margins: Margins::default(),
        content: vec![
            Block::TableOfContents(crate::ir::TableOfContents::Captions {
                identifier: "표".to_string(),
            }),
            Block::Caption(Caption {
                identifier: "표".to_string(),
                entry_text: "문서 서지 정보".to_string(),
                paragraph: Paragraph {
                    style: ParagraphStyle::default(),
                    runs: vec![Run {
                        text: "표 1 문서 서지 정보".to_string(),
                        style: TextStyle::default(),
                        href: None,
                        footnote: None,
                    }],
                },
            }),
        ],
        header: None,
        footer: None,
        columns: None,
        line_grid_pitch: None,
        line_grid_snaps_lines: false,
        page_numbering: Some(PageNumbering {
            start: Some(1),
            format: PageNumberFormat::LowerRoman,
        }),
    });

    let output = generate_typst(&make_doc(vec![page])).unwrap();

    assert!(
        output.source.contains("o2p-page-format.at(target)"),
        "the caption list reads the format back at each entry's location: {}",
        output.source
    );
    assert!(
        !output.source.contains("#entry_page]"),
        "the raw page count no longer reaches the row: {}",
        output.source
    );
}

/// Excel's `<row ht="…"/>` is the row's *total* printed height, and the text
/// sits inside it. office2pdf emitted the height as a Typst row track but let
/// the cell keep the default 0.65em leading on top of the face's own box, so
/// every line asked for ~1.83em and pushed the track open — rows printed about
/// 45% tall (issue #608). A sheet row that declares its height pins the line to
/// it instead.
#[test]
fn test_sheet_row_height_pins_the_cell_line_box() {
    use crate::ir::{Insets, Table, TableCell, TableRow};

    let cell = TableCell {
        content: vec![Block::Paragraph(Paragraph {
            style: ParagraphStyle::default(),
            runs: vec![Run {
                text: "office2pdf".to_string(),
                style: TextStyle {
                    font_size: Some(10.0),
                    font_family: Some("Calibri".to_string()),
                    ..TextStyle::default()
                },
                href: None,
                footnote: None,
            }],
        })],
        col_span: 1,
        row_span: 1,
        border: None,
        background: None,
        data_bar: None,
        icon_text: None,
        icon_color: None,
        spill_width: None,
        vertical_align: None,
        padding: None,
    };
    let page = Page::Sheet(SheetPage {
        name: "Sheet1".to_string(),
        size: PageSize::default(),
        margins: Margins::default(),
        table: Table {
            rows: vec![TableRow {
                cells: vec![cell],
                height: Some(15.0),
            }],
            column_widths: vec![120.0],
            header_row_count: 0,
            non_repeating_header_row_count: 0,
            alignment: None,
            default_cell_padding: Some(Insets {
                top: 1.0,
                right: 2.0,
                bottom: 1.5,
                left: 2.0,
            }),
            use_content_driven_row_heights: false,
            default_vertical_align: None,
        },
        header: None,
        footer: None,
        charts: vec![],
        images: Vec::new(),
        text_boxes: Vec::new(),
    });

    let output = generate_typst(&make_doc(vec![page])).unwrap();

    assert!(
        output.source.contains("#set par(leading: 0pt)"),
        "the sheet cell drops the default leading that pushed the track open: {}",
        output.source
    );
    // 15pt row less 1.0 + 1.5 of padding leaves 12.5pt for a 10pt line: the
    // emitted edges have to sum to 1.25em, not the ~1.83em the default model
    // asked for.
    let edges: Vec<f64> = regex_edges(&output.source);
    assert_eq!(
        edges.len(),
        2,
        "one top-edge/bottom-edge pair is emitted: {}",
        output.source
    );
    let total: f64 = edges[0] + edges[1];
    assert!(
        (total - 1.25).abs() < 0.01,
        "line box spans the declared row less its padding, got {total}em: {}",
        output.source
    );
}

/// Pull the `top-edge`/`bottom-edge` em values out of generated Typst.
fn regex_edges(source: &str) -> Vec<f64> {
    let mut values: Vec<f64> = Vec::new();
    for (marker, sign) in [("top-edge: ", 1.0), ("bottom-edge: -", 1.0)] {
        if let Some(start) = source.find(marker) {
            let rest = &source[start + marker.len()..];
            let end = rest.find("em").unwrap_or(0);
            if let Ok(value) = rest[..end].parse::<f64>() {
                values.push(value * sign);
            }
        }
    }
    values
}
