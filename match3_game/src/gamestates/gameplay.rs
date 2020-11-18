use super::{GameStateTransitionState, GameStateType, UpdateBehaviourDesc};
use crate::GameSpaceQuadData;
use crate::Int2;
use crate::{Float4, HeapAlloc};

use graphics_device::*;
use os_window::WindowMessages;

pub struct GameplayStateStaticData<'a> {
    game_space_quad_opaque_pso: PipelineStateObject<'a>,
    bg_texture: Texture<'a>,
    border_top_texture: Texture<'a>,
    border_bottom_texture: Texture<'a>,
    texture_item_background: Texture<'a>,
}

impl GameplayStateStaticData<'_> {
    pub fn new<'a>(device: &'a GraphicsDevice) -> GameplayStateStaticData<'a> {
        let game_space_quad_opaque_pso: PipelineStateObject = create_pso(
            device,
            PipelineStateObjectDesc {
                shader_name: "target_data/shaders/game_space_quad",
                premultiplied_alpha: true,
            },
        );

        let texture_bg = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/gameplay_background_tall.dds",
            device,
        )
        .unwrap();

        let texture_border_top = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/gameplay_top_border.dds",
            device,
        )
        .unwrap();

        let texture_border_bottom = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/gameplay_bottom_border.dds",
            device,
        )
        .unwrap();

        let texture_item_background = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/gameplay_item_background.dds",
            device,
        )
        .unwrap();

        GameplayStateStaticData {
            game_space_quad_opaque_pso,
            bg_texture: texture_bg,
            border_top_texture: texture_border_top,
            border_bottom_texture: texture_border_bottom,
            texture_item_background,
        }
    }
}

#[derive(Copy, Clone)]
pub enum GameState {
    // the initial state of the gameflow
    // in this mode the game wait for the user to select two tiles
    WaitingForSelection,

    // this triggers animations
    // updates the grid if appropriate
    // update move counter
    // ...
    ReactToSelection,

    // this will check the new grid state, if any tiles are in a position to be removed
    // remove them and add the necessary points
    // if something gets removed -> next state ArrangeTiles
    // nothing gets removed      -> next state WaitingForSelection
    ValidateGrid,

    // after tiles got removed, move the tiles that are left into the new slots
    // spawn additional files as spots get available
    ArrangeTiles,
}

pub struct GameplayStateFrameData {
    // the state of the grid
    grid: [[bool; 5]; 6],

    // random generator
    rnd_state: Xoroshiro128Rng,

    // maybe move this into an enum with the state (grid etc inside?)
    state: GameState,
}

pub struct GameplayState<'a> {
    pub static_data: GameplayStateStaticData<'a>,
    pub frame_data0: GameplayStateFrameData,
    pub frame_data1: GameplayStateFrameData,
}

impl GameplayStateFrameData {
    pub fn new() -> GameplayStateFrameData {
        GameplayStateFrameData {
            state: GameState::WaitingForSelection,
            grid: { [[false; 5]; 6] },
            rnd_state: Xoroshiro128Rng {
                state: [23_480_923_840_221, 459],
            },
        }
    }
}

impl GameplayState<'_> {
    pub fn new<'a>(device: &'a GraphicsDevice) -> GameplayState<'a> {
        GameplayState {
            static_data: GameplayStateStaticData::new(device),
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
    frame_data.state = prev_frame_data.state;

    match frame_data.state {
        GameState::WaitingForSelection => {
            for x in messages {
                match x {
                    WindowMessages::MousePositionChanged(pos) => {
                        println!("cursor position changed: x {0}, y {1}", pos.x, pos.y);
                    }

                    WindowMessages::MouseLeftButtonDown => {
                        // todo: calculate which tile the user clicked on
                        let rnd_row = (rnd_next_u64(&mut frame_data.rnd_state) % 6) as usize;
                        let rnd_col = (rnd_next_u64(&mut frame_data.rnd_state) % 5) as usize;

                        frame_data.grid[rnd_row][rnd_col] = true;

						// count how many items are selected now
						// if 2 are selected we are entering the next state
						if count_selected_fields(&frame_data.grid) >= 2 {
							frame_data.state = GameState::ReactToSelection;
							break;
						}
                    }

                    _ => {
                        // case we don't care
                    }
                }
            }
        }

        GameState::ReactToSelection => {}

        GameState::ArrangeTiles => {}

        GameState::ValidateGrid => {}
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

    bind_pso(command_list, &static_data.game_space_quad_opaque_pso);

    // draw the background
    {
        bind_texture(command_list, 0, &static_data.bg_texture.srv);

        let obj_alloc = HeapAlloc::new(
            GameSpaceQuadData {
                color: Float4 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                    a: 1.0,
                },
                size_pixels: Int2 { x: 540, y: 960 },
                position_bottom_left: Int2 { x: 0, y: 0 },
            },
            gpu_heap_data,
            gpu_heap_state,
        );

        bind_constant(command_list, 0, &obj_alloc);

        draw_vertices(command_list, 4);
    }

    {
        bind_texture(command_list, 0, &static_data.border_top_texture.srv);

        let obj_alloc = HeapAlloc::new(
            GameSpaceQuadData {
                color: Float4 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                    a: 1.0,
                },
                size_pixels: Int2 { x: 540, y: 184 },
                position_bottom_left: Int2 { x: 0, y: 960 - 184 },
            },
            gpu_heap_data,
            gpu_heap_state,
        );

        bind_constant(command_list, 0, &obj_alloc);

        draw_vertices(command_list, 4);
    }

    {
        bind_texture(command_list, 0, &static_data.border_bottom_texture.srv);

        let obj_alloc = HeapAlloc::new(
            GameSpaceQuadData {
                color: Float4 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                    a: 1.0,
                },
                size_pixels: Int2 { x: 540, y: 184 },
                position_bottom_left: Int2 { x: 0, y: 0 },
            },
            gpu_heap_data,
            gpu_heap_state,
        );

        bind_constant(command_list, 0, &obj_alloc);

        draw_vertices(command_list, 4);
    }

    bind_pso(command_list, &static_data.game_space_quad_opaque_pso);
    bind_texture(command_list, 0, &static_data.texture_item_background.srv);

    for (y, row) in frame_params.grid.iter().enumerate() {
        for (x, column) in row.iter().enumerate() {
            let x_offset_in_pixels = (x * 91) as i32;
            let y_offset_in_pixels = (y * 91) as i32;

            // allocate the constants for this draw call
            let obj_alloc = HeapAlloc::new(
                GameSpaceQuadData {
                    color: if !column {
                        Float4 {
                            x: 1.0,
                            y: 1.0,
                            z: 1.0,
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
                    size_pixels: Int2 { x: 90, y: 90 },
                    position_bottom_left: Int2 {
                        x: 45 + x_offset_in_pixels,
                        y: 960 - 330 + 45 - y_offset_in_pixels,
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
