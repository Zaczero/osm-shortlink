# osm-shortlink

[![PyPI - Python Version](https://shields.monicz.dev/pypi/pyversions/osm-shortlink)](https://pypi.org/project/osm-shortlink)
[![Liberapay Patrons](https://shields.monicz.dev/liberapay/patrons/Zaczero?logo=liberapay&label=Patrons)](https://liberapay.com/Zaczero/)
[![GitHub Sponsors](https://shields.monicz.dev/github/sponsors/Zaczero?logo=github&label=Sponsors&color=%23db61a2)](https://github.com/sponsors/Zaczero)

Fast and correct OpenStreetMap shortlink encoder and decoder implementation in Rust with Python bindings. Shortlinks allow you to represent a location on the map with a short code.

## Installation

The recommended installation method is through the PyPI package manager. The project is implemented in Rust, offering excellent performance characteristics. Several pre-built binary wheels are available for Linux, macOS, and Windows, with support for both x64 and ARM architectures.

```sh
pip install osm-shortlink
```

## Basic usage

```py
from osm_shortlink import shortlink_encode
shortlink_encode(0.054, 51.510, 9)  # -> '0EEQhq--'
shortlink_encode(19.579, 51.876, 19)  # -> '0OP4tR~rx'
shortlink_encode(0, 0, 23)  # ValueError: Invalid zoom: must be between 0 and 22, got 23

from osm_shortlink import shortlink_decode
shortlink_decode('0EEQhq--')  # -> (0.054, 51.510, 9)
shortlink_decode('0OP4tR~rx')  # -> (19.579, 51.876, 19)
shortlink_decode('X')  # ValueError: Invalid shortlink: too short
```

## Format specification

<https://wiki.openstreetmap.org/wiki/Shortlink>
