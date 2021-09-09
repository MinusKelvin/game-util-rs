layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 color;

uniform mat4 proj;

out vec4 col;

void main() {
    gl_Position = proj * vec4(pos, 0.0, 1.0);
    col = color;
}