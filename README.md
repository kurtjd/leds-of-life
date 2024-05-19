# LEDs of Life
|Front|Back|
|-----|----|
|[<img src = "pictures/leds_of_life_video.png?raw=true">](https://www.youtube.com/shorts/s3XXKaWJPEc)|<img src = "pictures/leds_of_life_wiring.jpg?raw=true">|

Just something I threw together for fun. John Conway's Game of Life implemented on a 16x16 LED matrix
powered by a STM32 BluePill board (with Rust firmware!). Click the picture above to see it in action!

Yes, the wiring and soldering took me a long time...

## How to Use
Your USB port likely cannot provide enough current, so you should use a 5V wall
port to connect a USB-to-MicroUSB cable from the wall to the device. Do not also simultaneously
power the board with a programmer.

However, once device is successfully powered, flip the switch on and you should see the glider symbol
as well as a crosshair appear. Use the buttons to move the crosshair around and to toggle individual
cells. Use the pause button to resume/pause the simulation.

The potentiometers can be used to control speed and LED brightness as well.

## Run/Flash
After ensuring programmer is attached correctly, simply run:  
`cargo run`

## Schematic
Please see `schematic.pdf` for the electrical schematic (in case you want to build your own and do
a much better wiring job than my mess).

## License
This project is licensed under the MIT license and is completely free to use and modify.