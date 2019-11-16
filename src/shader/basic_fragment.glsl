#version 330 core

uniform float red;

in VS_OUT {
    vec3 color;
} IN;

out vec4 frag_color;

void main() {
    frag_color = vec4(red, IN.color.yz, 1.0);
}
