use super::{GameStateTransitionState, UpdateBehaviourDesc};
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

    item_texture_cookie: Texture<'a>,
    item_texture_diamond: Texture<'a>,
    item_texture_flower: Texture<'a>,
    item_texture_heart: Texture<'a>,
    item_texture_square: Texture<'a>,
    item_texture_round: Texture<'a>,
    item_texture_selection: Texture<'a>,
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

        let texture_white =
            load_dds_from_file("target_data/textures/engine/white.dds", device).unwrap();

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

        let item_texture_cookie = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/cookie_base.dds",
            device,
        )
        .unwrap();

        let item_texture_diamond = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/diamond_base.dds",
            device,
        )
        .unwrap();

        let item_texture_flower = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/flower_base.dds",
            device,
        )
        .unwrap();

        let item_texture_heart = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/heart_base.dds",
            device,
        )
        .unwrap();

        let item_texture_square = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/square_base.dds",
            device,
        )
        .unwrap();

        let item_texture_round = load_dds_from_file(
            "target_data/textures/KawaiiCookieAssetPack/round_base.dds",
            device,
        )
        .unwrap();

        GameplayStateStaticData {
            game_space_quad_opaque_pso,
            bg_texture: texture_bg,
            border_top_texture: texture_border_top,
            border_bottom_texture: texture_border_bottom,
            texture_item_background,
            item_texture_cookie,
            item_texture_diamond,
            item_texture_flower,
            item_texture_heart,
            item_texture_square,
            item_texture_round,
            item_texture_selection: texture_white,
        }
    }
}

#[derive(Copy, Clone)]
struct WaitingForSelection2Data {
    selected_tile1: Int2,
}

#[derive(Copy, Clone)]
struct SwapSelectedTilesData {
    selected_tile1: Int2,
    selected_tile2: Int2,
}

#[derive(Copy, Clone)]
enum GameState {
    // the initial state of the gameflow
    // in this mode the game wait for the user to select the first tile
    WaitingForSelection1,

    // waiting for the user to select a second tile
    WaitingForSelection2(WaitingForSelection2Data),

    // user selected 2 tiles
    // OnEnter: swap tiles
    //	if 3+ matches are found -> RemoveMatchedTies
    //  if not
    //    swap tiles back
    //                          -> WaitingForSelection1
    SwapSelectedTiles(SwapSelectedTilesData),

    // this will check the new grid state, if any tiles are in a position to be removed
    // remove them and add the necessary points
    // if something gets removed -> next state ArrangeTiles
    // nothing gets removed      -> next state WaitingForSelection
    ValidateGrid,

    // after tiles got removed, move the tiles that are left into the new slots
    // spawn additional files as spots get available
    ArrangeTiles,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ItemType {
    None,
    Cookie,
    Diamond,
    Flower,
    Heart,
    Round,
    Square,
}

pub struct GameplayStateFrameData {
    grid_items: [[ItemType; 5]; 6],

