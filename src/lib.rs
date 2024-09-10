use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

// 64 chars to encode 6 bits
const CHARSET: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_~";
static CHARSET_MAP: OnceLock<HashMap<u8, usize>> = OnceLock::new();

const X_SCALE: f64 = ((u32::MAX as f64) + 1.0) / 360.0;
const Y_SCALE: f64 = ((u32::MAX as f64) + 1.0) / 180.0;
const X_SCALE_INV: f64 = 360.0 / (u32::MAX as f64 + 1.0);
const Y_SCALE_INV: f64 = 180.0 / (u32::MAX as f64 + 1.0);

fn charset_map() -> &'static HashMap<u8, usize> {
    CHARSET_MAP.get_or_init(|| {
        let mut map: HashMap<u8, usize> = CHARSET
            .into_iter()
            .enumerate()
            .map(|(i, c)| (c.clone(), i))
            .collect();
        // resolve '@' for backwards compatibility
        map.insert(b'@', map.get(&b'~').unwrap().clone());
        map
    })
}

#[pyfunction]
fn shortlink_encode(lon: f64, lat: f64, zoom: i8) -> PyResult<String> {
    if !(-90.0 <= lat && lat <= 90.0) {
        return Err(PyValueError::new_err(format!(
            "Invalid latitude: must be between -90 and 90, got {lat}"
        )));
    }
    if !(0 <= zoom && zoom <= 22) {
        return Err(PyValueError::new_err(format!(
            "Invalid zoom: must be between 0 and 22, got {zoom}"
        )));
    }

    let x: u32 = ((lon + 180.0) % 360.0 * X_SCALE) as u32;
    let y: u32 = ((lat + 90.0) * Y_SCALE) as u32;

    let mut c: u64 = 0;
    for i in 0_u8..=31 {
        let x_bit = ((x >> i) & 1) as u64;
        let y_bit = ((y >> i) & 1) as u64;
        c |= x_bit << i * 2 + 1;
        c |= y_bit << i * 2;
    }

    let r = (zoom + 8) % 3;
    let d = (zoom + 8) / 3 + (if r > 0 { 1 } else { 0 }); // ceil instead of floor
    let mut result = Vec::with_capacity((d + r) as usize);

    for i in 0..d {
        let digit = ((c >> (58 - 6 * i)) & 0x3F) as usize;
        result.push(CHARSET[digit]);
    }
    for _ in 0..r {
        result.push(b'-');
    }

    Ok(String::from_utf8(result).unwrap())
}

#[pyfunction]
fn shortlink_decode(s: String) -> PyResult<(f64, f64, u8)> {
    if s.len() != s.chars().count() {
        return Err(PyValueError::new_err(
            "Invalid shortlink: expected ASCII string",
        ));
    }

    let charset_map = charset_map();
    let mut x: u32 = 0;
    let mut y: u32 = 0;
    let mut z: u8 = 0;
    let mut z_offset: i8 = 0;

    for c in s.bytes() {
        // check '=' for backwards compatibility
        if c == b'-' || c == b'=' {
            z_offset -= 1;
            if z_offset <= -3 {
                return Err(PyValueError::new_err(
                    "Invalid shortlink: too many offset characters",
                ));
            }
        } else {
            match charset_map.get(&c) {
                Some(t) => {
                    x <<= 3;
                    y <<= 3;
                    z += 3;
                    if z > 32 {
                        return Err(PyValueError::new_err("Invalid shortlink: too long"));
                    }

                    let t = *t as u32;
                    for i in 0_u8..=2 {
                        x |= ((t >> 2 * i + 1) & 1) << i;
                        y |= ((t >> 2 * i + 0) & 1) << i;
                    }
                }
                None => {
                    return Err(PyValueError::new_err(format!(
                        "Invalid shortlink: bad character '{}'",
                        c as char
                    )));
                }
            }
        }
    }

    if z == 0 {
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

#[pymodule]
#[pyo3(name = "_lib")]
fn lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(shortlink_encode, m)?)?;
    m.add_function(wrap_pyfunction!(shortlink_decode, m)?)?;
    Ok(())
}
