#[macro_use]
extern crate glium;

extern crate nalgebra;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);

fn create_perspective(width: f32, height: f32) ->  nalgebra::PerspMat3<f32> {
	nalgebra::PerspMat3::<f32>::new(width/height, std::f32::consts::FRAC_PI_3, 0.1, 20.0)
}

fn tovec(input: [f32; 3]) -> nalgebra::Vec3<f32> {
	nalgebra::Vec3::<f32>::new(input[0], input[1], input[2])
}

fn tesslate(radius: f32, input_vertices: [nalgebra::Vec3<f32>; 3], input_vertex_indecis: [u16; 3], next_index: u16) -> ([nalgebra::Vec3<f32>; 3], [[u16; 3]; 4]) {
	use nalgebra::*;
	let new_vertices = [(input_vertices[0] + input_vertices[1]).normalize() * radius, (input_vertices[1] + input_vertices[2]).normalize() * radius, (input_vertices[0] + input_vertices[2]).normalize() * radius];

	let new_indices = [
		[input_vertex_indecis[0], next_index, next_index + 2],
		[next_index, input_vertex_indecis[1], next_index +1],
		[next_index + 2, next_index + 1, input_vertex_indecis[2]],
		[next_index, next_index + 1, next_index + 2],
	];


	(new_vertices, new_indices)
}

