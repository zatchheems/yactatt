# Yactatt — Yet Another (Unofficial!) CTA Transit Tracker

![alt-text](https://user-images.githubusercontent.com/8506829/232356892-81f47b17-7f56-47c8-bfd1-25f0210cbefc.jpg "An early splash screen displaying CTA colors and the text \"YACTATT\" on a 64x32 LED matrix.")
_For display on an LED array; early prototype optimized for 64x32 displays._

---

Basic Rust project making use of [Chicago Transit Authority data](https://www.transitchicago.com/developers/)
to display bus and train times on an LED matrix.

Powered by [rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix),
by way of [rpi-led-panel](https://github.com/EmbersArc/rpi_led_panel).

Remember to be kind to your drivers and conductors: they deserve better benefits.

## TODOs

- [ ] Poll CTA API for data
- [ ] Write out to LED panel, showing:
  - Route
  - Heading
  - Wait time
  - Delayed?
  - Run ID (optional)
  - Cool animations (optional)
- [x] Basic splash screen
  - [ ] Cool upgrades showing my handle, system version, package version
- [ ] Runtime args for the following:
  - [x] LED matrix rows
  - [x] LED matrix columns
  - [ ] Bus routes and stops
  - [ ] Train routes and stations
  - [x] Refresh rate
  - [x] "Silent" mode to work without LED panel
- [ ] Config file in TOML, supplementing/overridden by the above
- [ ] Tests

## Background

I'm an Elixir/ReScript developer by day and I don't really know Rust!

This started out as a way to leverage Nerves to try to get down to the hardware level but as time
goes by I am skeptical of doing _that_ much work when perfectly good LED panel libraries exist and
support languages such as Rust, which is one I've been itching to learn for a while.

Pardon my dust.
