#version 100

uniform highp mat4 transform;

attribute highp vec2 pos;
varying highp vec2 texcoord;

void main() {
    gl_Position = transform * vec4(pos, 0, 1);
    texcoord = vec2(pos.x/2.0 + 0.5, 1.0 - (pos.y/2.0 + 0.5));
}