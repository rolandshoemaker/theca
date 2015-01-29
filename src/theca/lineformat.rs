//  _   _                    
// | |_| |__   ___  ___ __ _ 
// | __| '_ \ / _ \/ __/ _` |
// | |_| | | |  __/ (_| (_| |
//  \__|_| |_|\___|\___\__,_|
//
// licensed under the MIT license <http://opensource.org/licenses/MIT>
//
// lineformat.rs
//   definition of LineFormat, it looks at all the notes provided and
//   tries to construct a line format that won't overflow the console
//   width.

use errors::{ThecaError};
use ::{ThecaItem};
use utils::{termsize};

#[derive(Copy)]
pub struct LineFormat {
    pub colsep: usize,
    pub id_width: usize,
    pub title_width: usize,
    pub status_width: usize,
    pub touched_width: usize
}

impl LineFormat {
    pub fn new(items: &Vec<ThecaItem>, condensed: bool, search: bool) -> Result<LineFormat, ThecaError> {
        // get termsize :>
        let console_width = termsize();

        // set colsep
        let colsep = match condensed {
            true => 1,
            false => 2
        };

        let mut line_format = LineFormat {colsep: colsep, id_width:0, title_width:0,
                                          status_width:0, touched_width:0};

        // get length of longest id string
        line_format.id_width = match items.iter().max_by(|n| n.id.to_string().len()) {
            Some(w) => w.id.to_string().len(),
            None => 0
        };
        // if longest id is 1 char and we are using extended printing
        // then set id_width to 2 so "id" isn't truncated
        if line_format.id_width < 2 && !condensed {line_format.id_width = 2;}

        // get length of longest title string
        line_format.title_width = match items.iter().max_by(|n| match n.body.len() > 0 {
            true => n.title.len()+4,
            false => n.title.len()
        }) {
            Some(n) => match n.body.is_empty() || search {
                true => n.title.len(),
                false => n.title.len()+4
            },
            None => 0
        };
        // if using extended and longest title is less than 5 chars
        // set title_width to 5 so "title" won't be truncated
        if line_format.title_width < 5 && !condensed {line_format.title_width = 5;}

        // status length stuff
        line_format.status_width = match items.iter().any(|n| n.status.len() > 0) {
            true => {
                match condensed {
                    // expanded print, get longest status (7 or 6 / started or urgent)
                    false => {
                        match items.iter().max_by(|n| n.status.len()) {
                            Some(w) => w.status.len(),
                            None => {
                                0
                            }
                        }
                    },
                    // only display first char of status (e.g. S or U) for condensed print
                    true => 1
                }
            },
            // no items have statuses so truncate column
            false => {
                0
            }
        };

        // last_touched has fixed string length so no need for silly iter stuff
        line_format.touched_width = match condensed {
            true => 10, // condensed
            false => 19 // expanded
        };

        // check to make sure our new line format isn't bigger than the console
        let line_width = line_format.line_width();
        if console_width > 0 && line_width > console_width &&
           (line_format.title_width-(line_width-console_width)) > 0 {
            // if it is trim text from the title width since it is always the biggest...
            // if there isn't any statuses, also give the title the colsep char space
            line_format.title_width -= line_width-console_width;
        }

        Ok(line_format)
    }

    pub fn line_width(&self) -> usize {
        let columns = match self.status_width == 0 {
            true => 2*self.colsep,
            false => 3*self.colsep
        };
        self.id_width+self.title_width+self.status_width+self.touched_width+columns
    }
}

#[cfg(test)]
mod tests {
    use ::{ThecaItem};
    use super::{LineFormat};

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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "".to_string(),
                        status: "".to_string(),
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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "".to_string(),
                        status: "".to_string(),
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

        test_formatter(&basic_tests[]);
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
                        status: "Started".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "".to_string(),
                        status: "".to_string(),
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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "".to_string(),
                        status: "Urgent".to_string(),
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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "".to_string(),
                        status: "Urgent".to_string(),
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

