use crate::renderer::gl;
use crate::renderer::mesh;
use crate::renderer::texture;

use json::JsonValue;

use std::error::Error;

struct TraverseState {
    translations_meshes: Vec<nalgebra_glm::Vec3>,
    rotations_meshes: Vec<nalgebra_glm::Quat>,
    scales_meshes: Vec<nalgebra_glm::Vec3>,
    meshes_to_load: Vec<usize>,
}

pub struct Model {
    gl: gl::Gl,
    program: gl::types::GLuint,
    json: JsonValue,
    data: Vec<u8>,
    texture_file: &'static [u8],
    meshes: Vec<mesh::Mesh>,
    position: nalgebra_glm::Vec3,
    scale: nalgebra_glm::Vec3,
    rotation: nalgebra_glm::Quat,
}

impl Model {
    pub unsafe fn new(
        gl: gl::Gl,
        program: gl::types::GLuint,
        gltf_file: &'static [u8],
        bin_file: &[u8],
        texture_file: &'static [u8],
    ) -> Self {
        let json = json::parse(std::str::from_utf8(&gltf_file).unwrap()).unwrap();

        let mut state = TraverseState {
            translations_meshes: Vec::new(),
            rotations_meshes: Vec::new(),
            scales_meshes: Vec::new(),
            meshes_to_load: Vec::new(),
        };

        Model::traverse_node(&mut state, json.clone(), 0, nalgebra_glm::Mat4::identity());

        let mut instance = Self {
            gl,
            program,
            json,
            data: bin_file.to_vec(),
            texture_file,
            meshes: Vec::new(),
            position: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            scale: nalgebra_glm::vec3(0.0, 0.0, 0.0),
            rotation: nalgebra_glm::quat_identity(),
        };

        for mesh_index in state.meshes_to_load {
            Model::load_mesh(&mut instance, mesh_index);
        }

        instance
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, position: nalgebra_glm::Vec3) {
        self.position = position;
    }

    #[allow(dead_code)]
    pub fn set_scale(&mut self, scale: nalgebra_glm::Vec3) {
        self.scale = scale;
    }

    #[allow(dead_code)]
    pub fn set_rotation(&mut self, rotation: nalgebra_glm::Quat) {
        self.rotation = rotation;
    }

    pub fn draw(&self) {
        for mesh in self.meshes.iter() {
            mesh.draw(self.position, self.scale, self.rotation);
        }
    }

    fn get_floats(&mut self, accessor: JsonValue) -> Vec<f32> {
        let buff_view_ind = if accessor["bufferView"].is_null() {
            1
        } else {
            accessor["bufferView"].as_u64().unwrap() as usize
        };
        let count = accessor["count"].as_u64().unwrap() as usize;
        let acc_byte_offset = if accessor["byteOffset"].is_null() {
            0
        } else {
            accessor["byteOffset"].as_u64().unwrap() as usize
        };
        let type_str = accessor["type"].as_str().unwrap();

        let buffer_view = &self.json["bufferViews"][buff_view_ind];
        let byte_offset = buffer_view["byteOffset"].as_u64().unwrap() as usize;

        let num_per_vert = interpret_type(type_str).expect("test");

        let beginning_of_data = byte_offset + acc_byte_offset;
        let length_of_data = count * 4 * num_per_vert;
        let mut float_vec = Vec::with_capacity(count * num_per_vert);

        for i in (beginning_of_data..beginning_of_data + length_of_data).step_by(4) {
            let bytes: [u8; 4] = [
                self.data[i],
                self.data[i + 1],
                self.data[i + 2],
                self.data[i + 3],
            ];
            let value: f32;
            unsafe {
                value = std::mem::transmute(bytes);
            }
            float_vec.push(value);
        }

        float_vec
    }

