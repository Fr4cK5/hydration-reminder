## Hydration Reminder

A small, unintrusive application to remind you to hydrate!

## Usage

Just let it run! You will be reminded to hydrate every 20 minutes.

No notifications or sounds, just some text changing its color. (Yes, this was made for setups with more than 1 monitors.)

Once reminded, just click the text that says "Hydrate 💧" and it'll shut up for 20 minutes.

Hovering your mouse pointer over the window will show you the elapsed
time since either the last reminder or since the current reminder has started.

Additionally, if you happen to hydrate before the timer runs out,
you can click the text with your secondary mouse button (usually the right one) to reset the timer.

## Configuration

In order to configure the reminder interval to your liking, change value for `reminder_interval` in `hrconfig.json`.
A schema is present and can be easily referred to in order to understand the duration syntax.

The schema can be found [here](https://github.com/Fr4cK5/hydration-reminder/blob/master/schema.json) (or as [raw](https://raw.githubusercontent.com/Fr4cK5/hydration-reminder/refs/heads/master/schema.json)), and is already present in the default `hrconfig.json` file.

Make sure this config file is always placed next to the binary, such that the binary can access it by reading `./hrconfig.json`.

## Build

```sh
cargo build --profile opt
```
