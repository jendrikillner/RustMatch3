cbuffer ScreenSpaceQuadData : register(b0)
{
	float4 color;
	float2 scale;
	float2 position;
};

struct VertexToPixelShader
{
	float4 position_clip : SV_POSITION;
	float2 uv : TEXCOORD0;
};

Texture2D Texture;
SamplerState Sampler;

VertexToPixelShader VS_main(uint vertex_id: SV_VertexID)
{
    VertexToPixelShader output;

    switch (vertex_id) {
    case 0: output.position_clip = float4(-1,  1, 0, 1); break; // top-left
    case 1: output.position_clip = float4( 1,  1, 0, 1); break; // top-right
    case 2: output.position_clip = float4(-1, -1, 0, 1); break; // bottom-left
    case 3: output.position_clip = float4( 1, -1, 0, 1); break; // bottom-right
    }

	switch (vertex_id) {
	case 0: output.uv = float2(0, 0); break; // top-left
	case 1: output.uv = float2(1, 0); break; // top-right
	case 2: output.uv = float2(0, 1); break; // bottom-left
	case 3: output.uv = float2(1, 1); break; // bottom-right
	}

	output.position_clip.xy *= scale;
	output.position_clip.xy += position;

    return output;
}

float4 PS_main(VertexToPixelShader input) : SV_TARGET
{
	return Texture.Sample(Sampler, input.uv) * color;
}