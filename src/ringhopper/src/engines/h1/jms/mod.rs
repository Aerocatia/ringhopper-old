use crate::types::*;
use crate::error::*;
use std::str::FromStr;
use std::fmt::Display;
use strings::*;

#[cfg(test)]
mod tests;

/// Delimiters used in JMS files.
pub const JMS_DELIMITERS: [char; 3] = ['\r', '\n', '\t'];

/// Version for JMS
const JMS_VERSION: u16 = 8200;

/// Factor for dividing position
const SCALE_FACTOR: f32 = 100.0;

trait JMSParser: Sized {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Self>;
    fn to_jms_string(&self) -> String;
}

fn read_str_token<'a>(string: &'a str, offset: &mut usize) -> ErrorMessageResult<&'a str> {
    let bytes = string.as_bytes();

    // Find the range of our string
    let mut start = None;
    for i in *offset..bytes.len() {
        // Find first instance of non-delimiter
        let byte = &bytes[i];
        if !JMS_DELIMITERS.contains(&(*byte as char)) {
            start = Some(i);
            break;
        }
    }
    let start = match start {
        Some(n) => n,
        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.jms.error_expected_token")))
    };
    let mut end = None;
    for i in start..bytes.len() {
        // Find first instance of delimiter
        let byte = &bytes[i];
        if JMS_DELIMITERS.contains(&(*byte as char)) {
            end = Some(i);
            break;
        }
    }
    let end = match end {
        Some(n) => n,
        None => return Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.jms.error_expected_token_end")))
    };

    let output = &string[start..end];
    *offset = end + 1;

    return Ok(output)
}

fn parse_str_token<T: FromStr>(string: &str, offset: &mut usize) -> ErrorMessageResult<T> where <T as FromStr>::Err: Display {
    let token = read_str_token(string, offset)?;
    match T::from_str(token) {
        Ok(n) => Ok(n),
        Err(e) => Err(ErrorMessage::AllocatedString(e.to_string()))
    }
}

macro_rules! define_parsers {
    ($t:ty) => {
        impl JMSParser for $t {
            fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<$t> {
                parse_str_token(string, offset)
            }
            fn to_jms_string(&self) -> String {
                format!("{}\r\n", self)
            }
        }
    }
}

define_parsers!(u16);
define_parsers!(usize);
define_parsers!(f32);

impl JMSParser for String {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<String> {
        Ok(parse_str_token::<String>(string, offset)?.to_ascii_lowercase()) // lowercase
    }
    fn to_jms_string(&self) -> String {
        format!("{}\r\n", self)
    }
}

impl JMSParser for Option<u16> {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Option<u16>> {
        let token = read_str_token(string, offset)?;
        if token == "-1" || token == "65535" {
            Ok(None)
        }
        else {
            match u16::from_str(token) {
                Ok(n) => Ok(Some(n)),
                Err(e) => Err(ErrorMessage::AllocatedString(e.to_string()))
            }
        }
    }
    fn to_jms_string(&self) -> String {
        match self {
            Some(n) => format!("{}\r\n", n),
            None => format!("-1\r\n")
        }
    }
}

impl<T: JMSParser> JMSParser for Vec<T> {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Vec<T>> {
        let count = usize::from_jms_string(string, offset)?;
        let mut result = Vec::<T>::new();
        result.reserve(count);
        for _ in 0..count {
            result.push(T::from_jms_string(string, offset)?);
        }
        Ok(result)
    }
    fn to_jms_string(&self) -> String {
        let mut string = self.len().to_jms_string();
        for i in self {
            string += &i.to_jms_string();
        }
        string
    }
}

impl JMSParser for (u32,u32,u32) {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<(u32,u32,u32)> {
        Ok((
            parse_str_token(string, offset)?,
            parse_str_token(string, offset)?,
            parse_str_token(string, offset)?
        ))
    }
    fn to_jms_string(&self) -> String {
        format!("{}\t{}\t{}\r\n", self.0, self.1, self.2)
    }
}

impl JMSParser for Point3D {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Point3D> {
        Ok(Point3D {
            x: parse_str_token(string, offset)?,
            y: parse_str_token(string, offset)?,
            z: parse_str_token(string, offset)?
        })
    }
    fn to_jms_string(&self) -> String {
        format!("{}\t{}\t{}\r\n", self.x, self.y, self.z)
    }
}