fn main() {
    use glium::DisplayBuild;
    use glium::Surface;
    use nalgebra::*;

    // building the display, ie. the main object
    let display = glium::glutin::WindowBuilder::new()
    	.with_dimensions(500,500)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let phi = 1.61803398875 as f32;

    let radius = ((1.0 + phi*phi) as f64).sqrt() as f32;

    let mut shape = vec![
    	Vertex { position: [0.0, 1.0, phi] },
    	Vertex { position: [0.0, -1.0, phi] },
    	Vertex { position: [0.0, 1.0, -phi] },
    	Vertex { position: [0.0, -1.0, -phi] },

    	Vertex { position: [1.0, phi, 0.0] },
    	Vertex { position: [-1.0, phi, 0.0] },
    	Vertex { position: [1.0, -phi, 0.0] },
    	Vertex { position: [-1.0, -phi, 0.0] },

    	Vertex { position: [phi, 0.0, 1.0] },
    	Vertex { position: [phi, 0.0, -1.0] },
    	Vertex { position: [-phi, 0.0, 1.0] },
    	Vertex { position: [-phi, 0.0, -1.0] },
    ];


    let mut triangles: Vec<[u16; 3]> = vec![
    	[4, 5, 0],
    	[4, 5, 2],
    	[6, 7, 1],
    	[6, 7, 3],


    	[8, 9, 4],
    	[8, 9, 6],
    	[10, 11, 5],
    	[10, 11, 7],

    	[0, 1, 8],
    	[0, 1, 10],
    	[2, 3, 9],
    	[2, 3, 11],

    	[0, 4, 8],
    	[0, 5, 10],

    	[1, 6, 8],
    	[1, 7, 10],

    	[2, 4, 9],
    	[2, 5, 11],

    	[3, 6, 9],
    	[3, 7, 11],
    ];
    let mut next_index = 12;

    {
    	for _ in 0..3 {

    		let mut next_triangles: Vec<[u16; 3]> = vec![];
			
			{
				let mut helper = |indices: [u16;3]| {
					let (new_vertices, new_indices) = tesslate(radius, [tovec(shape[indices[0] as usize].position), tovec(shape[indices[1] as usize].position), tovec(shape[indices[2] as usize].position)], indices, next_index);
					shape.extend(new_vertices.iter().map(|v| Vertex {position :[v.x, v.y, v.z]}));
				    next_triangles.extend(new_indices.iter());
				    next_index += 3;
				};

				for triangle in triangles {
					helper(triangle);
				}
			}

			triangles = next_triangles.clone();
		}
	}
    	

    let mut index_data: Vec<u16> = vec![];
    for triangle in triangles {
		index_data.extend(triangle.iter());
	}

    let triangle_indices = glium::index::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &index_data).unwrap();

  	let mut width = 500.0;
  	let mut height = 500.0;
    let mut perspective = create_perspective(width, height);
    let mut camera: nalgebra::Iso3<f32> = nalgebra::one();
    camera.append_rotation_mut(&nalgebra::Vec3::new(0.0,std::f32::consts::PI,0.0));

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let vertex_shader_src = r#"
        #version 140

        uniform mat4 matrix;
        in vec3 position;

        void main() {
            gl_Position = matrix * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        uniform vec3 color_in;
        out vec4 color;

        void main() {
            color = vec4(color_in, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();



    loop {
    	let mut target = display.draw();

    	let red_color: [f32; 3] = [1.0, 0.0, 0.0];
    	let black_color: [f32; 3] = [0.0, 0.0, 0.0];

    	let matrix = *(perspective.as_mat()) * camera.to_homogeneous();
    	let uniforms_black = uniform! {
    	    matrix: *(matrix.as_ref()),
    	    color_in: black_color,
    	};

    	let uniforms_red = uniform! {
    	    matrix: *(matrix.as_ref()),
    	    color_in: red_color,
    	};

    	let params_line = glium::DrawParameters {
    	    point_size: Some(20.0),
    	    polygon_mode: glium::draw_parameters::PolygonMode::Line,
    	    depth: glium::Depth {
    	        test: glium::draw_parameters::DepthTest::IfLess,
    	        write: true,
    	        .. Default::default()
    	    },
    	    .. Default::default()
    	};

    	let params_fill = glium::DrawParameters {
    	    point_size: Some(20.0),
    	    depth: glium::Depth {
    	        test: glium::draw_parameters::DepthTest::IfLess,
    	        write: true,
    	        .. Default::default()
    	    },
    	    .. Default::default()
    	};

    	target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
    	target.draw(&vertex_buffer, &triangle_indices, &program, &uniforms_red, &params_fill).unwrap();
    	target.draw(&vertex_buffer, &triangle_indices, &program, &uniforms_black, &params_line).unwrap();
    	target.draw(&vertex_buffer, &indices, &program, &uniforms_black, &params_line).unwrap();
    	target.finish().unwrap();

        // listing the events produced by the window and waiting to be received
        for ev in display.poll_events() {
        	// println!("{:?}", ev);
            match ev {
                glium::glutin::Event::Closed => return,   // the window has been closed by the user
                glium::glutin::Event::Resized(new_width, new_height) => {
                	width = new_width as f32;
                	height = new_height as f32;
                	perspective = create_perspective(width, height);
                }
                glium::glutin::Event::KeyboardInput(_, key, _) => {
                	if key == 111 {
                		camera.append_translation_mut(&nalgebra::Vec3::new(0.0,-0.1,0.0));
                	} else if key == 116 {
                		camera.append_translation_mut(&nalgebra::Vec3::new(0.0,0.1,0.0));
                	} else if key == 25 {
                		camera.append_translation_mut(&nalgebra::Vec3::new(0.0,0.0,-0.1));
                	} else if key == 39 {
                		camera.append_translation_mut(&nalgebra::Vec3::new(0.0,0.0,0.1));
                	} else if key == 24 {
                		camera.append_rotation_mut(&nalgebra::Vec3::new(0.0,-0.1,0.0));
                	} else if key == 26 {
                		camera.append_rotation_mut(&nalgebra::Vec3::new(0.0,0.1,0.0));
                	} else if key == 38 {
                		camera.append_translation_mut(&nalgebra::Vec3::new(-0.1,0.0,0.0));
                	} else if key == 40 {
                		camera.append_translation_mut(&nalgebra::Vec3::new(0.1,0.0,0.0));
                	} 
                }
                _ => ()
            }
        }
    }
}