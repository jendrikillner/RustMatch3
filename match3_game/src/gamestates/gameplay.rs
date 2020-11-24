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

    item_texture_cookie: Texture<'a>,
    item_texture_diamond: Texture<'a>,
    item_texture_flower: Texture<'a>,
    item_texture_heart: Texture<'a>,
    item_texture_square: Texture<'a>,
    item_texture_round: Texture<'a>,
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
    // the state of the grid
    grid_selection: [[bool; 5]; 6],

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
            state: GameState::WaitingForSelection,
            grid_selection: { [[false; 5]; 6] },
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

fn reset_grid(grid: &mut [[bool; 5]; 6]) {
    for (y, row) in grid.iter_mut().enumerate() {
        for (x, item) in row.iter_mut().enumerate() {
            *item = false;
        }
    }
}

fn find_first_selected_tile_coordinate(grid: &[[bool; 5]; 6]) -> (i32, i32) {
    for (y, row) in grid.iter().enumerate() {
        for (x, _column) in row.iter().enumerate() {
            if grid[y][x] {
                return (x as i32, y as i32);
            }
        }
    }

    panic!("user is expected to call this function with atleast one item selected");
}

fn valid_grid_id(grid_pos: (i32, i32)) -> bool {
    if grid_pos.0 < 0 {
        return false;
    }

    if grid_pos.0 >= 5 {
        return false;
    }

    if grid_pos.1 < 0 {
        return false;
    }

    if grid_pos.1 >= 6 {
        return false;
    }

    return true;
}

fn is_direct_neighbor_selected(grid: &[[bool; 5]; 6], tile_x: i32, tile_y: i32) -> bool {
    let top = (tile_x, tile_y + 1);
    let bottom = (tile_x, tile_y - 1);
    let left = (tile_x - 1, tile_y);
    let right = (tile_x + 1, tile_y);

    if valid_grid_id(top) && grid[top.1 as usize][top.0 as usize] {
        return true;
    }

    if valid_grid_id(bottom) && grid[bottom.1 as usize][bottom.0 as usize] {
        return true;
    }

    if valid_grid_id(left) && grid[left.1 as usize][left.0 as usize] {
        return true;
    }

    if valid_grid_id(right) && grid[right.1 as usize][right.0 as usize] {
        return true;
    }

    return false;
}

fn swap_selected_tiles(grid_items: &mut [[ItemType; 5]; 6], selection_grid: &[[bool; 5]; 6]) {
    let mut selection_count = 0;
    let mut selection1 = (0, 0);
    let mut selection2 = (0, 0);

    assert_eq!(count_selected_fields(selection_grid), 2);

    // first find the two selected items
    for (y, row) in selection_grid.iter().enumerate() {
        for (x, item) in row.iter().enumerate() {
            if *item {
                if selection_count == 0 {
                    selection1 = (x, y);
                    selection_count = selection_count + 1;
                } else {
                    selection2 = (x, y);
                }
            }
        }
    }

    // once we have the two selected tiles, we can swap

    // store the item from selection 1 in a temp variable
    let temp = grid_items[selection1.1][selection1.0];

    // assign the item from selection 2 into the spot of selection 1
    grid_items[selection1.1][selection1.0] = grid_items[selection2.1][selection2.0];

    // and store the old selection1 into the slot of selection 2
    grid_items[selection2.1][selection2.0] = temp;
}

fn try_find_non_empty_group(row: &[ItemType], search_start: usize) -> Option<(i32, i32)> {
    for (start_index, item) in row.iter().skip(search_start).enumerate() {
        let mut match_counter = 1;
        let group_match_type = *item;

        for x in (start_index + 1)..row.len() {
            if row[x] == group_match_type {
                match_counter += 1;
            } else {
                break;
            }
        }

        if match_counter >= 3 && group_match_type != ItemType::None {
            return Some((start_index as i32, match_counter));
        }
    }

    None
}

