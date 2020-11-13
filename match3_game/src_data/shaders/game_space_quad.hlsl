cbuffer GameSpaceQuadData : register(b0)
{
	float4 color;
	int2 size_pixels;
	int2 position_bottom_left;
};

struct VertexToPixelShader
{
	float4 position_clip : SV_POSITION;
	float2 uv : TEXCOORD0;
};

Texture2D Texture;
SamplerState Sampler;

float4 TransformWorldToScreen(int2 world_space_pos)
{
	float2 screen_space_pos = float2(
		(world_space_pos.x / 540.0f) * 2 - 1,
		(world_space_pos.y / 960.0f) * 2 - 1);

	return float4(screen_space_pos, 0, 1);
}

VertexToPixelShader VS_main(uint vertex_id: SV_VertexID)
{
    VertexToPixelShader output;

	// calculate the corners of the sprite in pixels
	int2 bottom_left  = int2(position_bottom_left.x                , position_bottom_left.y                );
	int2 bottom_right = int2(position_bottom_left.x + size_pixels.x, position_bottom_left.y                );
	int2 top_left     = int2(position_bottom_left.x                , position_bottom_left.y + size_pixels.y);
	int2 top_right    = int2(position_bottom_left.x + size_pixels.x, position_bottom_left.y + size_pixels.y);

	// project the position into screenspace
	// worldspace in pixels
	// x going right
	// y going up

	// screenspace is -1,1 range
	// x going right
	// y going up

    switch (vertex_id) {
    case 0: output.position_clip = TransformWorldToScreen(top_left); break; // top-left
    case 1: output.position_clip = TransformWorldToScreen(top_right); break; // top-right
    case 2: output.position_clip = TransformWorldToScreen(bottom_left); break; // bottom-left
    case 3: output.position_clip = TransformWorldToScreen(bottom_right); break; // bottom-right
    }

	switch (vertex_id) {
	case 0: output.uv = float2(0, 0); break; // top-left
	case 1: output.uv = float2(1, 0); break; // top-right
	case 2: output.uv = float2(0, 1); break; // bottom-left
	case 3: output.uv = float2(1, 1); break; // bottom-right
	}

    return output;
}

float4 PS_main(VertexToPixelShader input) : SV_TARGET
{
	return Texture.Sample(Sampler, input.uv) * color;
}