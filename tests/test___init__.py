from contextlib import nullcontext
from math import isclose, sqrt

import pytest
from osm_shortlink import shortlink_decode, shortlink_encode


@pytest.mark.parametrize(
    'input',
    [
        (0, 0, 5),
        (156, 45, 17),
        (1.23456789, 2.34567891, 20),
        (-1.23456789, -2.34567891, 20),
        (119.99999999, 39.99999999, 21),
        (15.545454, 45.454545, 13),
    ],
)
def test_encode_decode(input: tuple[float, float, int]):
    encoded = shortlink_encode(*input)
    decoded = shortlink_decode(encoded)
    assert input[2] == decoded[2]  # zoom must be equal
    distance = sqrt((input[0] - decoded[0]) ** 2 + (input[1] - decoded[1]) ** 2)
    max_distance = 360 / (2 ** (input[2] + 8)) * 0.5 * sqrt(5)
    assert max_distance > distance


def test_encode_wrapping():
    assert shortlink_encode(720, 0, 5) == shortlink_encode(0, 0, 5)


@pytest.mark.parametrize(
    ('lat', 'valid'),
    [
        (-91, False),
        (-90, True),
        (90, True),
        (91, False),
    ],
)
def test_encode_lat(lat, valid):
    with nullcontext() if valid else pytest.raises(ValueError, match='Invalid latitude'):
        shortlink_encode(0, lat, 5)


@pytest.mark.parametrize(
    ('zoom', 'valid'),
    [
        (-1, False),
        (0, True),
        (1, True),
        (22, True),
        (23, False),
    ],
)
def test_encode_invalid_zoom(zoom, valid):
    with nullcontext() if valid else pytest.raises(ValueError, match='Invalid zoom'):
        shortlink_encode(0, 0, zoom)


@pytest.mark.parametrize(
    ('input', 'expected'),
    [
        ('0EEQjE--', (0.0550, 51.5110, 9)),
        ('0OP4tXGMB', (19.57922, 51.87695, 19)),
        ('ecetE--', (-31.113, 64.130, 6)),
    ],
)
def test_decode(input, expected):
    decoded = shortlink_decode(input)
    for a, b in zip(expected, decoded):
        assert isclose(a, b, abs_tol=0.01)


@pytest.mark.parametrize(
    ('new', 'old'),
    [
        ('--~v2juONc', '@v2juONc=-'),
        ('as3I3GpG~-', 'as3I3GpG@='),
        ('D~hV--', 'D@hV--'),
        ('CO0O-~m8-', 'CO0O@m8--'),
    ],
)
def test_decode_deprecated(new, old):
    decoded1 = shortlink_decode(new)
    decoded2 = shortlink_decode(old)
    for a, b in zip(decoded1, decoded2):
        assert isclose(a, b)


def test_decode_non_ascii():
    with pytest.raises(ValueError, match='expected ASCII string'):
        shortlink_decode('D~hV--ą')


def test_decode_too_many_offsets():
    with pytest.raises(ValueError, match='too many offset characters'):
        shortlink_decode('D~hV---')


def test_decode_bad_character():
    with pytest.raises(ValueError, match=r"bad character '\$'"):
        shortlink_decode('D~hV--$')


def test_decode_too_long():
    with pytest.raises(ValueError, match='too long'):
        shortlink_decode('aaaaaaaaaaa')


@pytest.mark.parametrize(
    'input',
    [
        '',
        '--',
        'a',
        'a--',
        'ab',
        'ab--',
    ],
)
def test_decode_too_short(input):
    with pytest.raises(ValueError, match='too short'):
        shortlink_decode(input)


def test_decode_malformed_zoom():
    with pytest.raises(ValueError, match='malformed zoom'):
        shortlink_decode('aaa-')