        test_formatter(&status_tests[]);
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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "this is a body".to_string(),
                        status: "".to_string(),
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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "this is a body".to_string(),
                        status: "".to_string(),
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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "this is a body".to_string(),
                        status: "".to_string(),
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
                        status: "".to_string(),
                        last_touched: "2015-01-22 19:43:24 -0800".to_string()
                    },
                    ThecaItem {
                        id: 2,
                        title: "a longer title".to_string(),
                        body: "this is a body".to_string(),
                        status: "".to_string(),
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

        test_formatter(&body_tests[]);
    }

    #[test]
    fn test_new_line_format_body_started_expanded() {
        let notes = vec![
            ThecaItem {
                id: 1,
                title: "a title".to_string(),
                body: "".to_string(),
                status: "Started".to_string(),
                last_touched: "2015-01-22 19:43:24 -0800".to_string()
            },
            ThecaItem {
                id: 2,
                title: "a longer title".to_string(),
                body: "this is a body".to_string(),
                status: "".to_string(),
                last_touched: "2015-01-22 19:43:24 -0800".to_string()
            }
        ];
        let expected_format = LineFormat {
            colsep: 2,
            id_width: 2,
            title_width: 18,
            status_width: 7,
            touched_width: 19
        };
        let wrapped_format = LineFormat::new(&notes, false, false);
        assert!(wrapped_format.is_ok());
        let actual_format = wrapped_format.ok().unwrap();
        assert_eq!(expected_format.colsep, actual_format.colsep);
        assert_eq!(expected_format.id_width, actual_format.id_width);
        assert_eq!(expected_format.title_width, actual_format.title_width);
        assert_eq!(expected_format.status_width, actual_format.status_width);
        assert_eq!(expected_format.touched_width, actual_format.touched_width);
    }

    #[test]
    fn test_new_line_format_body_urgent_expanded() {
        let notes = vec![
            ThecaItem {
                id: 1,
                title: "a title".to_string(),
                body: "".to_string(),
                status: "".to_string(),
                last_touched: "2015-01-22 19:43:24 -0800".to_string()
            },
            ThecaItem {
                id: 2,
                title: "a longer title".to_string(),
                body: "this is a body".to_string(),
                status: "Urgent".to_string(),
                last_touched: "2015-01-22 19:43:24 -0800".to_string()
            }
        ];
        let expected_format = LineFormat {
            colsep: 2,
            id_width: 2,
            title_width: 18,
            status_width: 6,
            touched_width: 19
        };
        let wrapped_format = LineFormat::new(&notes, false, false);
        assert!(wrapped_format.is_ok());
        let actual_format = wrapped_format.ok().unwrap();
        assert_eq!(expected_format.colsep, actual_format.colsep);
        assert_eq!(expected_format.id_width, actual_format.id_width);
        assert_eq!(expected_format.title_width, actual_format.title_width);
        assert_eq!(expected_format.status_width, actual_format.status_width);
        assert_eq!(expected_format.touched_width, actual_format.touched_width);
    }

    #[test]
    fn test_new_line_format_body_started_condensed() {
        let notes = vec![
            ThecaItem {
                id: 1,
                title: "a title".to_string(),
                body: "".to_string(),
                status: "Started".to_string(),
                last_touched: "2015-01-22 19:43:24 -0800".to_string()
            },
            ThecaItem {
                id: 2,
                title: "a longer title".to_string(),
                body: "this is a body".to_string(),
                status: "".to_string(),
                last_touched: "2015-01-22 19:43:24 -0800".to_string()
            }
        ];
        let expected_format = LineFormat {
            colsep: 1,
            id_width: 1,
            title_width: 18,
            status_width: 1,
            touched_width: 10
        };
        let wrapped_format = LineFormat::new(&notes, true, false);
        assert!(wrapped_format.is_ok());
        let actual_format = wrapped_format.ok().unwrap();
        assert_eq!(expected_format.colsep, actual_format.colsep);
        assert_eq!(expected_format.id_width, actual_format.id_width);
        assert_eq!(expected_format.title_width, actual_format.title_width);
        assert_eq!(expected_format.status_width, actual_format.status_width);
        assert_eq!(expected_format.touched_width, actual_format.touched_width);
    }
}
