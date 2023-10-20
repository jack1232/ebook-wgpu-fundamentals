struct Output {
    @builtin(position) Position : vec4f,
    @location(0) vColor : vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) VertexIndex: u32) -> Output {
    var pos : array<vec2f, 9> = array<vec2f, 9>(             
        vec2(-0.63,  0.80),
        vec2(-0.65,  0.20),
        vec2(-0.20,  0.60),
        vec2(-0.37, -0.07),
        vec2( 0.05,  0.18),
        vec2(-0.13, -0.40),
        vec2( 0.30, -0.13),
        vec2( 0.13, -0.64),
        vec2( 0.70, -0.30)     
    );

    var color : array<vec3f, 9> = array<vec3f, 9>(             
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),  
    );
    var output: Output;
    output.Position = vec4(pos[VertexIndex], 0.0, 1.0);
    output.vColor = vec4(color[VertexIndex], 1.0);
    return output;
}

@fragment
fn fs_main(@location(0) vColor: vec4f) -> @location(0) vec4f {
    return vColor;
}
