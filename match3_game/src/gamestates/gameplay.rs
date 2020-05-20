use crate::Float2;
use crate::Float4;
use crate::ScreenSpaceQuadData;
use super::UpdateBehaviourDesc;
use super::GameStateTransitionState;
use super::GameStateType;
use super::super::HeapAlloc;

use graphics_device::begin_render_pass_and_clear;
use graphics_device::bind_constant;
use graphics_device::bind_pso;
use graphics_device::create_pso;
use graphics_device::draw_vertices;
use graphics_device::GraphicsCommandList;
use graphics_device::GraphicsDeviceLayer;
use graphics_device::LinearAllocatorState;
use graphics_device::PipelineStateObject;
use graphics_device::PipelineStateObjectDesc;
use graphics_device::RenderTargetView;
use os_window::WindowMessages;

pub struct GameplayStateStaticData<'a> {
    screen_space_quad_opaque_pso: PipelineStateObject<'a>,
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

        GameplayStateStaticData {
            screen_space_quad_opaque_pso,
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
    pub fn new<'a>() -> GameplayStateFrameData {
        GameplayStateFrameData {
            grid: { [[false; 5]; 6] },
            rnd_state: Xoroshiro128Rng {
                state: [23480923840238, 459],
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
    messages: &Vec<WindowMessages>,
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

    if selected_fields == 5 {
        if count_selected_fields(&prev_frame_data.grid) != 5 {
            return UpdateBehaviourDesc {
                transition_state: GameStateTransitionState::TransitionToNewState(
                    GameStateType::Pause,
                ),
                block_input: false,
            };
        }
    }

    if selected_fields == 10 {
        if count_selected_fields(&prev_frame_data.grid) != 10 {
            return UpdateBehaviourDesc {
                transition_state: GameStateTransitionState::ReturnToPreviousState,
                block_input: false,
            };
        }
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

            draw_vertices(command_list, 4);
        }
    }
}
