//! qqwing - Sudoku solver and generator
//!
//! Copyright (C) 2006-2014 Stephen Ostermiller <http://ostermiller.org/>
//!
//! Copyright (C) 2007 Jacques Bensimon (jacques@ipm.com)
//!
//! Copyright (C) 2007 Joel Yarde (joel.yarde - gmail.com)
//!
//!
//! This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; either version 2 of the License, or (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License along with this program; if not, write to the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use rand::{self, random, seq::SliceRandom, thread_rng};
use std::usize;
use strum::{EnumIter, EnumString};
use thiserror::Error;
use tracing::{debug, info};

use difficulty::Difficulty;
use logitem::LogItem;
use logtype::LogType;
use symmetry::Symmetry;

/// Module for puzzle difficulty.
pub mod difficulty;
/// Module for log item.
pub mod logitem;
/// Module for log type.
pub mod logtype;
/// Module for puzzle symmetry.
pub mod symmetry;
const UNSET_VALUE: usize = 4294967295;
const NL: &str = "\n";
const GRID_SIZE: usize = 3;
const ROW_COL_SEC_SIZE: usize = GRID_SIZE * GRID_SIZE;
const SEC_GROUP_SIZE: usize = ROW_COL_SEC_SIZE * GRID_SIZE;
pub const BOARD_SIZE: usize = ROW_COL_SEC_SIZE * ROW_COL_SEC_SIZE;
const POSSIBILITY_SIZE: usize = BOARD_SIZE * ROW_COL_SEC_SIZE;

#[derive(Error, Debug)]
pub enum QQWingError {
    #[error("Marking position that already has been marked.")]
    PositionAlreadyMarked,
    #[error("Marking position that was marked another round.")]
    PositionMarkedAnotherRound,
    #[error("Marking impossible position.")]
    PositionImpossible,
}

/// The board containing all the memory structures and methods for solving or
/// generating sudoku puzzles.
#[derive(Debug)]
pub struct QQWing {
    /**
     * The last round of solving
     */
    last_solve_round: u8,

    /**
     * The 81 integers that make up a sudoku puzzle. Givens are 1-9, unknowns
     * are 0. Once initialized, this puzzle remains as is. The answer is worked
     * out in "solution".
     */
    puzzle: [u8; BOARD_SIZE],

    /**
     * The 81 integers that make up a sudoku puzzle. The solution is built here,
     * after completion all will be 1-9.
     */
    solution: [u8; BOARD_SIZE],

    /**
     * Recursion depth at which each of the numbers in the solution were placed.
     * Useful for backing out solve branches that don't lead to a solution.
     */
    solution_round: [u8; BOARD_SIZE],

    /**
     * The 729 integers that make up a the possible values for a Sudoku puzzle.
     * (9 possibilities for each of 81 squares). If possibilities[i] is zero,
     * then the possibility could still be filled in according to the Sudoku
     * rules. When a possibility is eliminated, possibilities[i] is assigned the
     * round (recursion level) at which it was determined that it could not be a
     * possibility.
     */
    possibilities: [u8; POSSIBILITY_SIZE],

    /**
     * An array the size of the board (81) containing each of the numbers 0-n
     * exactly once. This array may be shuffled so that operations that need to
     * look at each cell can do so in a random order.
     */
    random_board_array: [u8; BOARD_SIZE],

    /**
     * An array with one element for each position (9), in some random order to
     * be used when trying each position in turn during guesses.
     */
    random_possibility_array: [u8; ROW_COL_SEC_SIZE],

    /**
     * Whether or not to record history
     */
    record_history: bool,

    /**
     * Whether or not to print history as it happens
     */
    log_history: bool,

    /**
     * A list of moves used to solve the puzzle. This list contains all moves,
     * even on solve branches that did not lead to a solution.
     */
    solve_history: Vec<LogItem>,

    /**
     * A list of moves used to solve the puzzle. This list contains only the
     * moves needed to solve the puzzle, but doesn't contain information about
     * bad guesses.
     */
    solve_instructions: Vec<LogItem>,

    /**
     * The style with which to print puzzles and solutions
     */
    pub print_style: PrintStyle,
}

impl QQWing {
    pub fn new() -> Self {
        Self {
            last_solve_round: 0,
            puzzle: [0; BOARD_SIZE],
            solution: [0; BOARD_SIZE],
            solution_round: [0; BOARD_SIZE],
            possibilities: [0; POSSIBILITY_SIZE],
            random_possibility_array: core::array::from_fn::<u8, ROW_COL_SEC_SIZE, _>(|i| i as u8),
            random_board_array: core::array::from_fn::<u8, BOARD_SIZE, _>(|i| i as u8),
            record_history: false,
            log_history: false,
            solve_history: Vec::new(),
            solve_instructions: Vec::new(),
            print_style: PrintStyle::READABLE,
        }
    }

    /**
     * Get the number of cells that are set in the puzzle (as opposed to figured
     * out in the solution
     */
    fn get_given_count(&self) -> u32 {
        let mut count = 0;
        for i in 0..BOARD_SIZE {
            if self.puzzle[i] != 0 {
                count += 1;
            }
        }
        count
    }

    /**
     * Set the board to the given puzzle. The given puzzle must be an array of 81 integers.
     */
    pub fn set_puzzle(&mut self, init_puzzle: Vec<u8>) -> bool {
        for i in 0..BOARD_SIZE {
            self.puzzle[i] = init_puzzle[i];
        }
        self.reset()
    }

    /**
     * Reset the board to its initial state with only the givens. This method
     * clears any solution, resets statistics, and clears any history messages.
     */
    fn reset(&mut self) -> bool {
        self.solution.fill(0);
        self.solution_round.fill(0);
        self.possibilities.fill(0);
        self.solve_history.clear();
        self.solve_instructions.clear();

        let round = 1;
        for position in 0..BOARD_SIZE {
            if self.puzzle[position] > 0 {
                let val_index = self.puzzle[position] - 1;
                let val_pos = QQWing::get_possibility_index(val_index as usize, position);
                let value = self.puzzle[position];
                if self.possibilities[val_pos] != 0 {
                    return false;
                }
                let _ = self.mark(position, round, value).unwrap();
                if self.log_history || self.record_history {
                    self.add_history_item(LogItem::new(
                        round,
                        LogType::Given,
                        value as usize,
                        position,
                    ));
                }
            }
        }

        true
    }

    /**
     * Get the difficulty rating.
     *
     * This method will return Difficulty::UNKNOWN unless
     * a puzzle has been generated or set and then the following methods called:
     * set_record_history(true), and solve()
     */
    pub fn get_difficulty(&self) -> Difficulty {
        if self.get_guess_count() > 0 {
            return Difficulty::EXPERT;
        }
        if self.get_box_line_reduction_count() > 0 {
            return Difficulty::MEDIUM;
        }
        if self.get_pointing_pair_triple_count() > 0 {
            return Difficulty::MEDIUM;
        }
        if self.get_hidden_pair_count() > 0 {
            return Difficulty::MEDIUM;
        }
        if self.get_naked_pair_count() > 0 {
            return Difficulty::MEDIUM;
        }
        if self.get_hidden_single_count() > 0 {
            return Difficulty::EASY;
        }
        if self.get_single_count() > 0 {
            return Difficulty::SIMPLE;
        }
        return Difficulty::UNKNOWN;
    }

