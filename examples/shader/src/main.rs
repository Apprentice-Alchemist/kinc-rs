use kinc::{
    g4::{
        ClearMode, IndexBuffer, Pipeline, PipelineBuilder, VertexBuffer, VertexBufferDesc,
        VertexStructure, VertexStructureBuilder,
    },
    Kinc, KincApp,
};

use krafix::shader;

struct Shader {
    // vertex_structure: &'static VertexStructure,
    // pipeline: Pipeline,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
}

impl<'a> KincApp for Shader {
    fn update(&mut self, kinc: &mut Kinc) {
        let mut g4 = kinc.g4();
        let window = kinc.default_window();
        {
            let mut pass = g4.begin(&window);
            pass.clear(ClearMode::COLOR, 0xFF0FFFFF, 1.0, 1);
            pass.set_vertex_buffer(&self.vertex_buffer);
            pass.set_index_buffer(&self.index_buffer);
        }

        g4.swap_buffers().unwrap();
    }
}

fn main() {
    let (kinc, _) = kinc::KincBuilder::new("Kinc + Rust", 500, 500).build();
    let vertex_shader = shader!(vertex: r#"""
    """#);
    // let fragment_shader;
    // let pipeline = PipelineBuilder::new(vertex_shader, fragment_shader).build();
    let vertex_structure = VertexStructureBuilder::new()
        .add("pos", kinc::g4::VertexData::F32_3X)
        .build();
    let vertex_buffer = VertexBuffer::new(VertexBufferDesc {
        count: 0,
        vertex_structure,
        usage: kinc::g4::Usage::Static,
        instance_data_step_rate: 0,
    });
    let index_buffer =
        IndexBuffer::new(3, kinc::g4::Usage::Static, kinc::g4::IndexBufferFormat::U32);
    index_buffer.lock::<u32>().copy_from_slice(&[0, 1, 2]);
    kinc.start(Shader {
        // pipeline,
        vertex_buffer,
        index_buffer,
    });
}
