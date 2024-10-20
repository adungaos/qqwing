#[derive(Debug, PartialEq, Clone)]
pub enum LogType {
    Given,                       //("Mark given"),
    Single,                      //("Mark only possibility for cell"),
    HiddenSingleRow,           //("Mark single possibility for value in row"),
    HiddenSingleColumn,        //("Mark single possibility for value in column"),
    HiddenSingleSection,       //("Mark single possibility for value in section"),
    Guess,                       //("Mark guess , //(start round)"),
    Rollback,                    //("Roll back round"),
    NakedPairRow,              //("Remove possibilities for naked pair in row"),
    NakedPairColumn,           //("Remove possibilities for naked pair in column"),
    NakedPairSection,          //("Remove possibilities for naked pair in section"),
    PointingPairTripleRow, //("Remove possibilities for row because all values are in one section"),
    PointingPairTripleColumn, //("Remove possibilities for column because all values are in one section"),
    RowBox,         //("Remove possibilities for section because all values are in one row"),
    ColumnBox,      //("Remove possibilities for section because all values are in one column"),
    HiddenPairRow, //("Remove possibilities from hidden pair in row"),
    HiddenPairColumn, //("Remove possibilities from hidden pair in column"),
    HiddenPairSection, //("Remove possibilities from hidden pair in section");
}
