#![feature(likely_unlikely)]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use std::hint::unlikely;

// 64 chars to encode 6 bits
const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_~";

const X_SCALE: f64 = ((u32::MAX as f64) + 1.0) / 360.0;
const Y_SCALE: f64 = ((u32::MAX as f64) + 1.0) / 180.0;
const X_SCALE_INV: f64 = 360.0 / (u32::MAX as f64 + 1.0);
const Y_SCALE_INV: f64 = 180.0 / (u32::MAX as f64 + 1.0);

const fn build_charset_lut() -> [i8; 256] {
    let mut lut = [-1i8; 256];
    let mut i = 0;
    while i < CHARSET.len() {
        lut[CHARSET[i] as usize] = i as i8;
        i += 1;
    }
    // resolve '@' for backwards compatibility
    lut[b'@' as usize] = lut[b'~' as usize];
    lut
}

const CHARSET_LUT: [i8; 256] = build_charset_lut();

const fn build_chunks() -> ([u8; 64], [u8; 64]) {
    let mut xs = [0u8; 64];
    let mut ys = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        let t = i as u32;
        xs[i] = (((t >> 1) & 1) | (((t >> 3) & 1) << 1) | (((t >> 5) & 1) << 2)) as u8;
        ys[i] = ((t & 1) | (((t >> 2) & 1) << 1) | (((t >> 4) & 1) << 2)) as u8;
        i += 1;
    }
    (xs, ys)
}

const CHUNKS: ([u8; 64], [u8; 64]) = build_chunks();
const X_CHUNKS: [u8; 64] = CHUNKS.0;
const Y_CHUNKS: [u8; 64] = CHUNKS.1;

#[inline(always)]
fn interleave_bits(x: u32, y: u32) -> u64 {
    #[inline(always)]
    fn part1by1(n: u32) -> u64 {
        let mut x = n as u64;
        x = (x | (x << 16)) & 0x0000_FFFF_0000_FFFF;
        x = (x | (x << 8)) & 0x00FF_00FF_00FF_00FF;
        x = (x | (x << 4)) & 0x0F0F_0F0F_0F0F_0F0F;
        x = (x | (x << 2)) & 0x3333_3333_3333_3333;
        x = (x | (x << 1)) & 0x5555_5555_5555_5555;
        x
    }

    (part1by1(x) << 1) | part1by1(y)
}

#[pyfunction]
fn shortlink_encode(py: Python<'_>, lon: f64, lat: f64, zoom: i8) -> PyResult<Py<PyString>> {
    if unlikely(!(-90.0..=90.0).contains(&lat)) {
        return Err(PyValueError::new_err(format!(
            "Invalid latitude: must be between -90 and 90, got {lat}"
        )));
    }
    if unlikely(!(0..=22).contains(&zoom)) {
        return Err(PyValueError::new_err(format!(
            "Invalid zoom: must be between 0 and 22, got {zoom}"
        )));
    }

    let x: u32 = ((lon + 180.0).rem_euclid(360.0) * X_SCALE) as u32;
    let y: u32 = ((lat + 90.0) * Y_SCALE) as u32;

    let c: u64 = interleave_bits(x, y);

    let n = zoom as u8 + 8;
    let r = n % 3;
    let d = (n + 2) / 3; // ceil((zoom+8)/3)
    let mut buf = [0u8; 12]; // max length for zoom<=22
    let mut len: usize = 0;

    for i in 0..d {
        let digit = ((c >> (58 - 6 * i)) & 0x3F) as usize;
        buf[len] = CHARSET[digit];
        len += 1;
    }
    for _ in 0..r {
        buf[len] = b'-';
        len += 1;
    }

    // Safety: all bytes come from CHARSET or '-', so they are valid UTF-8 ASCII.
    let view = unsafe { std::str::from_utf8_unchecked(&buf[..len]) };
    Ok(PyString::new(py, view).into())
}

#[pyfunction]
fn shortlink_decode(s: &str) -> PyResult<(f64, f64, u8)> {
    if unlikely(!s.is_ascii()) {
        return Err(PyValueError::new_err(
            "Invalid shortlink: expected ASCII string",
        ));
    }

    let mut x: u32 = 0;
    let mut y: u32 = 0;
    let mut z: u8 = 0;
    let mut z_offset: i8 = 0;

    for c in s.bytes() {
        // check '=' for backwards compatibility
        if c == b'-' || c == b'=' {
            z_offset -= 1;
            if unlikely(z_offset <= -3) {
                return Err(PyValueError::new_err(
                    "Invalid shortlink: too many offset characters",
                ));
            }
            continue;
        }

        let t = CHARSET_LUT[c as usize];
        if unlikely(t < 0) {
            return Err(PyValueError::new_err(format!(
                "Invalid shortlink: bad character '{}'",
                c as char
            )));
        }

        x <<= 3;
        y <<= 3;
        z += 3;
        if unlikely(z > 32) {
            return Err(PyValueError::new_err("Invalid shortlink: too long"));
        }

        let t = t as usize;
        x |= X_CHUNKS[t] as u32;
        y |= Y_CHUNKS[t] as u32;
    }

    if unlikely(z == 0) {
        return Err(PyValueError::new_err("Invalid shortlink: too short"));
    }

    x <<= 32 - z;
    y <<= 32 - z;

    let Some(z) = z.checked_sub(8) else {
        return Err(PyValueError::new_err("Invalid shortlink: too short"));
    };
    let Some(z) = z.checked_sub(z_offset.rem_euclid(3) as u8) else {
        return Err(PyValueError::new_err("Invalid shortlink: malformed zoom"));
    };

    Ok((
        (x as f64 * X_SCALE_INV) - 180.0,
        (y as f64 * Y_SCALE_INV) - 90.0,
        z,
    ))
}

#[pymodule(gil_used = false)]
#[pyo3(name = "_lib")]
fn lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(shortlink_encode, m)?)?;
    m.add_function(wrap_pyfunction!(shortlink_decode, m)?)?;
    Ok(())
}