pub fn update_gameplay_state(
    prev_frame_data: &GameplayStateFrameData,
    frame_data: &mut GameplayStateFrameData,
    messages: &[WindowMessages],
    _dt: f32,
) -> UpdateBehaviourDesc {
    // copy the state of the previous state as starting point
    frame_data.grid_selection = prev_frame_data.grid_selection;
    frame_data.grid_items = prev_frame_data.grid_items;
    frame_data.rnd_state.state = prev_frame_data.rnd_state.state;
    frame_data.state = prev_frame_data.state;
    frame_data.mouse_pos_worldspace_x = prev_frame_data.mouse_pos_worldspace_x;
    frame_data.mouse_pos_worldspace_x = prev_frame_data.mouse_pos_worldspace_x;

    match frame_data.state {
        GameState::WaitingForSelection => {
            for x in messages {
                match x {
                    WindowMessages::MousePositionChanged(pos) => {
                        println!("cursor position changed: x {0}, y {1}", pos.x, pos.y);

                        // calculate which position the user clicked on
                        // mouse position is supplied in windows space
                        // and we need to convert it into game space
                        // window space origin is at the top left going down
                        // game space is is bottom-left going up
                        // so we have to adjust the y coordinate accordingly
                        frame_data.mouse_pos_worldspace_x = pos.x;
                        frame_data.mouse_pos_worldspace_y = 960 - pos.y;
                    }

                    WindowMessages::MouseLeftButtonDown => {
                        let grid_origin_x = 45;
                        let grid_origin_y = 675 - 6 * 90;

                        let cursor_relative_to_grid_x =
                            frame_data.mouse_pos_worldspace_x - grid_origin_x;
                        let cursor_relative_to_grid_y =
                            frame_data.mouse_pos_worldspace_y - grid_origin_y;

                        let tile_id_x = cursor_relative_to_grid_x / 91;
                        let tile_id_y = 6 - (cursor_relative_to_grid_y / 91);

                        if tile_id_x >= 0 && tile_id_x < 5 && tile_id_y >= 0 && tile_id_y < 6 {
                            frame_data.grid_selection[tile_id_y as usize][tile_id_x as usize] =
                                true;
                        }

                        if count_selected_fields(&frame_data.grid_selection) >= 2 {
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

        GameState::ReactToSelection => {
            assert_eq!( count_selected_fields(&frame_data.grid_selection), 2, "when entering the ReactToSelection state it's expected the user selected 2 items, but {} are selected", count_selected_fields(&frame_data.grid_selection) );

            // first, verify if the two selected items are next to each other
            let (tile_x, tile_y) = find_first_selected_tile_coordinate(&frame_data.grid_selection);

            // now check if any of the direct niegbor tiles are selected
            if is_direct_neighbor_selected(&frame_data.grid_selection, tile_x, tile_y) {
                // swap the tile so that the grid represents the state after the requested transition
                swap_selected_tiles(&mut frame_data.grid_items, &frame_data.grid_selection);

                // switch to the next state which is responsible for checking if the user matched 3 items
                frame_data.state = GameState::ValidateGrid;
            } else {
                // the user selected tiles that are not connected for a valid move
                reset_grid(&mut frame_data.grid_selection);
                frame_data.state = GameState::WaitingForSelection;
            }
        }

        GameState::ValidateGrid => {
            // check if there are any 3 matching tiles next to each

            // this grid will be updated as we go along
            // we don't want to change the original grid in one go
            let mut removale_grid = [[false; 5]; 6];

            let mut last_group_end = 0;

            // first check for each row if we have any 3 matching tiles
            for (y, row) in frame_data.grid_items.iter_mut().enumerate() {
                while let Some(group) = try_find_non_empty_group(row, last_group_end) {
                    for x in group.0..(group.0 + group.1) {
                        removale_grid[y][x as usize] = true;
                    }

                    last_group_end = (group.0 + group.1) as usize;
                }
            }

            // now check the coloums, for that we are transposing vectors into rows
            for x in 0..(frame_data.grid_items[0].len()) {
                // build a column from left to right
                let column = [
                    frame_data.grid_items[0][x],
                    frame_data.grid_items[1][x],
                    frame_data.grid_items[2][x],
                    frame_data.grid_items[3][x],
                    frame_data.grid_items[4][x],
                    frame_data.grid_items[5][x],
                ];

                let mut last_group_end = 0;

                while let Some(group) = try_find_non_empty_group(&column, last_group_end) {
                    for y in group.0..(group.0 + group.1) {
                        removale_grid[y as usize][x] = true;
                    }

                    last_group_end = (group.0 + group.1) as usize;
                }
            }

            if count_selected_fields(&removale_grid) < 3 {
                if count_selected_fields(&frame_data.grid_selection) == 2 {
                    // swap the tile back, the user did not match 3 tiles
                    swap_selected_tiles(&mut frame_data.grid_items, &frame_data.grid_selection);

                    reset_grid(&mut frame_data.grid_selection);
                }

                // back to selection state
                frame_data.state = GameState::WaitingForSelection;
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

                // reset the selection before transitioning to the next state
                reset_grid(&mut frame_data.grid_selection);

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
                        // assign the item from selection 2 into the spot of selection 1
                        frame_data.grid_items[y][x] = frame_data.grid_items[y - 1][x];

                        // and store the old selection1 into the slot of selection 2
                        frame_data.grid_items[y - 1][x] = item;
                    }
                }
            }

            // after we have done this there might be new matches been formed
            // reuse the logic of the validate state grid to check this
            frame_data.state = GameState::ValidateGrid;
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
    for (y, row) in frame_params.grid_selection.iter().enumerate() {
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
}
