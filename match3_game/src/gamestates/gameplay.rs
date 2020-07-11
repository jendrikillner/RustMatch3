use super::{GameStateTransitionState, GameStateType, UpdateBehaviourDesc};
use crate::{Float2, Float4, HeapAlloc, ScreenSpaceQuadData};

use graphics_device::*;
use os_window::WindowMessages;
use std::io::Read;

pub struct GameplayStateStaticData<'a> {
    screen_space_quad_opaque_pso: PipelineStateObject<'a>,
	texture: *mut winapi::um::d3d11::ID3D11Texture2D,
	texture_view: *mut winapi::um::d3d11::ID3D11ShaderResourceView ,
	sampler: *mut winapi::um::d3d11::ID3D11SamplerState ,
}

impl GameplayStateStaticData<'_> {
    pub fn new<'a>(device_layer: &GraphicsDeviceLayer) -> GameplayStateStaticData<'a> {
        let screen_space_quad_opaque_pso: PipelineStateObject = create_pso(
            &device_layer.device,
            PipelineStateObjectDesc {
                shader_name: "target_data/shaders/screen_space_quad",
                premultiplied_alpha: false,
            },
        );

		// load the test texture
		let file = std::fs::File::open(
			"C:/jendrik/projects/rustmatch3/dds_parser/tests/data/paintnet/red_4x4_bc1.dds",
		);
		let mut data = Vec::new();
		let file_read_result_ = file.unwrap().read_to_end(&mut data);

		// parse the header
		let texture_load_result = dds_parser::parse_dds_header(&data).unwrap();

		let mut texture: *mut winapi::um::d3d11::ID3D11Texture2D = std::ptr::null_mut();
		let mut texture_view: *mut winapi::um::d3d11::ID3D11ShaderResourceView = std::ptr::null_mut();
		let mut sampler: *mut winapi::um::d3d11::ID3D11SamplerState = std::ptr::null_mut();

		// and create the texture with the loaded information
		unsafe {
			device_layer.device.native.CreateTexture2D(
				&texture_load_result.desc,
				texture_load_result.subresources_data.as_ptr(),
				&mut texture,
			);

			// create a resource view
			device_layer.device.native.CreateShaderResourceView(
				texture as *mut winapi::um::d3d11::ID3D11Resource,
				std::ptr::null_mut(),
				&mut texture_view,
			);

			let sampler_desc = winapi::um::d3d11::D3D11_SAMPLER_DESC {
				Filter: winapi::um::d3d11::D3D11_FILTER_MIN_MAG_MIP_LINEAR,
				AddressU: winapi::um::d3d11::D3D11_TEXTURE_ADDRESS_CLAMP,
				AddressV: winapi::um::d3d11::D3D11_TEXTURE_ADDRESS_CLAMP,
				AddressW: winapi::um::d3d11::D3D11_TEXTURE_ADDRESS_CLAMP,
				MinLOD: 0.0,
				MaxLOD: 32.0,
				MipLODBias: 0.0,
				MaxAnisotropy: 1,
				ComparisonFunc: winapi::um::d3d11::D3D11_COMPARISON_NEVER,
				BorderColor: [1.0,1.0,1.0,1.0],
			};

			// create a sampler
			device_layer.device.native.CreateSamplerState(
				&sampler_desc,
				&mut sampler,
			);
		}

        GameplayStateStaticData {
            screen_space_quad_opaque_pso,
			texture,
			texture_view,
			sampler,
        }
    }
}

pub struct GameplayStateFrameData {
    // the state of the grid
    grid: [[bool; 5]; 6],

    rnd_state: Xoroshiro128Rng,
}

pub struct GameplayState<'a> {
    pub static_data: GameplayStateStaticData<'a>,
    pub frame_data0: GameplayStateFrameData,
    pub frame_data1: GameplayStateFrameData,
}

impl GameplayStateFrameData {
    pub fn new() -> GameplayStateFrameData {
        GameplayStateFrameData {
            grid: { [[false; 5]; 6] },
            rnd_state: Xoroshiro128Rng {
                state: [23_480_923_840_238, 459],
            },
        }
    }
}

impl GameplayState<'_> {
    pub fn new<'a>(device_layer: &GraphicsDeviceLayer) -> GameplayState<'a> {
        GameplayState {
            static_data: GameplayStateStaticData::new(device_layer),
            frame_data0: GameplayStateFrameData::new(),
            frame_data1: GameplayStateFrameData::new(),
        }
    }
}

struct Xoroshiro128Rng {
    state: [u64; 2],
}

