layout(location = 0) in vec2 pos;
layout(location = 1) in vec3 tex;
layout(location = 2) in vec4 color;

uniform mat4 proj;

out vec3 texcoord;
out vec4 col;

void main() {
    gl_Position = proj * vec4(pos, 0.0, 1.0);
    texcoord = tex;
    col = color;
}