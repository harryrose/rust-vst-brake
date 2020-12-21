# Braker

A simple VST audio effect plugin written in rust, which acts as a brake on a track.

It stores a 10 second buffer of audio from when the brake is applied, and slows the track
to a stop.

It provides two parameters:

 - `brake` : `0 - 1.0` -- if > 0.5, the brake is applied to the track.
 - `brake rate` : `0 - 1.0` -- the rate of braking on the track.  The closer to 1, the slower it brakes -- i.e. it takes longer to stop.

Currently being used as a quick and dirty first dive into rust.