impl JMSParser for Vector3D {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Vector3D> {
        Ok(Vector3D {
            x: parse_str_token(string, offset)?,
            y: parse_str_token(string, offset)?,
            z: parse_str_token(string, offset)?
        })
    }
    fn to_jms_string(&self) -> String {
        format!("{}\t{}\t{}\r\n", self.x, self.y, self.z)
    }
}

impl JMSParser for Quaternion {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Quaternion> {
        Ok(Quaternion {
            x: parse_str_token(string, offset)?,
            y: parse_str_token(string, offset)?,
            z: parse_str_token(string, offset)?,
            w: parse_str_token(string, offset)?
        })
    }
    fn to_jms_string(&self) -> String {
        format!("{}\t{}\t{}\t{}\r\n", self.x, self.y, self.z, self.w)
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Node {
    pub name: String,
    pub first_child: Option<u16>,
    pub sibling_node: Option<u16>,
    pub rotation: Quaternion,
    pub position: Point3D
}

impl JMSParser for Node {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Node> {
        Ok(Node {
            name: String::from_jms_string(string, offset)?,
            first_child: Option::<u16>::from_jms_string(string, offset)?,
            sibling_node: Option::<u16>::from_jms_string(string, offset)?,
            rotation: Quaternion::from_jms_string(string, offset)?,
            position: Point3D::from_jms_string(string, offset)?.scale(1.0 / SCALE_FACTOR)
        })
    }
    fn to_jms_string(&self) -> String {
        self.name.to_jms_string()
        + &self.first_child.to_jms_string()
        + &self.sibling_node.to_jms_string()
        + &self.rotation.to_jms_string()
        + &self.position.scale(SCALE_FACTOR).to_jms_string()
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Material {
    pub name: String,
    pub tif_path: String
}

impl JMSParser for Material {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Material> {
        Ok(Material {
            name: String::from_jms_string(string, offset)?,
            tif_path: String::from_jms_string(string, offset)?
        })
    }
    fn to_jms_string(&self) -> String {
        self.name.to_jms_string()
        + &self.tif_path.to_jms_string()
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Marker {
    pub name: String,
    pub region: Option<u16>,
    pub node: Option<u16>,
    pub rotation: Quaternion,
    pub position: Point3D,
    pub radius: f32
}

impl JMSParser for Marker {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Marker> {
        Ok(Marker {
            name: String::from_jms_string(string, offset)?,
            region: Option::<u16>::from_jms_string(string, offset)?,
            node: Option::<u16>::from_jms_string(string, offset)?,
            rotation: Quaternion::from_jms_string(string, offset)?,
            position: Point3D::from_jms_string(string, offset)?.scale(1.0 / SCALE_FACTOR),
            radius: f32::from_jms_string(string, offset)?
        })
    }
    fn to_jms_string(&self) -> String {
        self.name.to_jms_string()
        + &self.region.to_jms_string()
        + &self.node.to_jms_string()
        + &self.rotation.to_jms_string()
        + &self.position.scale(SCALE_FACTOR).to_jms_string()
        + &self.radius.to_jms_string()
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Region {
    pub name: String
}

impl JMSParser for Region {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Region> {
        Ok(Region {
            name: String::from_jms_string(string, offset)?
        })
    }
    fn to_jms_string(&self) -> String {
        self.name.to_jms_string()
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Vertex {
    pub node0: Option<u16>,
    pub position: Point3D,
    pub normal: Vector3D,
    pub node1: Option<u16>,
    pub node1_weight: f32,
    pub texture_coordinates: Point3D
}

impl JMSParser for Vertex {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Vertex> {
        Ok(Vertex {
            node0: Option::<u16>::from_jms_string(string, offset)?,
            position: Point3D::from_jms_string(string, offset)?.scale(1.0 / SCALE_FACTOR),
            normal: Vector3D::from_jms_string(string, offset)?,
            node1: Option::<u16>::from_jms_string(string, offset)?,
            node1_weight: f32::from_jms_string(string, offset)?,
            texture_coordinates: {
                let mut c = Point3D::from_jms_string(string, offset)?;
                c.y = 1.0 - c.y;
                c
            }
        })
    }
    fn to_jms_string(&self) -> String {
        self.node0.to_jms_string()
        + &self.position.scale(SCALE_FACTOR).to_jms_string()
        + &self.normal.to_jms_string()
        + &self.node1.to_jms_string()
        + &self.node1_weight.to_jms_string()
        + &{
            let mut c = self.texture_coordinates.clone();
            c.y = 1.0 - c.y;
            c.to_jms_string()
        }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Triangle {
    pub region: Option::<u16>,
    pub shader: Option::<u16>,
    pub vertices: (u32, u32, u32)
}

impl JMSParser for Triangle {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<Triangle> {
        Ok(Triangle {
            region: Option::<u16>::from_jms_string(string, offset)?,
            shader: Option::<u16>::from_jms_string(string, offset)?,
            vertices: <(u32, u32, u32)>::from_jms_string(string, offset)?
        })
    }
    fn to_jms_string(&self) -> String {
        self.region.to_jms_string()
        + &self.shader.to_jms_string()
        + &self.vertices.to_jms_string()
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct JMS {
    pub node_list_checksum: usize,
    pub nodes: Vec<Node>,
    pub materials: Vec<Material>,
    pub markers: Vec<Marker>,
    pub regions: Vec<Region>,
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>
}

impl JMSParser for JMS {
    fn from_jms_string(string: &str, offset: &mut usize) -> ErrorMessageResult<JMS> {
        let version = parse_str_token::<u16>(string, offset)?;
        if version == JMS_VERSION {
            let jms = Ok(JMS {
                node_list_checksum: usize::from_jms_string(string, offset)?,
                nodes: Vec::<Node>::from_jms_string(string, offset)?,
                materials: Vec::<Material>::from_jms_string(string, offset)?,
                markers: Vec::<Marker>::from_jms_string(string, offset)?,
                regions: Vec::<Region>::from_jms_string(string, offset)?,
                vertices: Vec::<Vertex>::from_jms_string(string, offset)?,
                triangles: Vec::<Triangle>::from_jms_string(string, offset)?
            });

            if read_str_token(string, offset).is_err() {
                jms
            }
            else {
                Err(ErrorMessage::StaticString(get_compiled_string!("engine.h1.jms.error_unexpected_token")))
            }
        }
        else {
            Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_version_mismatch"), version_expected=JMS_VERSION, version_read=version)))
        }
    }
    fn to_jms_string(&self) -> String {
        JMS_VERSION.to_jms_string()
        + &self.node_list_checksum.to_jms_string()
        + &self.nodes.to_jms_string()
        + &self.materials.to_jms_string()
        + &self.markers.to_jms_string()
        + &self.regions.to_jms_string()
        + &self.vertices.to_jms_string()
        + &self.triangles.to_jms_string()
    }
}

impl JMS {
    /// Parse the JMS file as a byte array.
    ///
    /// The array must be valid UTF-8.
    ///
    /// Returns an [`Err`] if it could not be parsed.
    pub fn parse_bytes(bytes: &[u8]) -> ErrorMessageResult<JMS> {
        match std::str::from_utf8(bytes) {
            Ok(n) => JMS::parse_str(n),
            Err(_) => Err(ErrorMessage::StaticString(get_compiled_string!("engine.types.error_string_not_valid_utf8")))
        }
    }

    /// Parse the JMS file as a UTF-8 string.
    ///
    /// Returns an [`Err`] if it could not be parsed.
    pub fn parse_str(string: &str) -> ErrorMessageResult<JMS> {
        let mut current_offset = 0usize;
        match JMS::from_jms_string(string, &mut current_offset) {
            Ok(n) => Ok(n),
            Err(e) => {
                let str_bytes = string.as_bytes();
                let mut line = 1usize;
                let mut column = 1usize;
                for i in 0..current_offset {
                    let byte = &str_bytes[i];
                    if *byte == '\n' as u8 {
                        line += 1;
                        column = 1;
                    }
                    else if *byte != '\r' as u8 {
                        column += 1;
                    }
                }
                Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_could_not_parse_jms"), line=line, column=column, error=e)))
            }
        }
    }

    /// Generate a vector of UTF-8 bytes.
    ///
    /// The output will use CRLF line endings.
    pub fn into_bytes(&self) -> Vec<u8> {
        self.to_jms_string().into_bytes()
    }

    /// Validate that the JMS indices are valid.
    pub fn validate(&self) -> ErrorMessageResult<()> {
        // Function to safely compare if an index is out of bounds
        let is_in_bounds = |index: Option<u16>, count: usize| {
            if let Some(n) = index {
                n as usize <= count
            }
            else {
                true
            }
        };

        // Verify triangles
        let vertex_count = self.vertices.len() as u32;
        for i in 0..self.triangles.len() {
            let triangle = &self.triangles[i];
            let validate_vertex = |vertex| {
                if vertex >= vertex_count {
                    Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_vertex"),
                        triangle=i,
                        vertex=vertex,
                        vertex_count=vertex_count)))
                }
                else {
                    Ok(())
                }
            };
            validate_vertex(triangle.vertices.0)?;
            validate_vertex(triangle.vertices.1)?;
            validate_vertex(triangle.vertices.2)?;

            if !is_in_bounds(triangle.region, self.regions.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_region_for_triangle"),
                           triangle=i,
                           region=triangle.region.unwrap(),
                           region_count=self.regions.len())))
            }

            if !is_in_bounds(triangle.shader, self.materials.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_material"),
                           triangle=i,
                           material=triangle.shader.unwrap(),
                           material_count=self.materials.len())))
            }
        }

        // Verify vertices
        for i in 0..self.vertices.len() {
            let vertex = &self.vertices[i];

            if !is_in_bounds(vertex.node0, self.nodes.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_node_for_vertex"),
                           vertex=i,
                           node=vertex.node0.unwrap(),
                           node_count=self.nodes.len())))
            }

            if !is_in_bounds(vertex.node1, self.nodes.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_node_for_vertex"),
                           vertex=i,
                           node=vertex.node1.unwrap(),
                           node_count=self.nodes.len())))
            }
        }

