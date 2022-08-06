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

impl<'a> Callbacks for Shader {
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
        }

        g4.swap_buffers().unwrap();
    }
}

fn main() {
    let (kinc, _) = kinc::KincBuilder::new("Kinc + Rust", 500, 500).build();

    let vertex_shader = kinc::g4::Shader::new(
        include_bytes!("../Deployment/shader.vert"),
        kinc::g4::ShaderType::Vertex,
    );
    let fragment_shader = kinc::g4::Shader::new(
        include_bytes!("../Deployment/shader.frag"),
        kinc::g4::ShaderType::Fragment,
    );
    // let fragment_shader;
    let pipeline = PipelineBuilder::new(&vertex_shader, &fragment_shader).build();
    let vertex_structure = VertexStructureBuilder::new()
        .add("pos", kinc::g4::VertexData::F32_3X)
        .build();
    let vertex_buffer = VertexBuffer::new(VertexBufferDesc {
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
