use std::fs::File;
use std::io::Write;

fn make_png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut png = Vec::new();

    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8);
    ihdr.push(6);
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_chunk(&mut png, b"IHDR", &ihdr);

    let row_size = 1 + width as usize * 4;
    let raw_size = row_size * height as usize;
    let mut raw_data = vec![0u8; raw_size];

    for y in 0..height as usize {
        raw_data[y * row_size] = 0;
        for x in 0..width as usize {
            let px = y * row_size + 1 + x * 4;
            raw_data[px + 0] = 0x60;
            raw_data[px + 1] = 0x34;
            raw_data[px + 2] = 0x0F;
            raw_data[px + 3] = 0xFF;
        }
    }

    let mut deflated = Vec::new();
    let mut remaining = &raw_data[..];
    loop {
        let block_size = remaining.len().min(65535);
        let is_final = block_size == remaining.len();
        deflated.push(if is_final { 0x01 } else { 0x00 });
        let len = block_size as u16;
        deflated.extend_from_slice(&len.to_le_bytes());
        deflated.extend_from_slice(&(!len).to_le_bytes());
        deflated.extend_from_slice(&remaining[..block_size]);
        remaining = &remaining[block_size..];
        if remaining.is_empty() {
            break;
        }
    }
    let adler = adler32(&raw_data);
    deflated.extend_from_slice(&adler.to_be_bytes());
    write_chunk(&mut png, b"IDAT", &deflated);
    write_chunk(&mut png, b"IEND", &[]);

    png
}

fn write_chunk(data: &mut Vec<u8>, chunk_type: &[u8; 4], chunk_data: &[u8]) {
    let length = chunk_data.len() as u32;
    data.extend_from_slice(&length.to_be_bytes());
    let mut crc_input = Vec::with_capacity(4 + chunk_data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(chunk_data);
    let crc = crc32(&crc_input);
    data.extend_from_slice(chunk_type);
    data.extend_from_slice(chunk_data);
    data.extend_from_slice(&crc.to_be_bytes());
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFFFFFF
}

fn make_rgba_data(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..height {
        for _ in 0..width {
            data.extend_from_slice(&[0x0F, 0x34, 0x60, 0xFF]);
        }
    }
    data
}

fn main() {
    let sizes = [32u32, 64, 128, 256];

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    for &size in &sizes {
        let rgba = make_rgba_data(size, size);
        let img = ico::IconImage::from_rgba_data(size, size, rgba);
        icon_dir.add_entry(ico::IconDirEntry::encode_as_png(&img).unwrap());
    }

    let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../src-tauri/icons");
    std::fs::create_dir_all(out_dir).unwrap();

    let file = File::create(format!("{}/icon.ico", out_dir)).unwrap();
    icon_dir.write(file).unwrap();
    println!("Created icon.ico with {} sizes", sizes.len());

    for &size in &[32, 128, 256] {
        let png_data = make_png_bytes(size, size);
        let name = if size == 256 {
            "icon-256.png"
        } else {
            &format!("{}x{}.png", size, size)
        };
        let mut file = File::create(format!("{}/{}", out_dir, name)).unwrap();
        file.write_all(&png_data).unwrap();
        println!("Created {}", name);
    }

    let png_data = make_png_bytes(256, 256);
    let mut file = File::create(format!("{}/128x128@2x.png", out_dir)).unwrap();
    file.write_all(&png_data).unwrap();
    println!("Created 128x128@2x.png");

    println!("All icons generated successfully!");
}
