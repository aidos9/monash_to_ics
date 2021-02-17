# monash_to_ics
Converts a Monash .xls timetable to a .ics file. This file can be easily imported into most calendar applications. 
This program was hacked together in a couple hours, so I'm sure there will be some bugs, if you encounter any or are
desparate for any feature just open a new issue and I might solve it.

## Usage
Using this tools is as simple as running the program from the command line using your timetable as an input.
```
./monash_to_ics timetable.xls -o output.ics
```

## Issues
If you have any problems and need me to solve them open an issue explaining the problem and INCLUDE your timetable file.

## Building
To build this program for yourself simply ensure you have the Rust language installed with Cargo and run
```
cargo build --release
```
