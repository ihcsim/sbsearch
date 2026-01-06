# sbfind

`sbfind` is a [Harvester support bundle][1] tool that search for keywords in the
resource logs and displayed them in chronological order.

![screenshot of the sbfind tui displaying resource logs output](./img/tui.png)

## Usage

To see general usage:

```sh
sbfind -h
```

```sh
Usage: sbfind --support-bundle-path <SUPPORT_BUNDLE_PATH> --resource-name <RESOURCE_NAME>

Options:
  -s, --support-bundle-path <SUPPORT_BUNDLE_PATH>
  -r, --resource-name <RESOURCE_NAME>
  -h, --help                                       Print help
  -V, --version                                    Print version
```

For example, to search for logs relevant to the PVC
`pvc-tg13d9d2-f7g3-46t1-770d-13wa01c36f01` in the support bundle located at
`~/Downloads/supportbundle_5t66d62c-u8a4-4311-8426-1d8493b2b576_2024-10-17T18-38-27Z`:

```sh
sbfind \
  -s ~/Downloads/supportbundle_5t66d62c-u8a4-4311-8426-1d8493b2b576_2024-10-17T18-38-27Z \
  -r pvc-tg13d9d2-f7g3-46t1-770d-13wa01c36f01
```

Unarchive the support bundle before passing its path to `sbfind`.

## Development

To compile the code:

```sh
make check
```

To run unit tests:

```sh
make test
```

To run the program in debug mode:

```sh
make run SUPPORT_BUNDLE_PATH=<path_to_support_bundle> RESOURCE_NAME=<resource_name>
```

To build the release:

```sh
make release
```

## License

See [License](LICENSE).

[1]: https://docs.harvesterhci.io/v1.8/troubleshooting/harvester/#generate-a-support-bundle