    fn get_indices(&mut self, accessor: JsonValue) -> Result<Vec<u32>, Box<dyn Error>> {
        let buff_view_ind = if accessor["bufferView"].is_null() {
            0
        } else {
            accessor["bufferView"].as_u64().unwrap() as usize
        };
        let count = accessor["count"].as_u64().unwrap() as usize;
        let acc_byte_offset = if accessor["byteOffset"].is_null() {
            0
        } else {
            accessor["byteOffset"].as_u64().unwrap() as usize
        };
        let component_type = accessor["componentType"].as_u64().unwrap() as u32;

        let buffer_view = &self.json["bufferViews"][buff_view_ind];
        let byte_offset = buffer_view["byteOffset"].as_u64().unwrap() as usize;

        let beginning_of_data = byte_offset + acc_byte_offset;
        let mut indices = Vec::with_capacity(count);

        match component_type {
            5125 => {
                for i in (beginning_of_data..beginning_of_data + count * 4).step_by(4) {
                    let bytes: [u8; 4] = [
                        self.data[i],
                        self.data[i + 1],
                        self.data[i + 2],
                        self.data[i + 3],
                    ];
                    let value: u32;
                    unsafe {
                        value = std::mem::transmute(bytes);
                    }
                    indices.push(value);
                }
            }
            5123 => {
                for i in (beginning_of_data..beginning_of_data + count * 2).step_by(2) {
                    let bytes: [u8; 2] = [self.data[i], self.data[i + 1]];
                    let value: u16;
                    unsafe {
                        value = std::mem::transmute(bytes);
                    }
                    indices.push(value as u32);
                }
            }
            5122 => {
                for i in (beginning_of_data..beginning_of_data + count * 2).step_by(2) {
                    let bytes: [u8; 2] = [self.data[i], self.data[i + 1]];
                    let value: i16;
                    unsafe {
                        value = std::mem::transmute(bytes);
                    }
                    indices.push(value as u32);
                }
            }
            _ => return Err("Invalid componentType for indices".into()),
        }

        Ok(indices)
    }

    unsafe fn load_mesh(&mut self, ind_mesh: usize) {
        let pos_acc_ind = self.json["meshes"][ind_mesh]["primitives"][0]["attributes"]["POSITION"]
            .as_u64()
            .expect("Expected a u64 value in JSON") as usize;
        let normal_acc_ind = self.json["meshes"][ind_mesh]["primitives"][0]["attributes"]["NORMAL"]
            .as_u64()
            .expect("Expected a u64 value in JSON") as usize;

        let tex_acc_ind = self.json["meshes"][ind_mesh]["primitives"][0]["attributes"]["TEXCOORD_0"]
            .as_u64()
            .expect("Expected a u64 value in JSON") as usize;

        let ind_acc_ind = self.json["meshes"][ind_mesh]["primitives"][0]["indices"]
            .as_u64()
            .expect("Expected a u64 value in JSON") as usize;

        let pos_vec = Model::get_floats(self, self.json["accessors"][pos_acc_ind].clone());
        let positions = group_floats_vec3(pos_vec);
        let normal_vec = Model::get_floats(self, self.json["accessors"][normal_acc_ind].clone());
        let normals = group_floats_vec3(normal_vec);
        let tex_vec = Model::get_floats(self, self.json["accessors"][tex_acc_ind].clone());
        let tex_uvs = group_floats_vec2(tex_vec);

        let vertices = assemble_vertices(positions, normals, tex_uvs);
        let indices = self
            .get_indices(self.json["accessors"][ind_acc_ind].clone())
            .unwrap();
        let texture = texture::Texture::new(self.gl.clone(), self.program, self.texture_file);

        self.meshes.push(mesh::Mesh::new(
            self.gl.clone(),
            self.program,
            vertices,
            indices,
            texture,
        ));
    }

