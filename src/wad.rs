pub fn load_playpal_lump(iwad_path: Option<&str>, wad_path: Option<&str>) -> Option<Vec<u8>> {
    for path in [wad_path, iwad_path].into_iter().flatten() {
        if let Ok(wad) = wad::load_wad_file(path) {
            if let Some(data) = wad.by_id(b"PLAYPAL") {
                return Some(data[0..768].to_vec());
            }
        }
    }
    None
}

pub fn load_titlepic_lump(
    iwad_path: Option<&str>,
    wad_path: Option<&str>,
) -> Option<(String, Vec<u8>)> {
    const TITLE_LUMPS: [&str; 3] = ["TITLEPIC", "TITLE", "HTITLE"];
    for path in [wad_path, iwad_path].into_iter().flatten() {
        if let Ok(wad) = wad::load_wad_file(path) {
            for lump in TITLE_LUMPS {
                let entry_id = wad::EntryId::from_str(lump).unwrap();
                if let Some(data) = wad.by_id(entry_id) {
                    return Some((lump.to_string(), data.to_vec()));
                }
            }
        }
    }
    None
}

pub fn get_titlepic_dimensions(data: &[u8]) -> (usize, usize) {
    let width = u16::from_le_bytes([data[0], data[1]]) as usize;
    let height = u16::from_le_bytes([data[2], data[3]]) as usize;

    if width == 0 || height == 0 || data.len() < 4 + width * 4 {
        (320, 200)
    } else {
        (width, height)
    }
}

pub fn decode_titlepic(
    data: &[u8],
    palette: &[u8],
    width: usize,
    height: usize,
) -> Option<Vec<u8>> {
    let mut out = vec![0u8; width * height * 4];
    let mut col_offsets = vec![0u32; width];
    if data.len() < width * 4 {
        return None;
    }
    for i in 0..width {
        col_offsets[i] = u32::from_le_bytes([
            data[i * 4],
            data[i * 4 + 1],
            data[i * 4 + 2],
            data[i * 4 + 3],
        ]);
    }
    for x in 0..width {
        let mut pos = col_offsets[x] as usize;
        loop {
            if pos >= data.len() {
                break;
            }
            let y_start = data[pos] as usize;
            if y_start == 255 {
                break;
            }
            let n_pixels = data[pos + 1] as usize;
            pos += 3;
            for y in y_start..(y_start + n_pixels) {
                let pal_idx = data[pos] as usize;
                let dst = (y * width + x) * 4;
                out[dst + 0] = palette[pal_idx * 3 + 0];
                out[dst + 1] = palette[pal_idx * 3 + 1];
                out[dst + 2] = palette[pal_idx * 3 + 2];
                out[dst + 3] = 16;
                pos += 1;
            }
            pos += 1;
        }
    }
    Some(out)
}

pub fn decode_htitle(data: &[u8], palette: &[u8]) -> Option<Vec<u8>> {
    if data.len() != 320 * 200 {
        return None;
    }
    let mut out = vec![0u8; 320 * 200 * 4];
    for i in 0..(320 * 200) {
        let pal_idx = data[i] as usize;
        out[i * 4 + 0] = palette[pal_idx * 3 + 0];
        out[i * 4 + 1] = palette[pal_idx * 3 + 1];
        out[i * 4 + 2] = palette[pal_idx * 3 + 2];
        out[i * 4 + 3] = 16;
    }
    Some(out)
}