    /**
     * Get the number of cells for which the solution was determined because
     * there was only one possible value for that cell.
     */
    fn get_single_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_instructions, LogType::Single)
    }

    /**
     * Get the number of cells for which the solution was determined because
     * that cell had the only possibility for some value in the row, column, or
     * section.
     */
    fn get_hidden_single_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_instructions, LogType::HiddenSingleRow)
            + QQWing::get_log_count(&self.solve_instructions, LogType::HiddenSingleColumn)
            + QQWing::get_log_count(&self.solve_instructions, LogType::HiddenSingleSection)
    }

    /**
     * Get the number of naked pair reductions that were performed in solving
     * this puzzle.
     */
    fn get_naked_pair_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_instructions, LogType::NakedPairRow)
            + QQWing::get_log_count(&self.solve_instructions, LogType::NakedPairColumn)
            + QQWing::get_log_count(&self.solve_instructions, LogType::NakedPairSection)
    }

    /**
     * Get the number of hidden pair reductions that were performed in solving
     * this puzzle.
     */
    fn get_hidden_pair_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_instructions, LogType::HiddenPairRow)
            + QQWing::get_log_count(&self.solve_instructions, LogType::HiddenPairColumn)
            + QQWing::get_log_count(&self.solve_instructions, LogType::HiddenPairSection)
    }

    /**
     * Get the number of pointing pair/triple reductions that were performed in
     * solving this puzzle.
     */
    fn get_pointing_pair_triple_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_instructions, LogType::PointingPairTripleRow)
            + QQWing::get_log_count(&self.solve_instructions, LogType::PointingPairTripleColumn)
    }

    /**
     * Get the number of box/line reductions that were performed in solving this
     * puzzle.
     */
    fn get_box_line_reduction_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_instructions, LogType::RowBox)
            + QQWing::get_log_count(&self.solve_instructions, LogType::ColumnBox)
    }

    /**
     * Get the number lucky guesses in solving this puzzle.
     */
    fn get_guess_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_instructions, LogType::Guess)
    }

    /**
     * Get the number of backtracks (unlucky guesses) required when solving this
     * puzzle.
     */
    fn get_backtrack_count(&self) -> usize {
        QQWing::get_log_count(&self.solve_history, LogType::Rollback)
    }

    fn shuffle_random_arrays(&mut self) {
        let mut rng = thread_rng();
        self.random_board_array.shuffle(&mut rng);
        self.random_possibility_array.shuffle(&mut rng);
    }

    fn clear_puzzle(&mut self) {
        debug!("Clear any existing puzzle");
        for i in 0..BOARD_SIZE {
            self.puzzle[i] = 0;
        }
        self.reset();
    }

    /// Generate a new sudoku puzzle.
    pub fn generate_puzzle(&mut self) -> bool {
        self.generate_puzzle_symmetry(Symmetry::NONE)
    }

    fn generate_puzzle_symmetry(&mut self, symmetry: Symmetry) -> bool {
        let mut symmetry = symmetry;
        if symmetry == Symmetry::RANDOM {
            symmetry = QQWing::get_random_symmetry();
        }
        debug!("Symmetry: {:?}", symmetry);
        // Don't record history while generating.
        let rec_history = self.record_history;
        self.set_record_history(false);
        let l_history = self.record_history;
        self.set_log_history(false);

        self.clear_puzzle();

        // Start by getting the randomness in order so that
        // each puzzle will be different from the last.
        self.shuffle_random_arrays();

        // Now solve the puzzle the whole way. The solve
        // uses random algorithms, so we should have a
        // really randomly totally filled sudoku
        // Even when starting from an empty grid
        self.solve();

        if symmetry == Symmetry::NONE {
            // Rollback any square for which it is obvious that
            // the square doesn't contribute to a unique solution
            // (ie, squares that were filled by logic rather
            // than by guess)
            self.rollback_non_guesses();
        }

        // Record all marked squares as the puzzle so
        // that we can call countSolutions without losing it.
        for i in 0..BOARD_SIZE {
            self.puzzle[i] = self.solution[i];
        }

        // Rerandomize everything so that we test squares
        // in a different order than they were added.
        self.shuffle_random_arrays();

        // Remove one value at a time and see if
        // the puzzle still has only one solution.
        // If it does, leave it out the point because
        // it is not needed.
        for i in 0..BOARD_SIZE {
            // check all the positions, but in shuffled order
            let position = self.random_board_array[i] as usize;
            if self.puzzle[position] > 0 {
                let mut positionsym1 = UNSET_VALUE;
                let mut positionsym2 = UNSET_VALUE;
                let mut positionsym3 = UNSET_VALUE;
                match symmetry {
                    Symmetry::ROTATE90 => {
                        positionsym2 = QQWing::row_column_to_cell(
                            ROW_COL_SEC_SIZE - 1 - QQWing::cell_to_column(position),
                            QQWing::cell_to_row(position),
                        );
                        positionsym3 = QQWing::row_column_to_cell(
                            QQWing::cell_to_column(position),
                            ROW_COL_SEC_SIZE - 1 - QQWing::cell_to_row(position),
                        );
                    }
                    Symmetry::ROTATE180 => {
                        positionsym1 = QQWing::row_column_to_cell(
                            ROW_COL_SEC_SIZE - 1 - QQWing::cell_to_row(position),
                            ROW_COL_SEC_SIZE - 1 - QQWing::cell_to_column(position),
                        )
                    }
                    Symmetry::MIRROR => {
                        positionsym1 = QQWing::row_column_to_cell(
                            QQWing::cell_to_row(position),
                            ROW_COL_SEC_SIZE - 1 - QQWing::cell_to_column(position),
                        )
                    }
                    Symmetry::FLIP => {
                        positionsym1 = QQWing::row_column_to_cell(
                            ROW_COL_SEC_SIZE - 1 - QQWing::cell_to_row(position),
                            QQWing::cell_to_column(position),
                        )
                    }
                    _ => {}
                }
                // try backing out the value and
                // counting solutions to the puzzle
                let saved_value = self.puzzle[position];
                self.puzzle[position] = 0;
                let mut saved_sym1 = 0;
                if positionsym1 != UNSET_VALUE {
                    saved_sym1 = self.puzzle[positionsym1];
                    self.puzzle[positionsym1] = 0;
                }
                let mut saved_sym2 = 0;
                if positionsym2 != UNSET_VALUE {
                    saved_sym2 = self.puzzle[positionsym2];
                    self.puzzle[positionsym2] = 0;
                }
                let mut saved_sym3 = 0;
                if positionsym3 != UNSET_VALUE {
                    saved_sym3 = self.puzzle[positionsym3];
                    self.puzzle[positionsym3] = 0;
                }
                self.reset();
                if self.count_solutions_round(2, true) > 1 {
                    // Put it back in, it is needed
                    self.puzzle[position] = saved_value;
                    if positionsym1 != UNSET_VALUE && saved_sym1 != 0 {
                        self.puzzle[positionsym1] = saved_sym1;
                    }
                    if positionsym2 != UNSET_VALUE && saved_sym2 != 0 {
                        self.puzzle[positionsym2] = saved_sym2;
                    }
                    if positionsym3 != UNSET_VALUE && saved_sym3 != 0 {
                        self.puzzle[positionsym3] = saved_sym3;
                    }
                }
            }
        }

        // Clear all solution info, leaving just the puzzle.
        self.reset();

        // Restore recording history.
        self.set_record_history(rec_history);
        self.set_log_history(l_history);

        true
    }

    fn rollback_non_guesses(&mut self) {
        // Guesses are odd rounds
        // Non-guesses are even rounds
        for i in 2..self.last_solve_round {
            if i % 2 == 1 {
                continue;
            }
            self.rollback_round(i);
        }
    }

    pub fn set_print_style(&mut self, ps: PrintStyle) {
        self.print_style = ps;
    }

    pub fn set_record_history(&mut self, rec_history: bool) {
        self.record_history = rec_history;
    }

    pub fn set_log_history(&mut self, log_hist: bool) {
        self.log_history = log_hist;
    }

    fn add_history_item(&mut self, l: LogItem) {
        if self.log_history {
            info!("{}", l);
        }
        if self.record_history {
            self.solve_history.push(l.clone()); // ->push_back(l);
            self.solve_instructions.push(l); // ->push_back(l);
        }
    }

    pub fn print_history(&self, v: Vec<LogItem>) {
        println!("{}", self.history_to_string(v));
    }

    fn history_to_string(&self, v: Vec<LogItem>) -> String {
        let mut sb = String::new();
        if !self.record_history {
            sb.push_str("History was not recorded.");
            if self.print_style == PrintStyle::CSV {
                sb.push_str(" -- ");
            } else {
                sb.push_str(NL);
            }
        }
        for i in 0..v.len() {
            sb.push_str(&(i + 1).to_string());
            sb.push_str(". ");
            sb.push_str(format!("{}", v[i]).as_str());
            if self.print_style == PrintStyle::CSV {
                sb.push_str(" -- ");
            } else {
                sb.push_str(NL);
            }
        }
        if self.print_style == PrintStyle::CSV {
            sb.push_str(",");
        } else {
            sb.push_str(NL);
        }
        sb
    }

    pub fn print_solve_instructions(&self) {
        println!("\nSolve instructions:");
        println!("{}", self.get_solve_instructions_string());
    }

    fn get_solve_instructions_string(&self) -> String {
        if self.is_solved() {
            return self.history_to_string(self.solve_instructions.clone());
        } else {
            return "No solve instructions - Puzzle is not possible to solve.".to_string();
        }
    }

    pub fn get_solve_instructions(&self) -> Vec<LogItem> {
        match self.is_solved() {
            true => self.solve_instructions.clone(),
            false => Vec::new(),
        }
    }

    pub fn print_solve_history(&self) {
        self.print_history(self.solve_history.clone());
    }

    pub fn get_solve_history_string(&self) -> String {
        self.history_to_string(self.solve_history.clone())
    }

    pub fn get_solve_history(&self) -> Vec<LogItem> {
        self.solve_history.clone()
    }

    /// Solve the puzzle.
    pub fn solve(&mut self) -> bool {
        self.reset();
        self.shuffle_random_arrays();
        debug!("Solve round 2");
        self.solve_round(2)
    }

    fn solve_round(&mut self, round: u8) -> bool {
        self.last_solve_round = round;

        while self.single_solve_move(round) {
            if self.is_solved() {
                return true;
            }
            if self.is_impossible() {
                return false;
            }
        }

        let next_guess_round = round + 1;
        let next_round = round + 2;
        let mut guess_number = 0;
        while self.guess(next_guess_round, guess_number) {
            if self.is_impossible() || !self.solve_round(next_round) {
                self.rollback_round(next_round);
                self.rollback_round(next_guess_round);
            } else {
                return true;
            }
            guess_number += 1;
        }
        false
    }

    /**
     * return true if the puzzle has no solutions at all
     */
    pub fn has_no_solution(&mut self) -> bool {
        self.count_solutions_limited() == 0
    }

    /**
     * return true if the puzzle has a solution
     * and only a single solution
     */
    pub fn has_unique_solution(&mut self) -> bool {
        self.count_solutions_limited() == 1
    }

    /**
     * return true if the puzzle has more than one solution
     */

    pub fn has_multiple_solutions(&mut self) -> bool {
        self.count_solutions_limited() > 1
    }

    /**
     * Count the number of solutions to the puzzle
     */
    pub fn count_total_solutions(&mut self) -> u32 {
        self.count_solutions(false)
    }

    /**
     * Count the number of solutions to the puzzle
     * but return two any time there are two or
     * more solutions.  This method will run much
     * faster than count_total_solutions() when there
     * are many possible solutions and can be used
     * when you are interested in knowing if the
     * puzzle has zero, one, or multiple solutions.
     */
    pub fn count_solutions_limited(&mut self) -> u32 {
        self.count_solutions(true)
    }

    fn count_solutions(&mut self, limit_to_two: bool) -> u32 {
        // Don't record history while generating.
        let rec_history = self.record_history;
        self.set_record_history(false);
        let l_history = self.log_history;
        self.set_log_history(false);

        self.reset();
        let solution_count = self.count_solutions_round(2, limit_to_two);

        // Restore recording history.
        self.set_record_history(rec_history);
        self.set_log_history(l_history);

        solution_count
    }

    fn count_solutions_round(&mut self, round: u8, limit_to_two: bool) -> u32 {
        while self.single_solve_move(round) {
            if self.is_solved() {
                self.rollback_round(round);
                return 1;
            }
            if self.is_impossible() {
                self.rollback_round(round);
                return 0;
            }
        }

        let mut solutions = 0;
        let next_round = round + 1;
        let mut guess_number = 0;
        while self.guess(next_round, guess_number) {
            solutions += self.count_solutions_round(next_round, limit_to_two);
            if limit_to_two && solutions >= 2 {
                self.rollback_round(round);
                return solutions;
            }
            guess_number += 1;
        }
        self.rollback_round(round);

        solutions
    }

    fn rollback_round(&mut self, round: u8) {
        if self.log_history || self.record_history {
            self.add_history_item(LogItem::new(
                round,
                LogType::Rollback,
                4294967295,
                4294967295,
            ));
        }

        for i in 0..BOARD_SIZE {
            if self.solution_round[i] == round {
                self.solution_round[i] = 0;
                self.solution[i] = 0;
            }
        }
        for i in 0..POSSIBILITY_SIZE {
            if self.possibilities[i] == round {
                self.possibilities[i] = 0;
            }
        }
        while self.solve_instructions.len() > 0
            && self.solve_instructions.last().unwrap().get_round() == round
        {
            let i = self.solve_instructions.len() - 1;
            self.solve_instructions.remove(i);
        }
    }

    /// Check if the puzzle is solved.
    pub fn is_solved(&self) -> bool {
        for i in 0..BOARD_SIZE {
            if self.solution[i] == 0 {
                return false;
            }
        }
        true
    }

    fn is_impossible(&self) -> bool {
        for position in 0..BOARD_SIZE {
            if self.solution[position] == 0 {
                let mut count = 0;
                for val_index in 0..ROW_COL_SEC_SIZE {
                    let val_pos = QQWing::get_possibility_index(val_index, position);
                    if self.possibilities[val_pos] == 0 {
                        count += 1;
                    }
                }
                if count == 0 {
                    return true;
                }
            }
        }
        false
    }

    fn find_position_with_fewest_possibilities(&self) -> usize {
        let mut min_possibilities = 10;
        let mut best_position = 0;
        for i in 0..BOARD_SIZE {
            let position = self.random_board_array[i];
            if self.solution[position as usize] == 0 {
                let mut count = 0;
                for val_index in 0..ROW_COL_SEC_SIZE {
                    let val_pos = QQWing::get_possibility_index(val_index, position as usize);
                    if self.possibilities[val_pos] == 0 {
                        count += 1;
                    }
                }
                if count < min_possibilities {
                    min_possibilities = count;
                    best_position = position;
                }
            }
        }
        return best_position as usize;
    }

    fn guess(&mut self, round: u8, guess_number: u32) -> bool {
        debug!("Guess round: {}, number: {}", round, guess_number);
        let mut local_guess_count = 0;
        let position = self.find_position_with_fewest_possibilities();
        for i in 0..ROW_COL_SEC_SIZE {
            let val_index = self.random_possibility_array[i];
            let val_pos = QQWing::get_possibility_index(val_index as usize, position);
            if self.possibilities[val_pos] == 0 {
                if local_guess_count == guess_number {
                    let value = val_index + 1;
                    if self.log_history || self.record_history {
                        self.add_history_item(LogItem::new(
                            round,
                            LogType::Guess,
                            value as usize,
                            position,
                        ));
                    }
                    let _ = self.mark(position, round, value).unwrap();
                    return true;
                }
                local_guess_count += 1;
            }
        }
        false
    }

    fn single_solve_move(&mut self, round: u8) -> bool {
        debug!("Single Solve Move, round: {}", round);
        if self.only_possibility_for_cell(round) {
            debug!("only_possibility_for_cell round {} is ture", round);
            return true;
        }
        if self.only_value_in_section(round) {
            debug!("only_value_in_section round {} is ture", round);
            return true;
        }
        if self.only_value_in_row(round) {
            debug!("only_value_in_row round {} is ture", round);
            return true;
        }
        if self.only_value_in_column(round) {
            debug!("only_value_in_column round {} is ture", round);
            return true;
        }
        if self.handle_naked_pairs(round) {
            debug!("handle_naked_pairs round {} is ture", round);
            return true;
        }
        if self.pointing_row_reduction(round) {
            debug!("pointing_row_reduction round {} is ture", round);
            return true;
        }
        if self.pointing_column_reduction(round) {
            debug!("pointing_column_reduction round {} is ture", round);
            return true;
        }
        if self.row_box_reduction(round) {
            debug!("row_box_reduction round {} is ture", round);
            return true;
        }
        if self.col_box_reduction(round) {
            debug!("col_box_reduction round {} is ture", round);
            return true;
        }
        if self.hidden_pair_in_row(round) {
            debug!("hidden_pair_in_row round {} is ture", round);
            return true;
        }
        if self.hidden_pair_in_column(round) {
            debug!("hidden_pair_in_column round {} is ture", round);
            return true;
        }
        if self.hidden_pair_in_section(round) {
            debug!("hidden_pair_in_section round {} is ture", round);
            return true;
        }
        debug!("single_solve_move round {} is false", round);
        false
    }

    fn col_box_reduction(&mut self, round: u8) -> bool {
        debug!("col_box_reduction round: {}", round);
        for val_index in 0..ROW_COL_SEC_SIZE {
            for col in 0..ROW_COL_SEC_SIZE {
                let col_start = col;
                let mut in_one_box = true;
                let mut col_box = UNSET_VALUE;
                for i in 0..GRID_SIZE {
                    for j in 0..GRID_SIZE {
                        let row = i * GRID_SIZE + j;
                        let position = QQWing::row_column_to_cell(row, col);
                        let val_pos = QQWing::get_possibility_index(val_index, position);
                        if self.possibilities[val_pos] == 0 {
                            if col_box == UNSET_VALUE || col_box == i {
                                col_box = i;
                            } else {
                                in_one_box = false;
                            }
                        }
                    }
                }
                if in_one_box && col_box != UNSET_VALUE {
                    let mut done_something = false;
                    let row = GRID_SIZE * col_box;
                    let sec_start =
                        QQWing::cell_to_section_start_cell(QQWing::row_column_to_cell(row, col));
                    let sec_start_row = QQWing::cell_to_row(sec_start);
                    let sec_start_col = QQWing::cell_to_column(sec_start);
                    for i in 0..GRID_SIZE {
                        for j in 0..GRID_SIZE {
                            let row2 = sec_start_row + i;
                            let col2 = sec_start_col + j;
                            let position = QQWing::row_column_to_cell(row2, col2);
                            let val_pos = QQWing::get_possibility_index(val_index, position);
                            if col != col2 && self.possibilities[val_pos] == 0 {
                                self.possibilities[val_pos] = round;
                                done_something = true;
                            }
                        }
                    }
                    if done_something {
                        if self.log_history || self.record_history {
                            self.add_history_item(LogItem::new(
                                round,
                                LogType::ColumnBox,
                                val_index + 1,
                                col_start,
                            ));
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn row_box_reduction(&mut self, round: u8) -> bool {
        debug!("row_box_reduction round: {}", round);
        for val_index in 0..ROW_COL_SEC_SIZE {
            for row in 0..ROW_COL_SEC_SIZE {
                let row_start = row * 9;
                let mut in_one_box = true;
                let mut row_box = UNSET_VALUE;
                for i in 0..GRID_SIZE {
                    for j in 0..GRID_SIZE {
                        let column = i * GRID_SIZE + j;
                        let position = QQWing::row_column_to_cell(row, column);
                        let val_pos = QQWing::get_possibility_index(val_index, position);
                        if self.possibilities[val_pos] == 0 {
                            if row_box == UNSET_VALUE || row_box == i {
                                row_box = i;
                            } else {
                                in_one_box = false;
                            }
                        }
                    }
                }
                if in_one_box && row_box != UNSET_VALUE {
                    let mut done_something = false;
                    let column = GRID_SIZE * row_box;
                    let sec_start =
                        QQWing::cell_to_section_start_cell(QQWing::row_column_to_cell(row, column));
                    let sec_start_row = QQWing::cell_to_row(sec_start);
                    let sec_start_col = QQWing::cell_to_column(sec_start);
                    for i in 0..GRID_SIZE {
                        for j in 0..GRID_SIZE {
                            let row2 = sec_start_row + i;
                            let col2 = sec_start_col + j;
                            let position = QQWing::row_column_to_cell(row2, col2);
                            let val_pos = QQWing::get_possibility_index(val_index, position);
                            if row != row2 && self.possibilities[val_pos] == 0 {
                                self.possibilities[val_pos] = round;
                                done_something = true;
                            }
                        }
                    }
                    if done_something {
                        if self.log_history || self.record_history {
                            self.add_history_item(LogItem::new(
                                round,
                                LogType::RowBox,
                                val_index + 1,
                                row_start,
                            ));
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn pointing_row_reduction(&mut self, round: u8) -> bool {
        debug!("pointing_row_reduction round: {}", round);
        for val_index in 0..ROW_COL_SEC_SIZE {
            for section in 0..ROW_COL_SEC_SIZE {
                let sec_start = QQWing::section_to_first_cell(section);
                let mut in_one_row = true;
                let mut box_row = UNSET_VALUE;
                for j in 0..GRID_SIZE {
                    for i in 0..GRID_SIZE {
                        let sec_val = sec_start + i + (ROW_COL_SEC_SIZE * j);
                        let val_pos = QQWing::get_possibility_index(val_index, sec_val);
                        if self.possibilities[val_pos] == 0 {
                            if box_row == UNSET_VALUE || box_row == j {
                                box_row = j;
                            } else {
                                in_one_row = false;
                            }
                        }
                    }
                }
                if in_one_row && box_row != UNSET_VALUE {
                    let mut done_something = false;
                    let row = QQWing::cell_to_row(sec_start) + box_row;
                    let row_start = row * 9;

                    for i in 0..ROW_COL_SEC_SIZE {
                        let position = row_start + i;
                        let section2 = QQWing::cell_to_section(position);
                        let val_pos = QQWing::get_possibility_index(val_index, position);
                        if section != section2 && self.possibilities[val_pos] == 0 {
                            self.possibilities[val_pos] = round;
                            done_something = true;
                        }
                    }
                    if done_something {
                        if self.log_history || self.record_history {
                            self.add_history_item(LogItem::new(
                                round,
                                LogType::PointingPairTripleRow,
                                val_index + 1,
                                row_start,
                            ));
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn pointing_column_reduction(&mut self, round: u8) -> bool {
        debug!("pointing_column_reduction round: {}", round);
        for val_index in 0..ROW_COL_SEC_SIZE {
            for section in 0..ROW_COL_SEC_SIZE {
                let sec_start = QQWing::section_to_first_cell(section);
                let mut in_one_col = true;
                let mut box_col = UNSET_VALUE;
                for i in 0..GRID_SIZE {
                    for j in 0..GRID_SIZE {
                        let sec_val = sec_start + i + (ROW_COL_SEC_SIZE * j);
                        let val_pos = QQWing::get_possibility_index(val_index, sec_val);
                        if self.possibilities[val_pos] == 0 {
                            if box_col == UNSET_VALUE || box_col == i {
                                box_col = i;
                            } else {
                                in_one_col = false;
                            }
                        }
                    }
                }
                if in_one_col && box_col != UNSET_VALUE {
                    let mut done_something = false;
                    let col = QQWing::cell_to_column(sec_start) + box_col;
                    let col_start = col;

                    for i in 0..ROW_COL_SEC_SIZE {
                        let position = col_start + (ROW_COL_SEC_SIZE * i);
                        let section2 = QQWing::cell_to_section(position);
                        let val_pos = QQWing::get_possibility_index(val_index, position);
                        if section != section2 && self.possibilities[val_pos] == 0 {
                            self.possibilities[val_pos] = round;
                            done_something = true;
                        }
                    }
                    if done_something {
                        if self.log_history || self.record_history {
                            self.add_history_item(LogItem::new(
                                round,
                                LogType::PointingPairTripleColumn,
                                val_index + 1,
                                col_start,
                            ));
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn count_possibilities(&self, position: usize) -> u32 {
        let mut count = 0;
        for val_index in 0..ROW_COL_SEC_SIZE {
            let val_pos = QQWing::get_possibility_index(val_index, position);
            if self.possibilities[val_pos] == 0 {
                count += 1;
            }
        }
        count
    }

    fn are_possibilities_same(&self, position1: usize, position2: usize) -> bool {
        for val_index in 0..ROW_COL_SEC_SIZE {
            let val_pos1 = QQWing::get_possibility_index(val_index, position1);
            let val_pos2 = QQWing::get_possibility_index(val_index, position2);
            if (self.possibilities[val_pos1] == 0 || self.possibilities[val_pos2] == 0)
                && (self.possibilities[val_pos1] != 0 || self.possibilities[val_pos2] != 0)
            {
                return false;
            }
        }
        true
    }

    fn remove_possibilities_in_one_from_two(
        &mut self,
        position1: usize,
        position2: usize,
        round: u8,
    ) -> bool {
        let mut done_something = false;
        for val_index in 0..ROW_COL_SEC_SIZE {
            let val_pos1 = QQWing::get_possibility_index(val_index, position1);
            let val_pos2 = QQWing::get_possibility_index(val_index, position2);

            if self.possibilities[val_pos1] == 0 && self.possibilities[val_pos2] == 0 {
                self.possibilities[val_pos2] = round;
                done_something = true;
            }
        }
        done_something
    }

    fn hidden_pair_in_column(&mut self, round: u8) -> bool {
        debug!("hidden_pair_in_column round: {}", round);
        for column in 0..ROW_COL_SEC_SIZE {
            for val_index in 0..ROW_COL_SEC_SIZE {
                let mut r1 = UNSET_VALUE;
                let mut r2 = UNSET_VALUE;
                let mut val_count = 0;
                for row in 0..ROW_COL_SEC_SIZE {
                    let position = QQWing::row_column_to_cell(row, column);
                    let val_pos = QQWing::get_possibility_index(val_index, position);
                    if self.possibilities[val_pos] == 0 {
                        if r1 == UNSET_VALUE || r1 == row {
                            r1 = row;
                        } else if r2 == UNSET_VALUE || r2 == row {
                            r2 = row;
                        }
                        val_count += 1;
                    }
                }
                if val_count == 2 {
                    for val_index2 in (val_index + 1)..ROW_COL_SEC_SIZE {
                        let mut r3 = UNSET_VALUE;
                        let mut r4 = UNSET_VALUE;
                        let mut val_count2 = 0;
                        for row in 0..ROW_COL_SEC_SIZE {
                            let position = QQWing::row_column_to_cell(row, column);
                            let val_pos = QQWing::get_possibility_index(val_index2, position);
                            if self.possibilities[val_pos] == 0 {
                                if r3 == UNSET_VALUE || r3 == row {
                                    r3 = row;
                                } else if r4 == UNSET_VALUE || r4 == row {
                                    r4 = row;
                                }
                                val_count2 += 1;
                            }
                        }
                        if val_count2 == 2 && r1 == r3 && r2 == r4 {
                            let mut done_something = false;
                            for val_index3 in 0..ROW_COL_SEC_SIZE {
                                if val_index3 != val_index && val_index3 != val_index2 {
                                    let position1 = QQWing::row_column_to_cell(r1, column);
                                    let position2 = QQWing::row_column_to_cell(r2, column);
                                    let val_pos1 =
                                        QQWing::get_possibility_index(val_index3, position1);
                                    let val_pos2 =
                                        QQWing::get_possibility_index(val_index3, position2);
                                    if self.possibilities[val_pos1] == 0 {
                                        self.possibilities[val_pos1] = round;
                                        done_something = true;
                                    }
                                    if self.possibilities[val_pos2] == 0 {
                                        self.possibilities[val_pos2] = round;
                                        done_something = true;
                                    }
                                }
                            }
                            if done_something {
                                if self.log_history || self.record_history {
                                    self.add_history_item(LogItem::new(
                                        round,
                                        LogType::HiddenPairColumn,
                                        val_index + 1,
                                        QQWing::row_column_to_cell(r1, column),
                                    ));
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn hidden_pair_in_section(&mut self, round: u8) -> bool {
        debug!("hidden_pair_in_section round: {}", round);
        for section in 0..ROW_COL_SEC_SIZE {
            for val_index in 0..ROW_COL_SEC_SIZE {
                let mut si1 = UNSET_VALUE;
                let mut si2 = UNSET_VALUE;
                let mut val_count = 0;
                for sec_ind in 0..ROW_COL_SEC_SIZE {
                    let position = QQWing::section_to_cell(section, sec_ind);
                    let val_pos = QQWing::get_possibility_index(val_index, position);
                    if self.possibilities[val_pos] == 0 {
                        if si1 == UNSET_VALUE || si1 == sec_ind {
                            si1 = sec_ind;
                        } else if si2 == UNSET_VALUE || si2 == sec_ind {
                            si2 = sec_ind;
                        }
                        val_count += 1;
                    }
                }
                if val_count == 2 {
                    for val_index2 in (val_index + 1)..ROW_COL_SEC_SIZE {
                        let mut si3 = UNSET_VALUE;
                        let mut si4 = UNSET_VALUE;
                        let mut val_count2 = 0;
                        for sec_ind in 0..ROW_COL_SEC_SIZE {
                            let position = QQWing::section_to_cell(section, sec_ind);
                            let val_pos = QQWing::get_possibility_index(val_index2, position);
                            if self.possibilities[val_pos] == 0 {
                                if si3 == UNSET_VALUE || si3 == sec_ind {
                                    si3 = sec_ind;
                                } else if si4 == UNSET_VALUE || si4 == sec_ind {
                                    si4 = sec_ind;
                                }
                                val_count2 += 1;
                            }
                        }
                        if val_count2 == 2 && si1 == si3 && si2 == si4 {
                            let mut done_something = false;
                            for val_index3 in 0..ROW_COL_SEC_SIZE {
                                if val_index3 != val_index && val_index3 != val_index2 {
                                    let position1 = QQWing::section_to_cell(section, si1);
                                    let position2 = QQWing::section_to_cell(section, si2);
                                    let val_pos1 =
                                        QQWing::get_possibility_index(val_index3, position1);
                                    let val_pos2 =
                                        QQWing::get_possibility_index(val_index3, position2);
                                    if self.possibilities[val_pos1] == 0 {
                                        self.possibilities[val_pos1] = round;
                                        done_something = true;
                                    }
                                    if self.possibilities[val_pos2] == 0 {
                                        self.possibilities[val_pos2] = round;
                                        done_something = true;
                                    }
                                }
                            }
                            if done_something {
                                if self.log_history || self.record_history {
                                    self.add_history_item(LogItem::new(
                                        round,
                                        LogType::HiddenPairSection,
                                        val_index + 1,
                                        QQWing::section_to_cell(section, si1),
                                    ));
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn hidden_pair_in_row(&mut self, round: u8) -> bool {
        debug!("hidden_pair_in_row round: {}", round);
        for row in 0..ROW_COL_SEC_SIZE {
            for val_index in 0..ROW_COL_SEC_SIZE {
                let mut c1 = UNSET_VALUE;
                let mut c2 = UNSET_VALUE;
                let mut val_count = 0;
                for column in 0..ROW_COL_SEC_SIZE {
                    let position = QQWing::row_column_to_cell(row, column);
                    let val_pos = QQWing::get_possibility_index(val_index, position);
                    if self.possibilities[val_pos] == 0 {
                        if c1 == UNSET_VALUE || c1 == column {
                            c1 = column;
                        } else if c2 == UNSET_VALUE || c2 == column {
                            c2 = column;
                        }
                        val_count += 1;
                    }
                }
                if val_count == 2 {
                    for val_index2 in (val_index + 1)..ROW_COL_SEC_SIZE {
                        let mut c3 = UNSET_VALUE;
                        let mut c4 = UNSET_VALUE;
                        let mut val_count2 = 0;
                        for column in 0..ROW_COL_SEC_SIZE {
                            let position = QQWing::row_column_to_cell(row, column);
                            let val_pos = QQWing::get_possibility_index(val_index2, position);
                            if self.possibilities[val_pos] == 0 {
                                if c3 == UNSET_VALUE || c3 == column {
                                    c3 = column;
                                } else if c4 == UNSET_VALUE || c4 == column {
                                    c4 = column;
                                }
                                val_count2 += 1;
                            }
                        }
                        if val_count2 == 2 && c1 == c3 && c2 == c4 {
                            let mut done_something = false;
                            for val_index3 in 0..ROW_COL_SEC_SIZE {
                                if val_index3 != val_index && val_index3 != val_index2 {
                                    let position1 = QQWing::row_column_to_cell(row, c1);
                                    let position2 = QQWing::row_column_to_cell(row, c2);
                                    let val_pos1 =
                                        QQWing::get_possibility_index(val_index3, position1);
                                    let val_pos2 =
                                        QQWing::get_possibility_index(val_index3, position2);
                                    if self.possibilities[val_pos1] == 0 {
                                        self.possibilities[val_pos1] = round;
                                        done_something = true;
                                    }
                                    if self.possibilities[val_pos2] == 0 {
                                        self.possibilities[val_pos2] = round;
                                        done_something = true;
                                    }
                                }
                            }
                            if done_something {
                                if self.log_history || self.record_history {
                                    self.add_history_item(LogItem::new(
                                        round,
                                        LogType::HiddenPairRow,
                                        val_index + 1,
                                        QQWing::row_column_to_cell(row, c1),
                                    ));
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn handle_naked_pairs(&mut self, round: u8) -> bool {
        debug!("handle_naked_pairs round: {}", round);
        for position in 0..BOARD_SIZE {
            let possibilities = self.count_possibilities(position);
            if possibilities == 2 {
                let row = QQWing::cell_to_row(position);
                let column = QQWing::cell_to_column(position);
                let section = QQWing::cell_to_section_start_cell(position);
                for position2 in position..BOARD_SIZE {
                    if position != position2 {
                        let possibilities2 = self.count_possibilities(position2);
                        if possibilities2 == 2 && self.are_possibilities_same(position, position2) {
                            if row == QQWing::cell_to_row(position2) {
                                let mut done_something = false;
                                for column2 in 0..ROW_COL_SEC_SIZE {
                                    let position3 = QQWing::row_column_to_cell(row, column2);
                                    if position3 != position
                                        && position3 != position2
                                        && self.remove_possibilities_in_one_from_two(
                                            position, position3, round,
                                        )
                                    {
                                        done_something = true;
                                    }
                                }
                                if done_something {
                                    if self.log_history || self.record_history {
                                        self.add_history_item(LogItem::new(
                                            round,
                                            LogType::NakedPairRow,
                                            0,
                                            position,
                                        ));
                                    }
                                    return true;
                                }
                            }
                            if column == QQWing::cell_to_column(position2) {
                                let mut done_something = false;
                                for row2 in 0..ROW_COL_SEC_SIZE {
                                    let position3 = QQWing::row_column_to_cell(row2, column);
                                    if position3 != position
                                        && position3 != position2
                                        && self.remove_possibilities_in_one_from_two(
                                            position, position3, round,
                                        )
                                    {
                                        done_something = true;
                                    }
                                }
                                if done_something {
                                    if self.log_history || self.record_history {
                                        self.add_history_item(LogItem::new(
                                            round,
                                            LogType::NakedPairColumn,
                                            0,
                                            position,
                                        ));
                                    }
                                    return true;
                                }
                            }
                            if section == QQWing::cell_to_section_start_cell(position2) {
                                let mut done_something = false;
                                let sec_start = QQWing::cell_to_section_start_cell(position);
                                for i in 0..GRID_SIZE {
                                    for j in 0..GRID_SIZE {
                                        let position3 = sec_start + i + (ROW_COL_SEC_SIZE * j);
                                        if position3 != position
                                            && position3 != position2
                                            && self.remove_possibilities_in_one_from_two(
                                                position, position3, round,
                                            )
                                        {
                                            done_something = true;
                                        }
                                    }
                                }
                                if done_something {
                                    if self.log_history || self.record_history {
                                        self.add_history_item(LogItem::new(
                                            round,
                                            LogType::NakedPairSection,
                                            0,
                                            position,
                                        ));
                                    }
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /**
     * Mark exactly one cell which is the only possible value for some row, if
     * such a cell exists. This method will look in a row for a possibility that
     * is only listed for one cell. This type of cell is often called a
     * "hidden single"
     */
    fn only_value_in_row(&mut self, round: u8) -> bool {
        debug!("only_value_in_row round: {}", round);
        for row in 0..ROW_COL_SEC_SIZE {
            for val_index in 0..ROW_COL_SEC_SIZE {
                let mut count = 0;
                let mut last_position = 0;
                for col in 0..ROW_COL_SEC_SIZE {
                    let position = (row * ROW_COL_SEC_SIZE) + col;
                    let val_pos = QQWing::get_possibility_index(val_index, position);
                    if self.possibilities[val_pos] == 0 {
                        count += 1;
                        last_position = position;
                    }
                }
                if count == 1 {
                    let value = val_index + 1;
                    if self.log_history || self.record_history {
                        self.add_history_item(LogItem::new(
                            round,
                            LogType::HiddenSingleRow,
                            value,
                            last_position,
                        ));
                    }
                    let _ = self.mark(last_position, round, value as u8).unwrap();
                    return true;
                }
            }
        }
        false
    }

    /**
     * Mark exactly one cell which is the only possible value for some column,
     * if such a cell exists. This method will look in a column for a
     * possibility that is only listed for one cell. This type of cell is often
     * called a "hidden single"
     */
    fn only_value_in_column(&mut self, round: u8) -> bool {
        debug!("only_value_in_column round: {}", round);
        for col in 0..ROW_COL_SEC_SIZE {
            for val_index in 0..ROW_COL_SEC_SIZE {
                let mut count = 0;
                let mut last_position = 0;
                for row in 0..ROW_COL_SEC_SIZE {
                    let position = QQWing::row_column_to_cell(row, col);
                    let val_pos = QQWing::get_possibility_index(val_index, position);
                    if self.possibilities[val_pos] == 0 {
                        count += 1;
                        last_position = position;
                    }
                }
                if count == 1 {
                    let value = val_index + 1;
                    if self.log_history || self.record_history {
                        self.add_history_item(LogItem::new(
                            round,
                            LogType::HiddenSingleColumn,
                            value,
                            last_position,
                        ));
                    }
                    let _ = self.mark(last_position, round, value as u8).unwrap();
                    return true;
                }
            }
        }
        false
    }

    /**
     * Mark exactly one cell which is the only possible value for some section,
     * if such a cell exists. This method will look in a section for a
     * possibility that is only listed for one cell. This type of cell is often
     * called a "hidden single"
     */
    fn only_value_in_section(&mut self, round: u8) -> bool {
        debug!("only_value_in_section round: {}", round);
        for sec in 0..ROW_COL_SEC_SIZE {
            let sec_pos = QQWing::section_to_first_cell(sec);
            for val_index in 0..ROW_COL_SEC_SIZE {
                let mut count = 0;
                let mut last_position = 0;
                for i in 0..GRID_SIZE {
                    for j in 0..GRID_SIZE {
                        let position = sec_pos + i + ROW_COL_SEC_SIZE * j;
                        let val_pos = QQWing::get_possibility_index(val_index, position);
                        if self.possibilities[val_pos] == 0 {
                            count += 1;
                            last_position = position;
                        }
                    }
                }
                if count == 1 {
                    let value = val_index + 1;
                    if self.log_history || self.record_history {
                        self.add_history_item(LogItem::new(
                            round,
                            LogType::HiddenSingleSection,
                            value,
                            last_position,
                        ));
                    }
                    let _ = self.mark(last_position, round, value as u8).unwrap();
                    return true;
                }
            }
        }
        false
    }

    /**
     * Mark exactly one cell that has a single possibility, if such a cell
     * exists. This method will look for a cell that has only one possibility.
     * This type of cell is often called a "single"
     */
    fn only_possibility_for_cell(&mut self, round: u8) -> bool {
        debug!("only_possibility_for_cell round: {}", round);
        for position in 0..BOARD_SIZE {
            if self.solution[position] == 0 {
                let mut count = 0;
                let mut last_value = 0;
                for val_index in 0..ROW_COL_SEC_SIZE {
                    let val_pos = QQWing::get_possibility_index(val_index, position);
                    if self.possibilities[val_pos] == 0 {
                        count += 1;
                        last_value = val_index + 1;
                    }
                }
                if count == 1 {
                    let _ = self.mark(position, round, last_value as u8).unwrap();
                    if self.log_history || self.record_history {
                        self.add_history_item(LogItem::new(
                            round,
                            LogType::Single,
                            last_value,
                            position,
                        ));
                    }
                    return true;
                }
            }
        }
        false
    }

    /**
     * Mark the given value at the given position. Go through the row, column,
     * and section for the position and remove the value from the possibilities.
     *
     * @param position Position into the board (0-80)
     * @param round Round to mark for rollback purposes
     * @param value The value to go in the square at the given position
     */
    fn mark(&mut self, position: usize, round: u8, value: u8) -> Result<bool, QQWingError> {
        debug!(
            "Mark position: {}, round: {}, value: {}",
            position, round, value
        );
        if self.solution[position] != 0 {
            return Err(QQWingError::PositionAlreadyMarked);
        }
        if self.solution_round[position] != 0 {
            return Err(QQWingError::PositionAlreadyMarked);
        }

        let val_index = value - 1;
        self.solution[position] = value;

        let poss_ind = QQWing::get_possibility_index(val_index as usize, position);
        if self.possibilities[poss_ind] != 0 {
            return Err(QQWingError::PositionAlreadyMarked);
        }

        // Take this value out of the possibilities for everything in the row
        self.solution_round[position] = round;
        let row_start = QQWing::cell_to_row(position) * ROW_COL_SEC_SIZE;
        for col in 0..ROW_COL_SEC_SIZE {
            let row_val = row_start + col;
            let val_pos = QQWing::get_possibility_index(val_index as usize, row_val);
            // System.out.println("Row Start: "+row_start+" Row Value: "+rowVal+" Value Position: "+val_pos);
            if self.possibilities[val_pos] == 0 {
                self.possibilities[val_pos] = round;
            }
        }

        // Take this value out of the possibilities for everything in the column
        let col_start = QQWing::cell_to_column(position);
        for i in 0..ROW_COL_SEC_SIZE {
            let col_val = col_start + (ROW_COL_SEC_SIZE * i);
            let val_pos = QQWing::get_possibility_index(val_index as usize, col_val);
            // System.out.println("Col Start: "+col_start+" Col Value: "+colVal+" Value Position: "+val_pos);
            if self.possibilities[val_pos] == 0 {
                self.possibilities[val_pos] = round;
            }
        }

        // Take this value out of the possibilities for everything in section
        let sec_start = QQWing::cell_to_section_start_cell(position);
        for i in 0..GRID_SIZE {
            for j in 0..GRID_SIZE {
                let sec_val = sec_start + i + (ROW_COL_SEC_SIZE * j);
                let val_pos = QQWing::get_possibility_index(val_index as usize, sec_val);
                // System.out.println("Sec Start: "+sec_start+" Sec Value: "+sec_val+" Value Position: "+val_pos);
                if self.possibilities[val_pos] == 0 {
                    self.possibilities[val_pos] = round;
                }
            }
        }

        // This position itself is determined, it should have possibilities.
        for val_index in 0..ROW_COL_SEC_SIZE {
            let val_pos = QQWing::get_possibility_index(val_index as usize, position);
            if self.possibilities[val_pos] == 0 {
                self.possibilities[val_pos] = round;
            }
        }
        Ok(true)
    }

    /**
     * print the given BOARD_SIZEd array of ints as a sudoku puzzle. Use print
     * options from member variables.
     */
    fn print(&self, sudoku: [u8; 81]) {
        println!("{}", self.puzzle_to_string(sudoku));
    }

    fn puzzle_to_string(&self, sudoku: [u8; 81]) -> String {
        let mut sb = String::new();
        for i in 0..BOARD_SIZE {
            if self.print_style == PrintStyle::READABLE {
                sb.push_str(" ");
            }
            if sudoku[i] == 0 {
                sb.push_str(".");
            } else {
                sb.push_str(sudoku[i].to_string().as_str());
            }
            if i == BOARD_SIZE - 1 {
                if self.print_style == PrintStyle::CSV {
                    sb.push_str(",");
                } else {
                    sb.push_str(NL);
                }
                if self.print_style == PrintStyle::READABLE
                    || self.print_style == PrintStyle::COMPACT
                {
                    sb.push_str(NL);
                }
            } else if i % ROW_COL_SEC_SIZE == ROW_COL_SEC_SIZE - 1 {
                if self.print_style == PrintStyle::READABLE
                    || self.print_style == PrintStyle::COMPACT
                {
                    sb.push_str(NL);
                }
                if i % SEC_GROUP_SIZE == SEC_GROUP_SIZE - 1 {
                    if self.print_style == PrintStyle::READABLE {
                        sb.push_str("-------|-------|-------");
                        sb.push_str(NL);
                    }
                }
            } else if i % GRID_SIZE == GRID_SIZE - 1 {
                if self.print_style == PrintStyle::READABLE {
                    sb.push_str(" |");
                }
            }
        }
        sb
    }

    /// Print any stats we were able to gather while solving the puzzle.
    pub fn get_stats(&self) -> String {
        let mut sb = String::new();
        let given_count = self.get_given_count();
        let single_count = self.get_single_count();
        let hidden_single_count = self.get_hidden_single_count();
        let naked_pair_count = self.get_naked_pair_count();
        let hidden_pair_count = self.get_hidden_pair_count();
        let pointing_pair_triple_count = self.get_pointing_pair_triple_count();
        let box_reduction_count = self.get_box_line_reduction_count();
        let guess_count = self.get_guess_count();
        let backtrack_count = self.get_backtrack_count();
        let difficulty_string = self.get_difficulty();
        if self.print_style == PrintStyle::CSV {
            sb.push_str(format!("{:?}", difficulty_string).as_str());
            sb.push_str(",");
            sb.push_str(given_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(single_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(hidden_single_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(naked_pair_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(hidden_pair_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(pointing_pair_triple_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(box_reduction_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(guess_count.to_string().as_str());
            sb.push_str(",");
            sb.push_str(backtrack_count.to_string().as_str());
            sb.push_str(",");
        } else {
            sb.push_str("Difficulty: ");
            sb.push_str(format!("{:?}", difficulty_string).as_str());
            sb.push_str(NL);
            sb.push_str("Number of Givens: ");
            sb.push_str(given_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Singles: ");
            sb.push_str(single_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Hidden Singles: ");
            sb.push_str(hidden_single_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Naked Pairs: ");
            sb.push_str(naked_pair_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Hidden Pairs: ");
            sb.push_str(hidden_pair_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Pointing Pairs/Triples: ");
            sb.push_str(pointing_pair_triple_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Box/Line Intersections: ");
            sb.push_str(box_reduction_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Guesses: ");
            sb.push_str(guess_count.to_string().as_str());
            sb.push_str(NL);
            sb.push_str("Number of Backtracks: ");
            sb.push_str(backtrack_count.to_string().as_str());
            sb.push_str(NL);
        }
        sb
    }

    /**
     * Print the sudoku puzzle.
     */
    pub fn print_puzzle(&self) {
        self.print(self.puzzle);
    }

    /**
     * Given a vector of LogItems, determine how many log items in the vector
     * are of the specified type.
     */
    fn get_log_count(v: &Vec<LogItem>, logtype: LogType) -> usize {
        let mut count = 0;
        for i in 0..v.len() {
            if v[i].log_type == logtype {
                count += 1;
            }
        }
        return count;
    }

    fn get_random_symmetry() -> Symmetry {
        let values = [
            Symmetry::NONE,
            Symmetry::ROTATE90,
            Symmetry::ROTATE180,
            Symmetry::MIRROR,
            Symmetry::FLIP,
            Symmetry::RANDOM,
        ];
        // not the first and last value which are NONE and RANDOM
        values[(random::<usize>() % (values.len() - 1)) + 1].clone()
    }

    /**
     * Given a value for a cell (0-8) and a cell number (0-80) calculate the
     * offset into the possibility array (0-728).
     */
    pub(crate) fn get_possibility_index(value_index: usize, cell: usize) -> usize {
        value_index + (ROW_COL_SEC_SIZE * cell)
    }

    /**
     * Given the index of a cell (0-80) calculate the row (0-8) in which it
     * resides.
     */
    pub(crate) fn cell_to_row(cell: usize) -> usize {
        cell / ROW_COL_SEC_SIZE
    }

    /**
     * Given the index of a cell (0-80) calculate the column (0-8) in which that
     * cell resides.
     */
    pub(crate) fn cell_to_column(cell: usize) -> usize {
        cell % ROW_COL_SEC_SIZE
    }

    /**
     * Given the index of a cell (0-80) calculate the section (0-8) in which it
     * resides.
     */
    pub(crate) fn cell_to_section(cell: usize) -> usize {
        (cell / SEC_GROUP_SIZE * GRID_SIZE) + (QQWing::cell_to_column(cell) / GRID_SIZE)
    }

    /**
     * Given the index of a cell (0-80) calculate the cell (0-80) that is the
     * upper left start cell of that section.
     */
    pub(crate) fn cell_to_section_start_cell(cell: usize) -> usize {
        (cell / SEC_GROUP_SIZE * SEC_GROUP_SIZE)
            + (QQWing::cell_to_column(cell) / GRID_SIZE * GRID_SIZE)
    }

    /**
     * Given a row (0-8) and a column (0-8) calculate the cell (0-80).
     */
    pub(crate) fn row_column_to_cell(row: usize, column: usize) -> usize {
        row * ROW_COL_SEC_SIZE + column
    }

    /**
     * Given a section (0-8) calculate the first cell (0-80) of that section.
     */
    pub(crate) fn section_to_first_cell(section: usize) -> usize {
        (section % GRID_SIZE * GRID_SIZE) + (section / GRID_SIZE * SEC_GROUP_SIZE)
    }

    /**
     * Given a section (0-8) and an offset into that section (0-8) calculate the
     * cell (0-80)
     */
    pub(crate) fn section_to_cell(section: usize, offset: usize) -> usize {
        QQWing::section_to_first_cell(section)
            + ((offset / GRID_SIZE) * ROW_COL_SEC_SIZE)
            + (offset % GRID_SIZE)
    }
}

#[derive(Debug, PartialEq, Clone, EnumString, EnumIter)]
pub enum PrintStyle {
    ONELINE,
    COMPACT,
    READABLE,
    CSV,
}
