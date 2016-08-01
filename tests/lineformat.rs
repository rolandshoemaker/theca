extern crate theca;

use theca::{Status, ThecaItem};
use theca::lineformat::{LineFormat};

struct LineTest {
    input_notes: Vec<ThecaItem>,
    condensed: bool,
    search: bool,
    expected_format: LineFormat
}

fn test_formatter(tests: &[LineTest]) {
    for t in tests.iter() {
        let wrapped_format = LineFormat::new(&t.input_notes, t.condensed, t.search);
        assert!(wrapped_format.is_ok());
        let actual_format = wrapped_format.ok().unwrap();
        assert_eq!(t.expected_format.colsep, actual_format.colsep);
        assert_eq!(t.expected_format.id_width, actual_format.id_width);
        assert_eq!(t.expected_format.title_width, actual_format.title_width);
        assert_eq!(t.expected_format.status_width, actual_format.status_width);
        assert_eq!(t.expected_format.touched_width, actual_format.touched_width);
    }
}

#[test]
fn test_new_line_format_basic() {
    let basic_tests = vec![
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: false,
            search: false,
            expected_format: LineFormat {
                colsep: 2,
                id_width: 2,
                title_width: 14,
                status_width: 0,
                touched_width: 19
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: true,
            search: false,
            expected_format: LineFormat {
                colsep: 1,
                id_width: 1,
                title_width: 14,
                status_width: 0,
                touched_width: 10
            }
        }
    ];

    test_formatter(&basic_tests[..]);
}

#[test]
fn test_new_line_format_statuses() {
    let status_tests = vec![
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::Started,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: false,
            search: false,
            expected_format: LineFormat {
                colsep: 2,
                id_width: 2,
                title_width: 14,
                status_width: 7,
                touched_width: 19
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "".to_string(),
                    status: Status::Urgent,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: false,
            search: false,
            expected_format: LineFormat {
                colsep: 2,
                id_width: 2,
                title_width: 14,
                status_width: 6,
                touched_width: 19
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "".to_string(),
                    status: Status::Urgent,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: true,
            search: false,
            expected_format: LineFormat {
                colsep: 1,
                id_width: 1,
                title_width: 14,
                status_width: 1,
                touched_width: 10
            }
        }
    ];

    test_formatter(&status_tests[..]);
}

#[test]
fn test_new_line_format_body() {
    let body_tests = vec![
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: false,
            search: false,
            expected_format: LineFormat {
                colsep: 2,
                id_width: 2,
                title_width: 18,
                status_width: 0,
                touched_width: 19
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: true,
            search: false,
            expected_format: LineFormat {
                colsep: 1,
                id_width: 1,
                title_width: 18,
                status_width: 0,
                touched_width: 10
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: false,
            search: true,
            expected_format: LineFormat {
                colsep: 2,
                id_width: 2,
                title_width: 14,
                status_width: 0,
                touched_width: 19
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: true,
            search: true,
            expected_format: LineFormat {
                colsep: 1,
                id_width: 1,
                title_width: 14,
                status_width: 0,
                touched_width: 10
            }
        }
    ];

    test_formatter(&body_tests[..]);
}

#[test]
fn test_new_line_format_full() {
    let body_tests = vec![
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::Started,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: false,
            search: false,
            expected_format: LineFormat {
                colsep: 2,
                id_width: 2,
                title_width: 18,
                status_width: 7,
                touched_width: 19
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::Started,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: true,
            search: false,
            expected_format: LineFormat {
                colsep: 1,
                id_width: 1,
                title_width: 18,
                status_width: 1,
                touched_width: 10
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::Urgent,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: false,
            search: true,
            expected_format: LineFormat {
                colsep: 2,
                id_width: 2,
                title_width: 14,
                status_width: 6,
                touched_width: 19
            }
        },
        LineTest {
            input_notes: vec![
                ThecaItem {
                    id: 1,
                    title: "a title".to_string(),
                    body: "".to_string(),
                    status: Status::NoStatus,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                },
                ThecaItem {
                    id: 2,
                    title: "a longer title".to_string(),
                    body: "this is a body".to_string(),
                    status: Status::Urgent,
                    last_touched: "2015-01-22 19:43:24 -0800".to_string()
                }
            ],
            condensed: true,
            search: true,
            expected_format: LineFormat {
                colsep: 1,
                id_width: 1,
                title_width: 14,
                status_width: 1,
                touched_width: 10
            }
        }
    ];

    test_formatter(&body_tests[..]);
}
