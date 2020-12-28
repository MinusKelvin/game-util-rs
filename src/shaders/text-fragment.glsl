#version 300 es
precision lowp float;

in vec2 texcoord;
in vec4 col;

uniform sampler2D text;

out vec4 color;

void main() {
    color = col;
    color.a *= texture(text, texcoord).r;
}