use graphics_device::GraphicsDeviceLayer;

mod paintnet {
    pub static BLACK_4X4_BC1: &'static [u8; 136] =
        include_bytes!("../tests/data/paintnet/black_4x4_bc1.dds");

	pub static BLACK_4X4_MIPS_BC1: &'static [u8; 152] =
        include_bytes!("../tests/data/paintnet/black_4x4_mips_bc1.dds");

	pub static WHITE_5X4_BC1: &'static [u8; 144] =
        include_bytes!("../tests/data/paintnet/white_5x4_bc1.dds");

	pub static WHITE_4X4_MIPS_BC2: &'static [u8; 176] =
        include_bytes!("../tests/data/paintnet/white_4x4_bc2_mips.dds");

	pub static WHITE_4X4_BC2: &'static [u8; 144] =
        include_bytes!("../tests/data/paintnet/white_4x4_bc2.dds");

	pub static WHITE_4X4_MIPS_BC3: &'static [u8; 176] =
        include_bytes!("../tests/data/paintnet/white_4x4_bc3_mips.dds");

	pub static WHITE_4X4_BC3: &'static [u8; 144] =
        include_bytes!("../tests/data/paintnet/white_4x4_bc3.dds");

	pub static WHITE_8X4_BC1: &'static [u8; 144] =
        include_bytes!("../tests/data/paintnet/black_8x4_bc1.dds");

	pub static WHITE_4X4_RGBA: &'static [u8; 192] =
        include_bytes!("../tests/data/paintnet/white_4x4_rgba8.dds");

	pub static WHITE_5X4_RGBA: &'static [u8; 208] =
        include_bytes!("../tests/data/paintnet/white_5x4_rgba8.dds");
}

mod nvtt_export {
    pub static WHITE_4X4_BC1_MIPS_DXT9: &'static [u8; 152] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc1_mips_dxt9.dds");

	pub static WHITE_4X4_BC1_MIPS_DXT10: &'static [u8; 172] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc1_mips_dxt10.dds");

	pub static WHITE_4X4_BC2_MIPS_DXT9: &'static [u8; 176] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc2_mips_dxt9.dds");

	pub static WHITE_4X4_BC2_MIPS_DXT10: &'static [u8; 196] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc2_mips_dxt10.dds");

	pub static WHITE_4X4_BC3_MIPS_DXT9: &'static [u8; 176] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc3_mips_dxt9.dds");

	pub static WHITE_4X4_BC3_MIPS_DXT10: &'static [u8; 196] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc3_mips_dxt10.dds");

	// BC4 is always stored in DXT10 and cannot be represented as DXT9
	pub static WHITE_4X4_BC4_MIPS_DXT10: &'static [u8; 172] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc4_mips_dxt10.dds");

	// BC5 is always stored in DXT10 and cannot be represented as DXT9
	pub static WHITE_4X4_BC5_MIPS_DXT10: &'static [u8; 196] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc5_mips_dxt10.dds");

	// BC6 is always stored in DXT10 and cannot be represented as DXT9
	pub static WHITE_4X4_BC6_MIPS_DXT10: &'static [u8; 196] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc6_mips_dxt10.dds");

	// BC7 is always stored in DXT10 and cannot be represented as DXT9
	pub static WHITE_4X4_BC7_MIPS_DXT10: &'static [u8; 196] =
        include_bytes!("../tests/data/nvtt_export/white_4x4_bc7_mips_dxt10.dds");
}

fn test_texture_load_and_creation(data: &[u8]) {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // parse the header
    let texture_load_result = dds_parser::parse_dds_header(&data).unwrap();

    let (_texture, _texture_view) = graphics_device::create_texture(
        &graphics_layer.device,
        texture_load_result.desc,
        texture_load_result.subresources_data,
    )
    .unwrap();
}

#[test]
fn load_and_create_black_4x4_mips_bc1() {
    test_texture_load_and_creation(paintnet::BLACK_4X4_MIPS_BC1);
}

#[test]
fn load_and_create_black_4x4_bc1() {
    test_texture_load_and_creation(paintnet::BLACK_4X4_BC1);
}

// validate that we receive an InvalidDimensions err value if we try to load a non-power of two texture
#[test]
#[should_panic(expected = r#"called `Result::unwrap()` on an `Err` value: ImageSizeNotMultipleOf4"#)]
fn load_and_create_black_5x4_bc1() {
    test_texture_load_and_creation(paintnet::WHITE_5X4_BC1);
}

#[test]
fn load_and_create_white_4x4_mips_bc2() {
    test_texture_load_and_creation(paintnet::WHITE_4X4_MIPS_BC2);
}

#[test]
fn load_and_create_white_4x4_bc2() {
    test_texture_load_and_creation(paintnet::WHITE_4X4_BC2);
}

#[test]
fn load_and_create_white_4x4_mips_bc3() {
    test_texture_load_and_creation(paintnet::WHITE_4X4_MIPS_BC3);
}

#[test]
fn load_and_create_white_4x4_bc3() {
    test_texture_load_and_creation(paintnet::WHITE_4X4_BC3);
}

#[test]
fn load_and_create_black_8x4_bc1() {
    test_texture_load_and_creation(paintnet::WHITE_8X4_BC1);
}

#[test]
fn load_and_create_white_4x4_rgba8() {
    test_texture_load_and_creation(paintnet::WHITE_4X4_RGBA);
}

#[test]
#[should_panic(expected = r#"called `Result::unwrap()` on an `Err` value: ImageSizeNotMultipleOf4"#)]
fn load_and_create_white_5x4_rgba8() {
    test_texture_load_and_creation(paintnet::WHITE_5X4_RGBA);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc1_dxt9() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC1_MIPS_DXT9);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc1_dxt10() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC1_MIPS_DXT10);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc2_dxt9() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC2_MIPS_DXT9);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc2_dxt10() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC2_MIPS_DXT10);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc3_dxt9() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC3_MIPS_DXT9);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc3_dxt10() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC3_MIPS_DXT10);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc4_dxt10() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC4_MIPS_DXT10);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc5_dxt10() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC5_MIPS_DXT10);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc6_dxt10() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC6_MIPS_DXT10);
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc7_dxt10() {
    test_texture_load_and_creation(nvtt_export::WHITE_4X4_BC7_MIPS_DXT10);
}
