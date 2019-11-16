#version 330 core

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_color;

out VS_OUT {
    vec3 color;
} OUT;

void main() {
    gl_Position = vec4(vertex_position, 1.0);
    OUT.color = vertex_color;
}
