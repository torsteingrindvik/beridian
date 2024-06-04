# Data

The `shpank/data` folder contains data from [geofabrik](https://download.geofabrik.de/europe/norway.html).

To make tests run smoothly the `.shp` files which are >100MB are moved to `shpank/data/big`.
The integration tests then skip them, but we can still run them manually.

# Benchmarking

## File in BufReader vs. reading entire file first
```sh
hyperfine -w3 "./target/release/parse --shp shpank/data/gis_osm_natural_a_free_1.shp --mode bytes" "./target/release/parse
--shp shpank/data/gis_osm_natural_a_free_1.shp --mode file"
```

# TODO

## Performance

The >GB sized files can take up to 10s to parse.
If we had a strategy to multi-thread this could likely be a lot faster.

If we find parsing the `.shx` files is fast, we could try to allow interpreting the contents
as jobs.

It might be a good strategy:

- `gis_osm_water_a_free_1.shp` is 1.3 GiB, while
- `gis_osm_water_a_free_1.shx` is 14.2 MiB

We could then benchmark doing `.shx` -> jobs -> N threads (N from CLI) -> parse.

Each job would only need:

- Knowing which byte offset it should start parsing records from
- Knowing how many bytes it should read, which is all the bytes until the next job's starting offset

If we use an ordered rayon flat map sort of approach we might end up in a good place.

## Tracing

Let's add some tracing integration so we can see how long various parts of code takes.

## Dbf

Showing results in `borld` won't be fun unless we parse this data, which tells us that a shape is supposed to be a bus stop or a tree or a road, and things like their names.
