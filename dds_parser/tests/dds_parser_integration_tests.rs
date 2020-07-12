use graphics_device::GraphicsDeviceLayer;
use std::io::Read;

#[test]
fn load_and_create_black_4x4_mips_bc1() {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // load the test texture
    let file = std::fs::File::open(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/paintnet/black_4x4_mips_bc1.dds",
    );
    let mut data = Vec::new();
    let _file_read_result = file.unwrap().read_to_end(&mut data);

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
fn load_and_create_black_4x4_bc1() {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // load the test texture
    let file = std::fs::File::open(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/paintnet/black_4x4_bc1.dds",
    );
    let mut data = Vec::new();
    let _file_read_result = file.unwrap().read_to_end(&mut data);

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
fn load_and_create_white_4x4_mips_bc2() {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // load the test texture
    let file = std::fs::File::open(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/paintnet/white_4x4_bc2_mips.dds",
    );
    let mut data = Vec::new();
    let _file_read_result = file.unwrap().read_to_end(&mut data);

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
fn load_and_create_white_4x4_bc2() {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // load the test texture
    let file = std::fs::File::open(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/paintnet/white_4x4_bc2.dds",
    );
    let mut data = Vec::new();
    let _file_read_result = file.unwrap().read_to_end(&mut data);

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
fn load_and_create_black_8x4_bc1() {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // load the test texture
    let file = std::fs::File::open(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/paintnet/black_8x4_bc1.dds",
    );
    let mut data = Vec::new();
    let _file_read_result = file.unwrap().read_to_end(&mut data);

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
fn load_and_create_nvtt_export_white_4x4_mips_bc1_dxt9() {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // load the test texture
    let file = std::fs::File::open(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/nvtt_export/white_4x4_bc1_mips_dxt9.dds",
    );
    let mut data = Vec::new();
    let _file_read_result = file.unwrap().read_to_end(&mut data);

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
fn load_and_create_nvtt_export_white_4x4_mips_bc1_dxt10() {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    // load the test texture
    let file = std::fs::File::open(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/nvtt_export/white_4x4_bc1_mips_dxt10.dds",
    );
    let mut data = Vec::new();
    let _file_read_result = file.unwrap().read_to_end(&mut data);

    // parse the header
    let texture_load_result = dds_parser::parse_dds_header(&data).unwrap();

    let (_texture, _texture_view) = graphics_device::create_texture(
        &graphics_layer.device,
        texture_load_result.desc,
        texture_load_result.subresources_data,
    )
    .unwrap();
}