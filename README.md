# Yactatt — Yet Another (Unofficial!) CTA Transit Tracker

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
- [ ] Runtime args for the following:
  - LED matrix rows
  - LED matrix columns
  - Bus routes
  - Train routes
  - Bus stops
  - Train stations
  - Refresh rate
- [ ] Config file in TOML, supplementing/overridden by the above

## Background

I'm an Elixir/ReScript developer by day and I don't really know Rust!

This started out as a way to leverage Nerves to try to get down to the hardware level bit as time
goes by I am skeptical of doing _that_ much work when perfectly good LED panel libraries exist and
support languages such as Rust, which is one I've been itching to learn for a while.

Pardon my dust.