    unsafe fn traverse_node(
        state: &mut TraverseState,
        json: JsonValue,
        next_node: usize,
        matrix: nalgebra_glm::Mat4,
    ) {
        let node = &json["nodes"][next_node];

        let mut translation: nalgebra_glm::Vec3 = nalgebra_glm::vec3(0.0, 0.0, 0.0);
        if !node["translation"].is_null() {
            let mut trans_values: [f32; 3] = [0.0; 3];
            for i in 0..node["translation"].len() {
                trans_values[i] = node["translation"][i].as_f32().unwrap();
            }
            translation = nalgebra_glm::make_vec3(&trans_values);
        }

        let mut rotation: nalgebra_glm::Quat = nalgebra_glm::quat(1.0, 0.0, 0.0, 0.0);
        if !node["rotation"].is_null() {
            let rotation_values: [f32; 4] = [
                node["rotation"][3].as_f32().unwrap(),
                node["rotation"][0].as_f32().unwrap(),
                node["rotation"][1].as_f32().unwrap(),
                node["rotation"][2].as_f32().unwrap(),
            ];
            rotation = nalgebra_glm::make_quat(&rotation_values)
        }

        let mut scale: nalgebra_glm::Vec3 = nalgebra_glm::vec3(1.0, 1.0, 1.0);
        if !node["scale"].is_null() {
            let mut scale_values: [f32; 3] = [0.0; 3];
            for i in 0..node["scale"].len() {
                scale_values[i] = node["scale"][i].as_f32().unwrap();
            }
            scale = nalgebra_glm::make_vec3(&scale_values);
        }

        let mut mat_node: nalgebra_glm::Mat4 = nalgebra_glm::Mat4::identity();
        if !node["matrix"].is_null() {
            let mut mat_values: [f32; 16] = [0.0; 16];
            for i in 0..node["matrix"].len() {
                mat_values[i] = node["matrix"][i].as_f32().unwrap();
            }
            mat_node = nalgebra_glm::make_mat4(&mat_values);
        }

        let trans = nalgebra_glm::translate(&nalgebra_glm::Mat4::identity(), &translation);
        let rot = nalgebra_glm::quat_cast(&rotation);
        let sca = nalgebra_glm::scale(&nalgebra_glm::Mat4::identity(), &scale);

        let mat_next_node: nalgebra_glm::Mat4 = matrix * mat_node * trans * rot * sca;

        if !node["mesh"].is_null() {
            state.translations_meshes.push(translation);
            state.rotations_meshes.push(rotation);
            state.scales_meshes.push(scale);
            state
                .meshes_to_load
                .push(node["mesh"].as_u64().expect("Expected a u64 value in JSON") as usize);
        }

        if !node["children"].is_null() {
            for i in 0..node["children"].len() {
                Model::traverse_node(
                    state,
                    json.clone(),
                    node["children"][i].as_u64().unwrap() as usize,
                    mat_next_node,
                );
            }
        }
    }
}

fn interpret_type(type_str: &str) -> Result<usize, &'static str> {
    match type_str {
        "SCALAR" => Ok(1),
        "VEC2" => Ok(2),
        "VEC3" => Ok(3),
        "VEC4" => Ok(4),
        _ => Err("Type is invalid (not SCALAR, VEC2, VEC3, or VEC4)"),
    }
}

fn group_floats_vec2(float_vec: Vec<f32>) -> Vec<nalgebra_glm::Vec2> {
    let mut vectors = Vec::with_capacity(float_vec.len() / 2);
    let mut iter = float_vec.iter();

    while let (Some(x), Some(y)) = (iter.next(), iter.next()) {
        vectors.push(nalgebra_glm::vec2(*x, *y));
    }

    vectors
}

fn group_floats_vec3(float_vec: Vec<f32>) -> Vec<nalgebra_glm::Vec3> {
    let mut vectors = Vec::with_capacity(float_vec.len() / 3);
    let mut iter = float_vec.iter();

    while let (Some(x), Some(y), Some(z)) = (iter.next(), iter.next(), iter.next()) {
        vectors.push(nalgebra_glm::vec3(*x, *y, *z));
    }

    vectors
}

#[allow(dead_code)]
fn group_floats_vec4(float_vec: Vec<f32>) -> Vec<nalgebra_glm::Vec4> {
    let mut vectors = Vec::with_capacity(float_vec.len() / 4);
    let mut iter = float_vec.iter();

    while let (Some(x), Some(y), Some(z), Some(w)) =
        (iter.next(), iter.next(), iter.next(), iter.next())
    {
        vectors.push(nalgebra_glm::vec4(*x, *y, *z, *w));
    }

    vectors
}

fn assemble_vertices(
    positions: Vec<nalgebra_glm::Vec3>,
    normals: Vec<nalgebra_glm::Vec3>,
    tex_uvs: Vec<nalgebra_glm::Vec2>,
) -> Vec<f32> {
    let mut vertices = Vec::new();

    for i in 0..positions.len() {
        let position = positions[i];
        let normal = normals[i];
        let tex_uv = tex_uvs[i];

        vertices.push(position.x);
        vertices.push(position.y);
        vertices.push(position.z);

        vertices.push(normal.x);
        vertices.push(normal.y);
        vertices.push(normal.z);

        vertices.push(1.0);
        vertices.push(1.0);
        vertices.push(1.0);

        vertices.push(tex_uv.x);
        vertices.push(tex_uv.y);
    }

    vertices
}
