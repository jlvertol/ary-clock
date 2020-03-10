To run the application you need to have Rust and Cargo installed. Once both are installed in your system, clone this project, navigate to the root folder, and execute `cargo run` in the terminal.

You will be presented with an ncurses-style small blue rectangle with the time on the header.

To quit the program press `q`
To add an alarm type two digits representing a minute, all integer values between 00 and 59 inclusive are valid. The number will be added to a list inside the rectangle, representing the alarm has been added.
You can store up to 10 different alarms in the list.
The clock works by playing `soundalarm.wav` every hour at the minutes marked by the alarm.
To remove an alarm, type its number again.
To create an alarm that plays the alternate file `soundalarm2.wav`, type `*` followed by your two-digit number.
Those alarms can also be removed simply by typing their number again (no need to write down `*`)

This software was made bespoke for a customer who graciously let me open source it to the world.

It can be cross-compiled for Windows and Linux on ARM, and since it's a big pain to do that I published the binaries in the releases folder.

This software is released under the MIT License, the binaries link statically to the GNU C libraries and as such they are licensed under the GPL.
