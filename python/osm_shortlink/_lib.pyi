def shortlink_encode(lon: float, lat: float, zoom: int) -> str:
    """
    Encode a coordinate pair and zoom level into a shortlink code.

    >>> shortlink_encode(19.57922, 51.87695, 19)
    '0OP4tXGGp'
    """

def shortlink_decode(s: str, /) -> tuple[float, float, int]:
    """
    Decode a shortlink code into a coordinate pair and zoom level.

    Returns a tuple of (lon, lat, zoom).

    >>> shortlink_decode('0OP4tXGGp')
    (19.57922, 51.87695, 19)
    """
