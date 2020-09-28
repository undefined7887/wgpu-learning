#version 450

layout(location=0) in vec3 position;
layout(location=1) in vec3 color;


layout(location=0) out struct vs_out
{
    vec3 color;
    vec2 position;
} Out;

void main() {
    Out.color = color;
    Out.position = position.xy;
    gl_Position = vec4(position, 1.0);
}
