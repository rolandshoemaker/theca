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

use errors::Result;
use {Item, Status};
use utils::termsize;

#[derive(Clone, Copy)]
pub struct LineFormat {
    pub colsep: usize,
    pub id_width: usize,
    pub title_width: usize,
    pub status_width: usize,
    pub touched_width: usize,
}

impl LineFormat {
    pub fn new(items: &[Item],
               condensed: bool,
               search: bool)
               -> Result<LineFormat> {
        // get termsize :>
        let console_width = termsize();

        // set colsep
        let colsep = if condensed {
            1
        } else {
            2
        };

        let mut line_format = LineFormat {
            colsep: colsep,
            id_width: 0,
            title_width: 0,
            status_width: 0,
            touched_width: 0,
        };

        // get length of longest id string
        line_format.id_width = match items.iter()
                                          .max_by_key(|n| n.id.to_string().len()) {
            Some(w) => w.id.to_string().len(),
            None => 0,
        };
        // if longest id is 1 char and we are using extended printing
        // then set id_width to 2 so "id" isn't truncated
        if line_format.id_width < 2 && !condensed {
            line_format.id_width = 2;
        }

        // get length of longest title string
        line_format.title_width = match items.iter()
                                             .max_by_key(|n| if !n.body.is_empty() {
                                                 n.title.len() + 4
                                                 } else {
                                                 n.title.len()
                                             }) {
            Some(n) => {
                if n.body.is_empty() || search {
                    n.title.len()
                } else {
                    n.title.len() + 4
                }
            }
            None => 0,
        };
        // if using extended and longest title is less than 5 chars
        // set title_width to 5 so "title" won't be truncated
        if line_format.title_width < 5 && !condensed {
            line_format.title_width = 5;
        }

        // status length stuff
        line_format.status_width = if items.iter()
                                           .any(|n| n.status != Status::Blank) {
            if condensed {
                // only display first char of status (e.g. S or U) for condensed print
                1
            } else {
                // expanded print, get longest status (7 or 6 / started or urgent)
                match items.iter().max_by_key(|n| format!("{}", n.status).len()) {
                    Some(w) => format!("{}", w.status).len(),
                    None => 0,
                }
            }
        } else {
            // no items have statuses so truncate column
            0
        };

        // last_touched has fixed string length so no need for silly iter stuff
        line_format.touched_width = if condensed {
            10 // condensed
        } else {
            19 // expanded
        };

        // check to make sure our new line format isn't bigger than the console
        let line_width = line_format.line_width();
        if console_width > 0 && line_width > console_width &&
           (line_format.title_width - (line_width - console_width)) > 0 {
            // if it is trim text from the title width since it is always the biggest...
            // if there isn't any statuses, also give the title the colsep char space
            line_format.title_width -= line_width - console_width;
        }

        Ok(line_format)
    }

    pub fn line_width(&self) -> usize {
        let columns = if self.status_width == 0 {
            2 * self.colsep
        } else {
            3 * self.colsep
        };
        self.id_width + self.title_width + self.status_width + self.touched_width + columns
    }
}
