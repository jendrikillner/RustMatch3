struct VertexToPixelShader
{
	float4 position_clip : SV_POSITION;
};

VertexToPixelShader VS_main(uint vertex_id: SV_VertexID)
{
	VertexToPixelShader output;

	switch (vertex_id) {
	case 0: output.position_clip = float4(-1, 1, 0, 1);   break; // top-left
	case 1: output.position_clip = float4 (1, 1, 0, 1);   break; // top-right
	case 2: output.position_clip = float4 (-1, -1, 0, 1); break; // bottom-left
	case 3: output.position_clip = float4 (1, -1, 0, 1);  break; // bottom-right
	}

	output.position_clip.xy *= 0.5f;

	return output;
}

float3 PS_main(VertexToPixelShader input) : SV_TARGET
{
	return float3(1, 1, 0);
}