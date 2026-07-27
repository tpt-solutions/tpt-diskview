use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    // Create a proper ICO file with BMP format (required by Windows RC.exe)
    // ICO format:
    // - 6 byte header
    // - 16 byte directory entry per image
    // - Image data (BMP format with AND mask)
    
    let width = 32u16;
    let height = 32u16;
    let bit_count = 32u16;
    let planes = 1u16;
    
    // BMP header size
    let bmp_header_size = 40u32;
    // Image size (32 * 32 * 4 bytes per pixel)
    let image_size = (width as u32) * (height as u32) * 4;
    // AND mask size (32 * 32 / 8 = 128 bytes, padded to 4-byte boundary per row = 4 * 32 = 128)
    let and_mask_size = ((width as u32 + 31) / 32) * 4 * (height as u32);
    let total_image_size = image_size + and_mask_size;
    
    // Directory entry offset (6 byte header + 16 byte directory entry = 22)
    let image_offset = 22u32;
    
    let file = File::create("../src-tauri/icons/icon.ico").unwrap();
    let mut writer = BufWriter::new(file);
    
    // ICO Header (6 bytes)
    writer.write_all(&[0x00, 0x00]).unwrap(); // Reserved (0)
    writer.write_all(&[0x01, 0x00]).unwrap(); // Type (1 = ICO)
    writer.write_all(&[0x01, 0x00]).unwrap(); // Count (1 image)
    
    // Directory Entry (16 bytes)
    writer.write_all(&[width as u8]).unwrap();        // Width (32)
    writer.write_all(&[height as u8]).unwrap();       // Height (32)
    writer.write_all(&[0x00]).unwrap();               // Color count (0 = no palette)
    writer.write_all(&[0x00]).unwrap();               // Reserved
    writer.write_all(&planes.to_le_bytes()).unwrap(); // Planes
    writer.write_all(&bit_count.to_le_bytes()).unwrap(); // Bit count
    writer.write_all(&total_image_size.to_le_bytes()).unwrap(); // Size of image data
    writer.write_all(&image_offset.to_le_bytes()).unwrap();     // Offset of image data
    
    // BMP Header (BITMAPINFOHEADER - 40 bytes)
    writer.write_all(&bmp_header_size.to_le_bytes()).unwrap();  // biSize
    writer.write_all(&(width as i32).to_le_bytes()).unwrap();   // biWidth
    writer.write_all(&((height * 2) as i32).to_le_bytes()).unwrap(); // biHeight (2x for ICO with AND mask)
    writer.write_all(&planes.to_le_bytes()).unwrap();           // biPlanes
    writer.write_all(&bit_count.to_le_bytes()).unwrap();        // biBitCount
    writer.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap();       // biCompression (BI_RGB = 0)
    writer.write_all(&image_size.to_le_bytes()).unwrap();       // biSizeImage
    writer.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap();       // biXPelsPerMeter
    writer.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap();       // biYPelsPerMeter
    writer.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap();       // biClrUsed
    writer.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap();       // biClrImportant
    
    // XOR mask - 32x32 32-bit BGRA (top-down)
    for _y in 0..height {
        for _x in 0..width {
            writer.write_all(&[0x16, 0x21, 0x3E, 0xFF]).unwrap(); // Blue color (BGRA)
        }
    }
    
    // AND mask - 32x32 1-bit (4 bytes per row, 32 rows = 128 bytes)
    // All zeros = fully opaque
    for _y in 0..height {
        writer.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap();
    }
    
    writer.flush().unwrap();
    println!("ICO file created successfully in BMP format!");
}