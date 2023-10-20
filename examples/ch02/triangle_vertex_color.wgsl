// vertex shader
struct Output {
    @builtin(position) Position : vec4f,
    @location(0) vColor : vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) VertexIndex: u32) -> Output {
    var pos = array<vec2f, 3>(
        vec2(0.0, 0.5),
        vec2(-0.5, -0.5),
        vec2(0.5, -0.5)
    );

    var color = array<vec3f, 3>(
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0)
    );

    var output: Output;
    output.Position = vec4(pos[VertexIndex], 0.0, 1.0);
    output.vColor = vec4(color[VertexIndex], 1.0);
    return output;
}

// fragment shader
@fragment
fn fs_main(@location(0) vColor: vec4f) -> @location(0) vec4f {
    return vColor;
}
