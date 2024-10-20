use crate::logtype::LogType;
use crate::QQWing;
/**
 * While solving the puzzle, log steps taken in a log item. This is useful for
 * later printing out the solve history or gathering statistics about how hard
 * the puzzle was to solve.
 */
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
impl LogItem {
    // pub LogItem(int r, LogType t) {
    // 	init(r, t, 0, -1);
    // }

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

    // /**
    //  * Get the column (1 indexed), or -1 if no column
    //  */
    // pub int getColumn() {
    // 	if (position <= -1) return -1;
    // 	return QQWing.cellToColumn(position) + 1;
    // }

    // /**
    //  * Get the position (0-80) on the board or -1 if no position
    //  */
    // pub int getPosition() {
    // 	return position;
    // }

    /**
     * Get the value, or -1 if no value
     */
    pub fn get_value(&self) -> usize {
        self.value
    }

    // /**
    //  * Print the current log item. The message used is determined by the type of
    //  * log item.
    //  */
    // pub String getDescription() {
    // 	StringBuilder sb = new StringBuilder();
    // 	sb.append("Round: ").append(getRound());
    // 	sb.append(" - ");
    // 	sb.append(getType().getDescription());
    // 	if (value > 0 || position > -1) {
    // 		sb.append(" (");
    // 		if (position > -1) {
    // 			sb.append("Row: ").append(getRow()).append(" - Column: ").append(getColumn());
    // 		}
    // 		if (value > 0) {
    // 			if (position > -1) sb.append(" - ");
    // 			sb.append("Value: ").append(getValue());
    // 		}
    // 		sb.append(")");
    // 	}
    // 	return sb.toString();
    // }

    // pub String toString() {
    // 	return getDescription();
    // }
}
