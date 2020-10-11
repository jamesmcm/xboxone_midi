# xboxone_midi

Test using Xbox One controller for MIDI synthesizer, written in Rust
using the `midir` and `stick` crates. 

Delay is currently too large to be usable, try:

* Testing without using a Bluetooth headset.
* Using JACK instead of ALSA (with the `jack` feature in midir).
* Using a real-time patched Linux kernel: https://wiki.archlinux.org/index.php/Professional_audio#Realtime_kernel
