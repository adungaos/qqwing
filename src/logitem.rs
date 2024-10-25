use std::fmt::Display;

use crate::logtype::LogType;
use crate::QQWing;
///
/// While solving the puzzle, log steps taken in a log item. This is useful for
/// later printing out the solve history or gathering statistics about how hard
/// the puzzle was to solve.
/// 
#[derive(Debug, Clone)]
pub struct LogItem {
    /**
     * The recursion level at which this item was gathered. Used for backing out
     * log items solve branches that don't lead to a solution.
     */
    round: u8,

    /**
     * The type of log message that will determine the message printed.
     */
    pub log_type: LogType,

    /**
     * Value that was set by the operation (or zero for no value)
     */
    value: usize,

    /**
     * position on the board at which the value (if any) was set.
     */
    position: usize,
}

impl Display for LogItem {
    /**
     * Print the current log item. The message used is determined by the type of
     * log item.
     */
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Round: {} - {:?} (Row: {} - Column: {} - Value: {})",
            self.round,
            self.log_type,
            self.get_row(),
            self.get_column(),
            self.value
        )
    }
}

impl LogItem {
    pub fn new(r: u8, t: LogType, v: usize, p: usize) -> Self {
        LogItem::init(r, t, v, p)
    }

    pub fn init(r: u8, t: LogType, v: usize, p: usize) -> Self {
        Self {
            round: r,
            log_type: t,
            value: v,
            position: p,
        }
    }

    pub fn get_round(&self) -> u8 {
        self.round
    }

    /**
     * Get the row (1 indexed), or -1 if no row
     */
    pub fn get_row(&self) -> u8 {
        if self.position == 255 {
            return 255;
        }
        QQWing::cell_to_row(self.position) as u8 + 1
    }

    /**
     * Get the column (1 indexed), or -1 if no column
     */
    pub fn get_column(&self) -> u8 {
        if self.position == 255 {
            return 255;
        }
        QQWing::cell_to_column(self.position) as u8 + 1
    }

    /**
     * Get the value, or -1 if no value
     */
    pub fn get_value(&self) -> usize {
        self.value
    }
}
