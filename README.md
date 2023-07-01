# Lox_Weather_RS

## Intro

This is a weather server implementation using PirateWeather API for loxone written in rust.
This is purely for educational/debugging purposes. Don't use this to replace the actual loxone weather service

## Build

Build via docker build

## Run

- Run the docker container on port 6066.
- Following env variables should be present:

  - COUNTRY_NAME=your country
  - CITY_NAME=your city
  - PIRATEWEATHER_API_KEY=your key for the pirate weather api

    note: Country and city name are data expected by the miniserver, the actual weather values are queried with the longitude and latitude values filled in in the loxone config.

- Add your own DNS server to your loxone and point 'weather.loxone.com' to this weather server.
- The weather service data should show up in Loxone!

## Credits

- https://github.com/sarnau/Inside-The-Loxone-Miniserver/
- https://github.com/mschlenstedt/LoxBerry-Plugin-Weather4Lox
- http://pirateweather.net/en/latest/

## Issues

Feel free to
