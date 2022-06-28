Texture2D Texture;
SamplerState ss;

float4 PShader(float4 position: SV_POSITION, float2 uv: TEXCOORD, float4 color: COLOR): SV_TARGET
{
    return Texture.Sample(ss, uv);
}