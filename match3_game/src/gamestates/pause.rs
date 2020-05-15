use crate::clamp;
use crate::Float2;
use crate::Float4;
use crate::ScreenSpaceQuadData;
use super::UpdateBehaviourDesc;
use graphics_device::begin_render_pass;
use graphics_device::bind_constant;
use graphics_device::bind_pso;
use graphics_device::create_pso;
use graphics_device::draw_vertices;
use graphics_device::GraphicsCommandList;
use graphics_device::GraphicsDeviceLayer;
use graphics_device::LinearAllocatorState;
use graphics_device::MappedGpuData;
use graphics_device::PipelineStateObject;
use graphics_device::PipelineStateObjectDesc;
use graphics_device::RenderTargetView;
use os_window::WindowMessages;

pub struct PauseStateStaticData<'a> {
    screen_space_quad_blended_pso: PipelineStateObject<'a>,
}

impl PauseStateStaticData<'_> {
    pub fn new<'a>(device_layer: &GraphicsDeviceLayer) -> PauseStateStaticData<'a> {
        let screen_space_quad_blended_pso: PipelineStateObject = create_pso(
            &device_layer.device,
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
    pub fn new<'a>() -> PauseStateFrameData {
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
    pub fn new<'a>(device_layer: &GraphicsDeviceLayer) -> PauseState<'a> {
        PauseState {
            static_data: PauseStateStaticData::new(&device_layer),
            frame_data0: PauseStateFrameData::new(),
            frame_data1: PauseStateFrameData::new(),
        }
    }
}

pub fn update_pause_state(
    prev_frame_params: &PauseStateFrameData,
    frame_params: &mut PauseStateFrameData,
    messages: &Vec<WindowMessages>,
    dt: f32,
) -> UpdateBehaviourDesc {
    // fade in the screen state
    frame_params.fade_in_status = clamp(prev_frame_params.fade_in_status + dt, 0.0, 1.0);

    for x in messages.iter() {
        match x {
            WindowMessages::MouseLeftButtonDown => {
                return UpdateBehaviourDesc {
                    transition_state: super::GameStateTransitionState::ReturnToPreviousState,
                    block_input: true,
                }
            }

            _ => {}
        }
    }

    UpdateBehaviourDesc {
        transition_state: super::GameStateTransitionState::Unchanged,
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

    let obj_alloc = super::super::HeapAlloc::new(
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
