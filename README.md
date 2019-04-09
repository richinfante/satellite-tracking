# satellite-rs demo

This demo requires a few environment variables to be set:
- `STATION_LATITUDE` - latitude (in degrees) for ground station.  ex: "-84.17723"
- `STATION_LONGITUDE` - latitude (in degrees) for ground station. ex: "20.238393"
- `STATION_ALTITUDE` - altitude (in km) for the ground station.
- `SPACETRACK_IDENTITY` - login name for space-track.org (required if using space-track)
- `SPACETRACK_SECRET` - login password for space-track.org (required if using space-track)

The easiest way to do this is to place all the variables in a file named ".env" in the project root:
```
STATION_LATITUDE="00.00000"
STATION_LONGITUDE="00.00000"
STATION_ALTITUDE="0.0"
SPACETRACK_IDENTITY="user@example.com"
SPACETRACK_SECRET="password"
```

Then, you can run the tracker with the following command:
```bash
env $(cat .env | xargs) cargo run
```