        // Verify markers
        for i in 0..self.markers.len() {
            let marker = &self.markers[i];

            if !is_in_bounds(marker.region, self.regions.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_region_for_marker"),
                           marker=i,
                           region=marker.region.unwrap(),
                           region_count=self.regions.len())))
            }

            if !is_in_bounds(marker.node, self.nodes.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_node_for_marker"),
                           marker=i,
                           node=marker.node.unwrap(),
                           node_count=self.nodes.len())))
            }
        }

        // Verify nodes
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];

            if !is_in_bounds(node.sibling_node, self.nodes.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_node_for_node"),
                           node_this=i,
                           node=node.sibling_node.unwrap(),
                           node_count=self.nodes.len())))
            }

            if !is_in_bounds(node.first_child, self.nodes.len()) {
                return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.jms.error_verify_fail_out_of_bounds_node_for_node"),
                           node_this=i,
                           node=node.first_child.unwrap(),
                           node_count=self.nodes.len())))
            }
        }

        // TODO: Verify infinite looping doesn't occur
        //
        // If a node has a child that has this node as a sibling/child, or the node has a sibling that has this node as a sibling/child, or recursively do this.

        Ok(())
    }

    /// Optimize the JMS, deduping all vertices and triangles as well as removing degenerate triangles.
    pub fn optimize(&mut self) {
        // First, copy our triangles.
        let mut new_triangles = self.triangles.clone();

        // Next, dedupe vertices.
        let mut new_vertices = Vec::new();
        new_vertices.reserve(self.vertices.len());
        let mut new_vertex_index = 0usize;
        for v in &self.vertices {
            let mut found = None;
            for i in 0..new_vertices.len() {
                if new_vertices[i] == *v {
                    found = Some(i);
                    break;
                }
            }

            match found {
                Some(n) => {
                    for t in &mut new_triangles {
                        let new_vertex_index_u32 = new_vertex_index as u32;
                        let n_u32 = n as u32;

                        if t.vertices.0 == new_vertex_index_u32 {
                            t.vertices.0 = n_u32;
                        }
                        else if t.vertices.0 > new_vertex_index_u32 {
                            t.vertices.0 -= 1;
                        }

                        if t.vertices.1 == new_vertex_index_u32 {
                            t.vertices.1 = n_u32;
                        }
                        else if t.vertices.1 > new_vertex_index_u32 {
                            t.vertices.1 -= 1;
                        }

                        if t.vertices.2 == new_vertex_index_u32 {
                            t.vertices.2 = n_u32;
                        }
                        else if t.vertices.2 > new_vertex_index_u32 {
                            t.vertices.2 -= 1;
                        }
                    }
                },
                None => {
                    new_vertex_index += 1;
                    new_vertices.push(*v);
                }
            }
        }

        // Lastly, dedupe triangles.
        new_triangles.dedup();

        self.triangles = new_triangles;
        self.vertices = new_vertices;
    }
}
