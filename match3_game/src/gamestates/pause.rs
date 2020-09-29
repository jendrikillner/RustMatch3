use crate::{
    clamp, Float2, Float4, GameStateTransitionState, HeapAlloc, ScreenSpaceQuadData,
    UpdateBehaviourDesc,
};
use graphics_device::*;
use os_window::WindowMessages;

pub struct PauseStateStaticData<'a> {
    screen_space_quad_blended_pso: PipelineStateObject<'a>,
}

impl PauseStateStaticData<'_> {
    pub fn new<'a>(device: &GraphicsDevice) -> PauseStateStaticData<'a> {
        let screen_space_quad_blended_pso: PipelineStateObject = create_pso(
            device,
            PipelineStateObjectDesc {
                shader_name: "target_data/shaders/screen_space_quad",
                premultiplied_alpha: true,
            },
        );

        PauseStateStaticData {
            screen_space_quad_blended_pso,
        }
    }
}

pub struct PauseStateFrameData {
    fade_in_status: f32,
}

impl PauseStateFrameData {
    pub fn new() -> PauseStateFrameData {
        PauseStateFrameData {
            fade_in_status: 0.0,
        }
    }
}

pub struct PauseState<'a> {
    pub static_data: PauseStateStaticData<'a>,
    pub frame_data0: PauseStateFrameData,
    pub frame_data1: PauseStateFrameData,
}

impl PauseState<'_> {
    pub fn new<'a>(device: &GraphicsDevice) -> PauseState<'a> {
        PauseState {
            static_data: PauseStateStaticData::new(device),
            frame_data0: PauseStateFrameData::new(),
            frame_data1: PauseStateFrameData::new(),
        }
    }
}

pub fn update_pause_state(
    prev_frame_params: &PauseStateFrameData,
    frame_params: &mut PauseStateFrameData,
    messages: &[WindowMessages],
    dt: f32,
) -> UpdateBehaviourDesc {
    // fade in the screen state
    frame_params.fade_in_status = clamp(prev_frame_params.fade_in_status + dt, 0.0, 1.0);

    for x in messages.iter() {
        if let WindowMessages::MouseLeftButtonDown = x {
            return UpdateBehaviourDesc {
                transition_state: GameStateTransitionState::ReturnToPreviousState,
                block_input: true,
            };
        }
    }

    UpdateBehaviourDesc {
        transition_state: GameStateTransitionState::Unchanged,
        block_input: true,
    }
}

pub fn draw_pause_state(
    static_state_data: &PauseStateStaticData,
    frame_params: &PauseStateFrameData,
    command_list: &mut GraphicsCommandList,
    backbuffer_rtv: &RenderTargetView,
    gpu_heap_data: &MappedGpuData,
    gpu_heap_state: &mut LinearAllocatorState,
) {
    begin_render_pass(command_list, backbuffer_rtv);

    bind_pso(
        command_list,
        &static_state_data.screen_space_quad_blended_pso,
    );

    let obj_alloc = HeapAlloc::new(
        ScreenSpaceQuadData {
            color: Float4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                a: frame_params.fade_in_status * 0.8,
            },
            scale: Float2 { x: 1.0, y: 1.0 },
            position: Float2 { x: 0.0, y: 0.0 },
        },
        gpu_heap_data,
        gpu_heap_state,
    );

    bind_constant(command_list, 0, &obj_alloc);

    draw_vertices(command_list, 4);
}
