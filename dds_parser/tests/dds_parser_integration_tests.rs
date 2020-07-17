use graphics_device::GraphicsDeviceLayer;
use std::io::Read;

fn test_texture_load_and_creation(file_path: &str) {
    let enable_debug_device = true;
    let graphics_layer: GraphicsDeviceLayer =
        graphics_device::create_device_graphics_layer_headless(enable_debug_device).unwrap();

    let file_path_complete = format!(
        "C:/jendrik/projects/rustmatch3/dds_parser/tests/data/{0}",
        file_path
    );

    // load the test texture
    let file = std::fs::File::open(file_path_complete);
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
fn load_and_create_black_4x4_mips_bc1() {
    test_texture_load_and_creation("paintnet/black_4x4_mips_bc1.dds");
}

#[test]
fn load_and_create_black_4x4_bc1() {
    test_texture_load_and_creation("paintnet/black_4x4_bc1.dds");
}

#[test]
fn load_and_create_white_4x4_mips_bc2() {
    test_texture_load_and_creation("paintnet/white_4x4_bc2_mips.dds");
}

#[test]
fn load_and_create_white_4x4_bc2() {
    test_texture_load_and_creation("paintnet/white_4x4_bc2.dds");
}

#[test]
fn load_and_create_white_4x4_mips_bc3() {
    test_texture_load_and_creation("paintnet/white_4x4_bc3_mips.dds");
}

#[test]
fn load_and_create_white_4x4_bc3() {
    test_texture_load_and_creation("paintnet/white_4x4_bc3.dds");
}

#[test]
fn load_and_create_black_8x4_bc1() {
    test_texture_load_and_creation("paintnet/black_8x4_bc1.dds");
}

#[test]
fn load_and_create_white_4x4_rgba8() {
    test_texture_load_and_creation("paintnet/white_4x4_rgba8.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc1_dxt9() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc1_mips_dxt9.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc1_dxt10() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc1_mips_dxt10.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc2_dxt9() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc2_mips_dxt9.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc2_dxt10() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc2_mips_dxt10.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc3_dxt9() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc3_mips_dxt9.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc3_dxt10() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc3_mips_dxt10.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc4_dxt9() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc4_mips_dxt9.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc4_dxt10() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc4_mips_dxt10.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc5_dxt9() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc5_mips_dxt9.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc5_dxt10() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc5_mips_dxt10.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc6_dxt9() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc6_mips_dxt9.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc6_dxt10() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc6_mips_dxt10.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc7_dxt9() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc7_mips_dxt9.dds");
}

#[test]
fn load_and_create_nvtt_export_white_4x4_mips_bc7_dxt10() {
    test_texture_load_and_creation("nvtt_export/white_4x4_bc7_mips_dxt10.dds");
}