fn rnd_next_u64(rnd: &mut Xoroshiro128Rng) -> u64 {
    let s0 = rnd.state[0];
    let mut s1 = rnd.state[1];
    let result = s0.wrapping_add(s1);

    s1 ^= s0;
    rnd.state[0] = s0.rotate_left(24) ^ s1 ^ (s1 << 16);
    rnd.state[1] = s1.rotate_left(37);

    result
}

fn count_selected_fields(grid: &[[bool; 5]; 6]) -> i32 {
    let mut count = 0;

    for (y, row) in grid.iter().enumerate() {
        for (x, _column) in row.iter().enumerate() {
            if grid[y][x] {
                count += 1;
            }
        }
    }

    count
}

pub fn update_gameplay_state(
    prev_frame_data: &GameplayStateFrameData,
    frame_data: &mut GameplayStateFrameData,
    messages: &[WindowMessages],
    _dt: f32,
) -> UpdateBehaviourDesc {
    // copy the state of the previous state as starting point
    frame_data.grid = prev_frame_data.grid;
    frame_data.rnd_state.state = prev_frame_data.rnd_state.state;

    for x in messages {
        match x {
            WindowMessages::MousePositionChanged(pos) => {
                println!("cursor position changed: x {0}, y {1}", pos.x, pos.y);
            }

            WindowMessages::MouseLeftButtonDown => {
                // pick a random slot
                let rnd_row = (rnd_next_u64(&mut frame_data.rnd_state) % 6) as usize;
                let rnd_col = (rnd_next_u64(&mut frame_data.rnd_state) % 5) as usize;

                frame_data.grid[rnd_row][rnd_col] = true;
            }

            WindowMessages::MouseLeftButtonUp => {
                println!("mouse:left up");
            }

            WindowMessages::MouseFocusGained => {
                println!("mouse:focus gained");
            }

            WindowMessages::MouseFocusLost => {
                println!("mouse:focus lost");
            }

            WindowMessages::WindowClosed => {
                panic!();
            } // this should never happen, handled by higher level code
            WindowMessages::WindowCreated(_x) => {
                panic!();
            } // this should never happen
        }
    }

    // count the number of selected fields
    // open the pause after 5
    // and close the game after 10
    let selected_fields = count_selected_fields(&frame_data.grid);

    if selected_fields == 5 && count_selected_fields(&prev_frame_data.grid) != 5 {
        return UpdateBehaviourDesc {
            transition_state: GameStateTransitionState::TransitionToNewState(GameStateType::Pause),
            block_input: false,
        };
    }

    if selected_fields == 10 && count_selected_fields(&prev_frame_data.grid) != 10 {
        return UpdateBehaviourDesc {
            transition_state: GameStateTransitionState::ReturnToPreviousState,
            block_input: false,
        };
    }

    // don't need to switch game states
    UpdateBehaviourDesc {
        transition_state: GameStateTransitionState::Unchanged,
        block_input: false,
    }
}

pub fn draw_gameplay_state(
    static_data: &GameplayStateStaticData,
    frame_params: &GameplayStateFrameData,
    command_list: &mut GraphicsCommandList,
    backbuffer_rtv: &RenderTargetView,
    gpu_heap_data: &super::super::MappedGpuData,
    gpu_heap_state: &mut LinearAllocatorState,
) {
    let color: [f32; 4] = [0.0, 0.2, 0.4, 1.0];

    begin_render_pass_and_clear(command_list, color, backbuffer_rtv);

    bind_pso(command_list, &static_data.screen_space_quad_opaque_pso);

    for (y, row) in frame_params.grid.iter().enumerate() {
        for (x, column) in row.iter().enumerate() {
            let x_offset_in_pixels = (x as f32) * 180.0;
            let y_offset_in_pixels = (y as f32) * 180.0;

            // allocate the constants for this draw call
            let obj_alloc = HeapAlloc::new(
                ScreenSpaceQuadData {
                    color: if !column {
                        Float4 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                            a: 1.0,
                        }
                    } else {
                        Float4 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                            a: 1.0,
                        }
                    },
                    scale: Float2 {
                        x: (90.0 / 540.0),
                        y: (90.0 / 960.0),
                    },
                    position: Float2 {
                        x: (90.0 / 540.0) * -4.0 + x_offset_in_pixels / 540.0,
                        y: (90.0 / 960.0) * 6.0 - y_offset_in_pixels / 960.0,
                    },
                },
                gpu_heap_data,
                gpu_heap_state,
            );

            bind_constant(command_list, 0, &obj_alloc);
			
			unsafe {

				command_list.command_context.as_ref().unwrap().PSSetSamplers( 0, 1, &static_data.sampler );

				command_list.command_context.as_ref().unwrap().PSSetShaderResources( 0, 1, & static_data.texture_view );
			}

            draw_vertices(command_list, 4);
        }
    }
}
