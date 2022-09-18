use kinc::{
    g4::{
        ClearMode, IndexBuffer, Pipeline, PipelineBuilder, VertexBuffer, VertexBufferDesc,
        VertexStructureBuilder,
    },
    Callbacks, Kinc,
};

struct Shader {
    // vertex_structure: &'static VertexStructure,
    pipeline: Pipeline,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
}

impl Callbacks for Shader {
    fn update(&mut self, kinc: &mut Kinc) {
        let mut g4 = kinc.g4();
        let window = kinc.default_window();
        {
            let mut pass = g4.begin(&window);
            pass.clear(ClearMode::COLOR, 0xFF0FFFFF, 1.0, 1);
            pass.set_vertex_buffer(&self.vertex_buffer);
            pass.set_index_buffer(&self.index_buffer);
            pass.set_pipeline(&self.pipeline);
            pass.draw_indexed_vertices();
            pass.end();
        }

        g4.swap_buffers().unwrap();
    }
}

fn main() {
    let (kinc, _) = kinc::KincBuilder::new("Kinc + Rust", 500, 500).build();

    let vb = kinc::compile_shader!(
        vertex,
        "#version 450

in vec3 pos;

void main() {
	gl_Position = vec4(pos.x, pos.y, 0.5, 1.0);
}"
    );

    let fb = kinc::compile_shader!(
        fragment,
        "#version 450

out vec4 frag;

void main() {
	frag = vec4(1.0, 0.0, 0.0, 1.0);
}"
    );

    let vertex_shader = kinc::g4::Shader::new(vb, kinc::g4::ShaderType::Vertex);
    let fragment_shader = kinc::g4::Shader::new(fb, kinc::g4::ShaderType::Fragment);
    let vertex_structure = VertexStructureBuilder::new()
        .add("pos", kinc::g4::VertexData::F32_3X)
        .build();
    let pipeline = PipelineBuilder::new(
        &vertex_shader,
        &fragment_shader,
        &[vertex_structure.clone()],
        &[Default::default()],
    )
    .build();

    let mut vertex_buffer = VertexBuffer::new(VertexBufferDesc {
        count: 3,
        vertex_structure,
        usage: kinc::g4::Usage::Static,
        instance_data_step_rate: 0,
    });
    vertex_buffer
        .lock_all::<f32>()
        .copy_from_slice(&[-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0]);
    let index_buffer =
        IndexBuffer::new(3, kinc::g4::Usage::Static, kinc::g4::IndexBufferFormat::U32);
    index_buffer.lock::<u32>().copy_from_slice(&[0, 1, 2]);
    kinc.start(Shader {
        pipeline,
        vertex_buffer,
        index_buffer,
    });
}
