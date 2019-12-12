:: compile VS shader
..\build_environment\microsoft\fxc\fxc.exe /T vs_4_0 src_data\shaders\screen_space_quad.hlsl /EVS_main /Fo target_data\shaders\screen_space_quad.vsb
..\build_environment\microsoft\fxc\fxc.exe /T ps_4_0 src_data\shaders\screen_space_quad.hlsl /EPS_main /Fo target_data\shaders\screen_space_quad.psb