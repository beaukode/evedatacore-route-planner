# EVE Data Core Route Planner

A high-performance route planning tool for EVE Frontier, built in Rust. This tool helps calculate optimal paths through star systems using A\* search algorithm.

## Prepare the data

1. Extract the star map from game files using https://github.com/frontier-reapers/frontier-static-data
2. Run `cargo run --release -- build -s data/starmap.json -o data/starmap.bin` to generate the star map binary file.

By default, the maximum jump distance is 200. Run `cargo run --release -- build --help` to show all available options.

## Calculate a path

You need to provide the start and end system IDs. Refer to [EVE Datacore](https://evedataco.re/explore/solarsystems) to find the system ID.

`cargo run --release -- path 30001573 30013956`

run `cargo run --release -- path --help` to show options.

## Run as AWS Lambda

You can package the binary into a zip file and upload it to AWS Lambda.

```bash
cargo lambda build --release --output-format zip
```

The binary star map will not be included in the zip file. You need to add it manually or use a layer.
Then set the `STARMAP_PATH` environment variable to the path of the star map binary file.

## Credits

This project is a fork of [eftb](https://github.com/shish/eftb), thanks to [shish](https://github.com/shish) for the original implementation.

## License

This project is licensed under the GNU General Public License v3.0 (GPL-3.0) - see the [LICENSE](LICENSE) file for details.

This means you are free to:

- Use this software for any purpose
- Change the software to suit your needs
- Share the software with anyone
- Share the changes you make

Under the following conditions:

- You must disclose source code when you distribute the software
- You must license any derivative work under GPL-3.0
- You must state significant changes made to the software
- You must include the original license and copyright notices

This is a copyleft license, which means any derivative work must be distributed under the same license terms.
