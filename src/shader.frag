#version 450

layout(location=0) out vec4 f_color;

vec2 viewport_size = vec2 (1600, 1200);

layout (location=0) in struct vs_out
{
    vec3 color;
    vec2 position;
} In;

void main() {
    if (dot(In.position, In.position) > 0.05f)
    discard;

    f_color = vec4(In.color, 1.0);
}