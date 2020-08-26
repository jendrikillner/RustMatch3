use std::convert::TryInto;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::um::d3d11::*;

#[derive(Debug)]
pub enum DdsParserError {
    InvalidHeader(& 'static str),
    InvalidFlags(& 'static str),
    FormatNotSupported,
    ImageSizeNotMultipleOf4,
}

pub struct ParsedTextureData {
    pub desc: D3D11_TEXTURE2D_DESC,
    pub subresources_data: Vec<D3D11_SUBRESOURCE_DATA>,
}

fn is_multiple_of4(value: u32) -> bool {
    if value % 4 == 0 {
        true
    } else {
        false
    }
}

pub fn parse_dds_header(src_data: &[u8]) -> Result<ParsedTextureData, DdsParserError> {
    // each dds file follows a high level structure
    // DWORD with value "DDS " 0x20534444
    // DDS_HEADER
    // optionally: DDS_HEADER_DXT10
    // BYTES (main surface data)

    // a valid DDS file needs at least 128 bytes to srore the DDS dword and the DDS_HEADER
    // if the file is smaller it cannot be a valid file
    if src_data.len() < 128 {
        return Err(DdsParserError::InvalidHeader("smaller than 128 bytes") );
    }

    let mut file_cursor = 0;

    // DDS files are expected to start with "DDS " = 0x20534444
    // if this is not the case the file is not a valid DDS file
    // try_into could panic if src_data is too short
    // but we checked the data length before
    let dw_magic: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    if dw_magic != 0x2053_4444 {
        return Err(DdsParserError::InvalidHeader("file is missing DDS DWORD at start of the file") );
    }

    // next is the DDS_HEADER
    // this structure is 124 bytes long
    let dds_header_dw_size: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    if dds_header_dw_size != 124 {
        return Err(DdsParserError::InvalidHeader("DDS_HEADER size is not 124 bytes") );
    }

    let dds_header_dw_flags: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    // validate that all the flags that are mandatory are written
    // see following docs for definitions
    // https://web.archive.org/web/20191225214138/https://docs.microsoft.com/en-us/windows/win32/direct3ddds/dds-header

    static DDSD_CAPS: u32 = 0x1;
    static DDSD_HEIGHT: u32 = 0x2;
    static DDSD_WIDTH: u32 = 0x4;
    // static DDSD_PITCH : u32 = 0x8;
    static DDSD_PIXELFORMAT: u32 = 0x1000;
    static DDSD_MIPMAPCOUNT: u32 = 0x20000;
    // static DDSD_LINEARSIZE : u32 = 0x80000;
    // static DDSD_DEPTH : u32 = 0x800000;

    static DDPF_FOURCC: u32 = 0x4;

    if dds_header_dw_flags & DDSD_CAPS == 0 {
        return Err(DdsParserError::InvalidFlags("missing DDSD_CAPS") );
    }

    if dds_header_dw_flags & DDSD_HEIGHT == 0 {
        return Err(DdsParserError::InvalidFlags("missing DDSD_HEIGHT"));
    }

    if dds_header_dw_flags & DDSD_WIDTH == 0 {
        return Err(DdsParserError::InvalidFlags("missing DDSD_WIDTH"));
    }

    if dds_header_dw_flags & DDSD_PIXELFORMAT == 0 {
        return Err(DdsParserError::InvalidFlags("missing DDSD_PIXELFORMAT"));
    }

    let dds_header_dw_height: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let dds_header_dw_width: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_dw_pitch_or_linear_size: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_dw_depth: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let dds_header_dw_mip_map_count: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    // dwReserved1 is 11 DWORDS of unused space in the header
    file_cursor += 4 * 11;

    // following blocks will parse the DDS_PIXELFORMAT
    let dds_header_pixel_format_size: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    // always needs to be 32 bytes
    // otherwise it's an invalid DDS file
    if dds_header_pixel_format_size != 32 {
        return Err(DdsParserError::InvalidHeader("dds header pixel format is not 32 bytes") );
    }

    let dds_header_pixel_format_flags: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let dds_header_pixel_format_fourcc: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_pixel_format_rgb_bit_count: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_pixel_format_r_bit_mask: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_pixel_format_g_bit_mask: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_pixel_format_b_bit_mask: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_pixel_format_a_bit_mask: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    // back to parsing the rest of the DDS_HEADER
    let _dds_header_caps1: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    let _dds_header_caps2: u32 =
        u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
    file_cursor += 4;

    file_cursor += 4; // dwCaps3 is unused
    file_cursor += 4; // dwCaps4 is unused
    file_cursor += 4; // dwReserved2 is unused

    // after we are done parsing the header the cursor should be pointing after the header
    // otherwise there is a bug in the previos parser code
    assert!(file_cursor == 128);

    // decide if we need to parse the DXT10 header too
    let format = if dds_header_pixel_format_fourcc == 0x30315844 {
        // the DXT 10 header is contained in the file

        let dxgi_format: u32 =
            u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
        file_cursor += 4;

        let _resource_dimension: u32 =
            u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
        file_cursor += 4;

        let _misc_flag: u32 =
            u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
        file_cursor += 4;

        let _array_size: u32 =
            u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
        file_cursor += 4;

        let _misc_flags2: u32 =
            u32::from_le_bytes(src_data[file_cursor..(file_cursor + 4)].try_into().unwrap()); // DWORD is 4 bytes long
        file_cursor += 4;

        // don't need to convert these formats
        // they are already in the expected format
        dxgi_format
    } else {
        // compressed texture
        if (dds_header_pixel_format_flags & DDPF_FOURCC) > 0 {
            match dds_header_pixel_format_fourcc {
                0x31545844 => DXGI_FORMAT_BC1_UNORM,
                0x33545844 => DXGI_FORMAT_BC2_UNORM,
                0x35545844 => DXGI_FORMAT_BC3_UNORM,
                _ => {
                    return Err(DdsParserError::FormatNotSupported);
                }
            }
        } else {
            // uncompressed textures
            if _dds_header_pixel_format_rgb_bit_count == 32 {
                if _dds_header_pixel_format_r_bit_mask == 0x00FF0000
                    && _dds_header_pixel_format_g_bit_mask == 0x0000FF00
                    && _dds_header_pixel_format_b_bit_mask == 0x000000FF
                    && _dds_header_pixel_format_a_bit_mask == 0xff000000
                {
                    DXGI_FORMAT_R8G8B8A8_UNORM
                } else {
                    return Err(DdsParserError::FormatNotSupported);
                }
            } else {
                return Err(DdsParserError::FormatNotSupported);
            }
        }
    };

    if format != DXGI_FORMAT_R8G8B8A8_UNORM && (is_multiple_of4(dds_header_dw_width) && is_multiple_of4(dds_header_dw_height)) == false {
        return Err(DdsParserError::ImageSizeNotMultipleOf4);
    }

    let mipmap_count = if dds_header_dw_flags & DDSD_MIPMAPCOUNT > 0 {
        dds_header_dw_mip_map_count
    } else {
        1
    };

    // fill the texture header with the information we parsed
    let texture_header_ref = D3D11_TEXTURE2D_DESC {
        Width: dds_header_dw_width,
        Height: dds_header_dw_height,
        MipLevels: mipmap_count,
        ArraySize: 1, // only supported with DXT10 headers
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE,
        MiscFlags: 0,
        CPUAccessFlags: 0,
    };

    let mut subresources: Vec<D3D11_SUBRESOURCE_DATA> = Vec::new();

    let block_size = match format {
        DXGI_FORMAT_BC1_UNORM | DXGI_FORMAT_BC1_UNORM_SRGB | DXGI_FORMAT_BC1_TYPELESS => 8,
        DXGI_FORMAT_BC2_UNORM | DXGI_FORMAT_BC2_UNORM_SRGB | DXGI_FORMAT_BC2_TYPELESS => 16,
        DXGI_FORMAT_BC3_UNORM | DXGI_FORMAT_BC3_UNORM_SRGB | DXGI_FORMAT_BC3_TYPELESS => 16,
        DXGI_FORMAT_BC4_UNORM | DXGI_FORMAT_BC4_SNORM | DXGI_FORMAT_BC4_TYPELESS => 8,
        DXGI_FORMAT_BC5_UNORM | DXGI_FORMAT_BC5_SNORM | DXGI_FORMAT_BC5_TYPELESS => 16,
        DXGI_FORMAT_BC6H_UF16 | DXGI_FORMAT_BC6H_SF16 | DXGI_FORMAT_BC6H_TYPELESS => 16,
        DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB | DXGI_FORMAT_BC7_TYPELESS => 16,
        DXGI_FORMAT_R8G8B8A8_UNORM => 1,
        _ => {
            return Err(DdsParserError::FormatNotSupported);
        }
    };

    // calculate the subresource data for each mip map

    for mip_level in 0..mipmap_count {
        let mip_level_width = texture_header_ref.Width >> mip_level;
        let mip_level_height = texture_header_ref.Height >> mip_level;

        let (line_pitch, slice_pitch) = if block_size > 1 {
            let line_pitch = std::cmp::max(1, (mip_level_width + 3) / 4) * block_size;
            let slice_pitch = line_pitch * std::cmp::max(1, (mip_level_height + 3) / 4);
            (line_pitch, slice_pitch)
        } else {
            // only uncompressed format supported for now
            assert!(format == DXGI_FORMAT_R8G8B8A8_UNORM);

            let bits_per_pixel = 32;
            let line_pitch = (mip_level_width * bits_per_pixel + 7) / 8;
            let slice_pitch = line_pitch * texture_header_ref.Height;

            (line_pitch, slice_pitch)
        };

        subresources.push(D3D11_SUBRESOURCE_DATA {
            pSysMem: src_data[file_cursor..(file_cursor + (slice_pitch as usize))].as_ptr()
                as *const winapi::ctypes::c_void, // todo, calculate this correctly
            SysMemPitch: line_pitch,
            SysMemSlicePitch: slice_pitch,
        });

        file_cursor += slice_pitch as usize;
    }

    // all data needs to be used, otherwise therer was a problem with parsing
    assert!(file_cursor == src_data.len());

    Ok(ParsedTextureData {
        desc: texture_header_ref,
        subresources_data: subresources,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // embed the data we will be testing against
    mod paintnet {
        pub static BLACK_4X4_BC1: &'static [u8; 136] =
            include_bytes!("../tests/data/paintnet/black_4x4_bc1.dds");
        pub static BLACK_4X4_MIPS_BC1: &'static [u8; 152] =
            include_bytes!("../tests/data/paintnet/black_4x4_mips_bc1.dds");
    }

    fn validate_texture_header(
        texture_header_ref: &D3D11_TEXTURE2D_DESC,
        texture_header: &D3D11_TEXTURE2D_DESC,
    ) {
        assert_eq!(texture_header_ref.Width, texture_header.Width);
        assert_eq!(texture_header_ref.Height, texture_header.Height);
        assert_eq!(texture_header_ref.MipLevels, texture_header.MipLevels);
        assert_eq!(texture_header_ref.ArraySize, texture_header.ArraySize);
        assert_eq!(texture_header_ref.Format, texture_header.Format);
        assert_eq!(
            texture_header_ref.SampleDesc.Count,
            texture_header.SampleDesc.Count
        );
        assert_eq!(
            texture_header_ref.SampleDesc.Quality,
            texture_header.SampleDesc.Quality
        );
        assert_eq!(texture_header_ref.Usage, texture_header.Usage);
        assert_eq!(texture_header_ref.BindFlags, texture_header.BindFlags);
        assert_eq!(texture_header_ref.MiscFlags, texture_header.MiscFlags);
        assert_eq!(
            texture_header_ref.CPUAccessFlags,
            texture_header.CPUAccessFlags
        );
    }

    #[test]
    fn validate_texture_header_black_4x4_bc1() {
        let texture_header_ref = D3D11_TEXTURE2D_DESC {
            Width: 4,
            Height: 4,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_BC1_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            MiscFlags: 0,
            CPUAccessFlags: 0,
        };

        let texture_data_desc = D3D11_SUBRESOURCE_DATA {
            pSysMem: std::ptr::null_mut(), // can't validate this, will be pointing into the original block
            SysMemPitch: 8,                // 4x4 texture = 1 BC1 block = 8 bytes
            SysMemSlicePitch: 8,           // 1 block
        };

        let texture_load_result = parse_dds_header(paintnet::BLACK_4X4_BC1);

        assert_eq!(texture_load_result.is_ok(), true);

        let texture_header = texture_load_result.unwrap();

        validate_texture_header(&texture_header_ref, &texture_header.desc);

        // should contain one subresource
        assert_eq!(texture_header.subresources_data.len(), 1);

        assert_eq!(
            texture_data_desc.SysMemPitch,
            texture_header.subresources_data[0].SysMemPitch
        );
        assert_eq!(
            texture_data_desc.SysMemSlicePitch,
            texture_header.subresources_data[0].SysMemSlicePitch
        );

        // validate that the texture can be created without any debug validation errors
    }

    #[test]
    fn validate_texture_header_black_4x4_mips_bc1() {
        let texture_header_ref = D3D11_TEXTURE2D_DESC {
            Width: 4,
            Height: 4,
            MipLevels: 3,
            ArraySize: 1,
            Format: DXGI_FORMAT_BC1_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE,
            MiscFlags: 0,
            CPUAccessFlags: 0,
        };

        let texture_load_result = parse_dds_header(paintnet::BLACK_4X4_MIPS_BC1);

        assert_eq!(texture_load_result.is_ok(), true);

        let texture_header = texture_load_result.unwrap();

        validate_texture_header(&texture_header_ref, &texture_header.desc);
    }
}
