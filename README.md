# Lox_Weather_RS

## Intro

This is a weather server implementation using PirateWeather API for loxone written in rust.
This is purely for educational/debugging purposes. Don't use this to replace the actual loxone weather service

> [!WARNING]
> This is not really usable anymore in it's current form as a replacement for the loxone weather service. Since Loxone 16 the validity of the SSL cert for weather.loxone.com is checked by the miniserver, which means the DNS trick below does not work anymore. Unless future updates allows importing custom CA's in the miniserver, it will never work like this anymore :(
>
> I added a new endpoint on /current/temp/, with the same query params that will just return the current outside temperature for use in the loxone (via the virtual inputs system).

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

## Test
You can fake a query like loxone does it with curl:
    curl http://weather.loxone.com/forecast/?user=loxone_XXXXXXXXXXXX&coord=X.YYYY,X.YYYY&asl=9&format=2&new_api=1

## Credits

- https://github.com/sarnau/Inside-The-Loxone-Miniserver/
- https://github.com/mschlenstedt/LoxBerry-Plugin-Weather4Lox
- http://pirateweather.net/en/latest/

## Issues

Feel free to let me know if you encounter any issues.
