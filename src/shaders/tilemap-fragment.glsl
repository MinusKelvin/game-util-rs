#version 300 es
precision highp float;

in vec2 coord;

uniform ivec2 tilemapSize;
uniform usampler2D tilemap;
uniform sampler2DArray tileset;

out vec4 color;

void main() {
    vec2 tilespace = coord - floor(coord);

    ivec2 mapspace = ivec2(coord);
    if (mapspace.x < 0 || mapspace.y < 0 || mapspace.x >= tilemapSize.x || mapspace.y >= tilemapSize.y)
        discard;
    uint tile = texelFetch(tilemap, mapspace, 0).r;

    color = textureGrad(tileset, vec3(tilespace, float(tile)), dFdx(coord), dFdy(coord));
}
