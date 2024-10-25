# QQwing
QQwing is software for generating and solving Sudoku puzzles. This is a rust version.


```
PS D:\Projects\qqwing> .\target\debug\qqwing.exe
Usage: qqwing.exe [OPTIONS] <COMMAND>

Commands:
  generate  Generate a puzzle
  solve     Solve a puzzle
  help      Print this message or the help of the given subcommand(s)

Options:
  -f, --file <FILE>                        Input or Output puzzle file
  -v, --verbose...                         Show more verbose information
  -p, --ps <ONELINE,COMPACT,READABLE,CSV>  Set print style [default: READABLE]
  -h, --help                               Print help
  -V, --version                            Print version
```
### License
```
qqwing - Sudoku solver and generator
Copyright (C) 2006-2014 Stephen Ostermiller http://ostermiller.org/
Copyright (C) 2007 Jacques Bensimon (jacques@ipm.com)
Copyright (C) 2007 Joel Yarde (joel.yarde - gmail.com)

This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; either version 2 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program; if not, write to the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
```

### Crates.io
* https://crates.io/crates/qqwing
### github
* https://github.com/adungaos/qqwing
### TODO
1. 多线程
2. 命令行工具完善

### 参考资料
* [QQWing Sudoku](https://qqwing.com/)，[Stephen Ostermiller](https://ostermiller.org/)