    mouse_pos_worldspace_x: i32,
    mouse_pos_worldspace_y: i32,

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

fn gen_random_item(random_generator: &mut Xoroshiro128Rng) -> ItemType {
    match rnd_next_u64(random_generator) % 6 {
        0 => ItemType::Cookie,
        1 => ItemType::Diamond,
        2 => ItemType::Flower,
        3 => ItemType::Heart,
        4 => ItemType::Round,
        5 => ItemType::Square,

        _ => {
            panic!("this cannot really happen, % 4 can only return values from 0-3");
        }
    }
}

fn generate_random_layout(random_generator: &mut Xoroshiro128Rng) -> [[ItemType; 5]; 6] {
    let mut arr = { [[ItemType::Cookie; 5]; 6] };

    for row in arr.iter_mut() {
        for x in row.iter_mut() {
            *x = gen_random_item(random_generator);
        }
    }

    arr
}

impl GameplayStateFrameData {
    pub fn new() -> GameplayStateFrameData {
        let mut rnd_generator = Xoroshiro128Rng {
            state: [23_480_923_840_221, 459],
        };

        GameplayStateFrameData {
            state: GameState::WaitingForSelection1,
            grid_items: generate_random_layout(&mut rnd_generator),
            rnd_state: rnd_generator,
            mouse_pos_worldspace_x: 0,
            mouse_pos_worldspace_y: 0,
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

fn swap_tiles(grid_items: &mut [[ItemType; 5]; 6], tile1: Int2, tile2: Int2) {
    assert!(tile1 != tile2);

    // store the item from selection 1 in a temp variable
    let temp = grid_items[tile1.y as usize][tile1.x as usize];

    // assign the item from selection 2 into the spot of selection 1
    grid_items[tile1.y as usize][tile1.x as usize] = grid_items[tile2.y as usize][tile2.x as usize];

    // and store the old selection1 into the slot of selection 2
    grid_items[tile2.y as usize][tile2.x as usize] = temp;
}

fn convert_window_space_to_world_space(window_space_pos: Int2) -> Int2 {
    // calculate which position the user clicked on
    // mouse position is supplied in windows space
    // and we need to convert it into game space
    // window space origin is at the top left going down
    // game space is is bottom-left going up
    // so we have to adjust the y coordinate accordingly
    Int2 {
        x: window_space_pos.x,
        y: 960 - window_space_pos.y,
    }
}

fn calculate_latest_mouse_position(
    messages: &[WindowMessages],
    prev_mouse_pos_world_space: Int2,
) -> Int2 {
    let mut mouse_pos_worldspace = prev_mouse_pos_world_space;

    for x in messages {
        if let WindowMessages::MousePositionChanged(pos) = x {
            mouse_pos_worldspace = convert_window_space_to_world_space(Int2 { x: pos.x, y: pos.y });
        }
    }

    mouse_pos_worldspace
}

fn check_for_mouse_select_field(
    messages: &[WindowMessages],
    prev_mouse_position_worldspace: Int2,
) -> Option<Int2> {
    let mut mouse_pos_worldspace = prev_mouse_position_worldspace;

    for x in messages {
        match x {
            WindowMessages::MousePositionChanged(pos) => {
                mouse_pos_worldspace =
                    convert_window_space_to_world_space(Int2 { x: pos.x, y: pos.y });
            }

            WindowMessages::MouseLeftButtonDown => {
                let grid_origin_x = 45;
                let grid_origin_y = 675 - 6 * 90;

                let cursor_relative_to_grid_x = mouse_pos_worldspace.x - grid_origin_x;
                let cursor_relative_to_grid_y = mouse_pos_worldspace.y - grid_origin_y;

                let tile_id_x = cursor_relative_to_grid_x / 91;
                let tile_id_y = 6 - (cursor_relative_to_grid_y / 91);

                if tile_id_x >= 0 && tile_id_x < 5 && tile_id_y >= 0 && tile_id_y < 6 {
                    return Some(Int2 {
                        x: tile_id_x,
                        y: tile_id_y,
                    });
                }
            }

            _ => {
                // case we don't care
            }
        }
    }

    None
}

fn find_connected_item_in_row(item_row: &[ItemType], match_row: &mut [bool]) {
    let mut start_index = 0;

    while start_index < item_row.len() {
        let mut match_counter = 1;
        let group_match_type = item_row[start_index];

        if group_match_type == ItemType::None {
            continue;
        }

        for item in item_row.iter().skip(start_index + 1) {
            if *item == group_match_type {
                match_counter += 1;
            } else {
                break;
            }
        }

        if match_counter >= 3 {
            for item in match_row.iter_mut().skip(start_index).take(match_counter) {
                *item = true;
            }

            start_index += match_counter;
        } else {
            start_index += 1;
        }
    }
}

fn find_connected_groups(grid_items: [[ItemType; 5]; 6]) -> [[bool; 5]; 6] {
    // this grid will be updated as we go along
    // we don't want to change the original grid in one go
    let mut removale_grid = [[false; 5]; 6];

    // first check for each row if we have any 3 matching tiles
    for y in 0..6 {
        let mut matched_items = [false; 5];
        find_connected_item_in_row(&grid_items[y], &mut matched_items);

        for (x, matched_item) in matched_items.iter().enumerate() {
            removale_grid[y][x] |= *matched_item;
        }
    }

    for x in 0..(grid_items[0].len()) {
        // build a column from left to right
        let column = [
            grid_items[0][x],
            grid_items[1][x],
            grid_items[2][x],
            grid_items[3][x],
            grid_items[4][x],
            grid_items[5][x],
        ];

        let mut matched_items = [false; 6];
        find_connected_item_in_row(&column, &mut matched_items);

        // or the returned state values to make sure only matched items are overwritting the previous state where it hasn't been set yet
        removale_grid[0][x] |= matched_items[0];
        removale_grid[1][x] |= matched_items[1];
        removale_grid[2][x] |= matched_items[2];
        removale_grid[3][x] |= matched_items[3];
        removale_grid[4][x] |= matched_items[4];
        removale_grid[5][x] |= matched_items[5];
    }

    removale_grid
}

pub fn update_gameplay_state(
    prev_frame_data: &GameplayStateFrameData,
    frame_data: &mut GameplayStateFrameData,
    messages: &[WindowMessages],
    _dt: f32,
) -> UpdateBehaviourDesc {
    // copy the state of the previous state as starting point
    frame_data.grid_items = prev_frame_data.grid_items;
    frame_data.rnd_state.state = prev_frame_data.rnd_state.state;
    frame_data.state = prev_frame_data.state;

    // update the mouse position for all states

    match frame_data.state {
        GameState::WaitingForSelection1 => {
            let new_selected_field: Option<Int2> = check_for_mouse_select_field(
                messages,
                Int2 {
                    x: prev_frame_data.mouse_pos_worldspace_x,
                    y: prev_frame_data.mouse_pos_worldspace_y,
                },
            );

            if let Some(selected_field_id) = new_selected_field {
                // the user selected a tile
                // move to the next state
                frame_data.state = GameState::WaitingForSelection2(WaitingForSelection2Data {
                    selected_tile1: selected_field_id,
                });
            }
        }

        GameState::WaitingForSelection2(state) => {
            let new_selected_field: Option<Int2> = check_for_mouse_select_field(
                messages,
                Int2 {
                    x: prev_frame_data.mouse_pos_worldspace_x,
                    y: prev_frame_data.mouse_pos_worldspace_y,
                },
            );

            if let Some(selected_field_id) = new_selected_field {
                // if the same tile is clicked again, cancel the selection
                if selected_field_id == state.selected_tile1 {
                    frame_data.state = GameState::WaitingForSelection1;
                } else {
                    // user selected a second field
                    // are the next to each other?
                    let diff_x = i32::abs(selected_field_id.x - state.selected_tile1.x);
                    let diff_y = i32::abs(selected_field_id.y - state.selected_tile1.y);

                    if diff_x + diff_y == 1 {
                        // user selected a fiel that has a single connection to the first tile
                        frame_data.state = GameState::SwapSelectedTiles(SwapSelectedTilesData {
                            selected_tile1: state.selected_tile1,
                            selected_tile2: selected_field_id,
                        });
                    }
                }
            }
        }

        GameState::SwapSelectedTiles(state) => {
            // swap the tiles first
            swap_tiles(
                &mut frame_data.grid_items,
                state.selected_tile1,
                state.selected_tile2,
            );

            // now validate the grid after swapping
            let removale_grid = find_connected_groups(frame_data.grid_items);

            if count_selected_fields(&removale_grid) < 3 {
                // no 3 connected tiles selected
                // swap back the tiles tiles and restore back to selection
                swap_tiles(
                    &mut frame_data.grid_items,
                    state.selected_tile1,
                    state.selected_tile2,
                );

                frame_data.state = GameState::WaitingForSelection1;
            } else {
                frame_data.state = GameState::ValidateGrid;
            }
        }

        GameState::ValidateGrid => {
            // check if there are any 3 matching tiles next to each

            let removale_grid = find_connected_groups(frame_data.grid_items);

            if count_selected_fields(&removale_grid) < 3 {
                // back to selection state
                frame_data.state = GameState::WaitingForSelection1;
            } else {
                // now all slots that we want to remove items from have been marked inside of removale_grid
                // so now actually empty the item grid
                for (y, row) in removale_grid.iter().enumerate() {
                    for (x, item) in row.iter().enumerate() {
                        if *item {
                            frame_data.grid_items[y][x] = ItemType::None;
                        }
                    }
                }

                frame_data.state = GameState::ArrangeTiles;
            }
        }

        GameState::ArrangeTiles => {
            // once the user matched a few tiles they wil lbe removed from the board
            // this will create holes, we will them by moving existing pieces into the slots
            // for this we start from the bottom of the grid and move them down

            for y in (1..frame_data.grid_items.len()).rev() {
                for x in 0..frame_data.grid_items[y].len() {
                    let item = frame_data.grid_items[y][x];

                    if item == ItemType::None {
                        swap_tiles(
                            &mut frame_data.grid_items,
                            Int2 {
                                x: x as i32,
                                y: y as i32,
                            },
                            Int2 {
                                x: x as i32,
                                y: (y - 1) as i32,
                            },
                        );
                    }
                }
            }

            // now run over the grid one more time and fill the empty spots with new entries
            for y in 0..frame_data.grid_items.len() {
                for x in 0..frame_data.grid_items[y].len() {
                    if frame_data.grid_items[y][x] == ItemType::None {
                        frame_data.grid_items[y][x] = gen_random_item(&mut frame_data.rnd_state);
                    }
                }
            }

            // after we have done this there might be new matches been formed
            // reuse the logic of the validate state grid to check this
            frame_data.state = GameState::ValidateGrid;
        }
    }

    // update mouse position for all states
    frame_data.mouse_pos_worldspace_x = calculate_latest_mouse_position(
        messages,
        Int2 {
            x: prev_frame_data.mouse_pos_worldspace_x,
            y: prev_frame_data.mouse_pos_worldspace_y,
        },
    )
    .x;
    frame_data.mouse_pos_worldspace_y = calculate_latest_mouse_position(
        messages,
        Int2 {
            x: prev_frame_data.mouse_pos_worldspace_x,
            y: prev_frame_data.mouse_pos_worldspace_y,
        },
    )
    .y;

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

    // draw the grid
    for (y, row) in frame_params.grid_items.iter().enumerate() {
        for (x, _column) in row.iter().enumerate() {
            let x_offset_in_pixels = (x * 91) as i32;
            let y_offset_in_pixels = (y * 91) as i32;

            // allocate the constants for this draw call
            let obj_alloc = HeapAlloc::new(
                GameSpaceQuadData {
                    color: Float4 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        a: 1.0,
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

    // draw the items
    for (y, row) in frame_params.grid_items.iter().enumerate() {
        for (x, item) in row.iter().enumerate() {
            let x_offset_in_pixels = (x * 91) as i32;
            let y_offset_in_pixels = (y * 91) as i32;

            // bind the correct texture based on the item

            let texture = match item {
                ItemType::Cookie => &static_data.item_texture_cookie,
                ItemType::Diamond => &static_data.item_texture_diamond,
                ItemType::Flower => &static_data.item_texture_flower,
                ItemType::Heart => &static_data.item_texture_heart,
                ItemType::Square => &static_data.item_texture_square,
                ItemType::Round => &static_data.item_texture_round,
                ItemType::None => {
                    continue;
                }
            };

            let item_size_x = texture.width;
            let item_size_y = texture.height;

            // divide by 2 since we want the items to be centered with the same amount of pixels on each side
            let x_offset_grid = (91 - item_size_x) / 2;
            let y_offset_grid = (91 - item_size_y) / 2;

            bind_texture(command_list, 0, &texture.srv);

            // allocate the constants for this draw call
            let obj_alloc = HeapAlloc::new(
                GameSpaceQuadData {
                    color: Float4 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                        a: 1.0,
                    },
                    size_pixels: Int2 {
                        x: item_size_x,
                        y: item_size_y,
                    },
                    position_bottom_left: Int2 {
                        x: 45 + x_offset_grid + x_offset_in_pixels,
                        y: 960 + y_offset_grid - 330 + 45 - y_offset_in_pixels,
                    },
                },
                gpu_heap_data,
                gpu_heap_state,
            );

            bind_constant(command_list, 0, &obj_alloc);

            draw_vertices(command_list, 4);
        }
    }

    bind_pso(command_list, &static_data.game_space_quad_opaque_pso);
    bind_texture(command_list, 0, &static_data.item_texture_selection.srv);

    // draw selection overlays
    if let GameState::WaitingForSelection2(selection_data) = frame_params.state {
        let x_offset_in_pixels = (selection_data.selected_tile1.x * 91) as i32;
        let y_offset_in_pixels = (selection_data.selected_tile1.y * 91) as i32;

        // allocate the constants for this draw call
        let obj_alloc = HeapAlloc::new(
            GameSpaceQuadData {
                color: Float4 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                    a: 0.5,
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
