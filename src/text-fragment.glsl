#version 330 core

in vec2 texcoord;
in vec4 col;

uniform sampler2D tex;

out vec4 color;

void main() {
    color = col;
    color.a *= texture(tex, texcoord).r;
}