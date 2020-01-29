#[macro_use]
pub mod input;
pub mod defs;

use log::info;

use std::collections::BTreeMap;
use std::marker::PhantomData;

use glsl::parser::Parse;
use glsl::visitor::Host;

use glium::uniforms::UniformType;
use glium::vertex::AttributeType;

pub use input::{HasUniforms, InstanceInput, ToUniforms, UniformInput};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VertexOutQualifier {
    Flat,
    Smooth,
    Local,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FragmentOutQualifier {
    Local,
    Yield,
}

pub type VariableName = String;
pub type GLSL = String;

pub type Type = UniformType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexOutDef(pub UniformType, pub VertexOutQualifier);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentOutDef(pub UniformType, pub FragmentOutQualifier);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexCore<P, I, V> {
    pub extra_uniforms: BTreeMap<VariableName, UniformType>,
    pub out_defs: BTreeMap<VariableName, VertexOutDef>,
    pub defs: GLSL,
    pub body: Vec<BodyElem>,
    pub phantom: PhantomData<(P, I, V)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyElem {
    Block(GLSL),
    Assignment(VariableName, GLSL),
}

impl BodyElem {
    pub fn is_assignment_of(&self, name: &str) -> bool {
        match self {
            BodyElem::Block(_) => false,
            BodyElem::Assignment(assignment_name, _) => *assignment_name == *name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentCore<P> {
    pub extra_uniforms: BTreeMap<VariableName, UniformType>,
    pub in_defs: BTreeMap<VariableName, VertexOutDef>,
    pub out_defs: BTreeMap<VariableName, FragmentOutDef>,
    pub defs: GLSL,
    pub body: Vec<BodyElem>,
    pub phantom: PhantomData<P>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core<P, I, V> {
    pub vertex: VertexCore<P, I, V>,
    pub fragment: FragmentCore<P>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedCore<P, I, V> {
    pub vertex: VertexCore<P, I, V>,
    pub fragment: FragmentCore<P>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstancingMode {
    Uniforms,
    Vertex,
}

impl<P, I, V> Default for VertexCore<P, I, V> {
    fn default() -> Self {
        Self {
            extra_uniforms: BTreeMap::new(),
            out_defs: BTreeMap::new(),
            defs: "".into(),
            body: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<P> Default for FragmentCore<P> {
    fn default() -> Self {
        Self {
            extra_uniforms: BTreeMap::new(),
            in_defs: BTreeMap::new(),
            out_defs: BTreeMap::new(),
            defs: "".into(),
            body: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<P, I, V> VertexCore<P, I, V> {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn has_out(&self, name: &str) -> bool {
        self.out_defs.contains_key(name)
    }

    pub fn has_out_def(&self, (name, def): (&str, VertexOutDef)) -> bool {
        self.out_defs
            .get(name)
            .map_or(false, |given_def| given_def.0 == def.0)
    }

    pub fn with_extra_uniform(mut self, name: &str, t: UniformType) -> Self {
        self.extra_uniforms.insert(name.into(), t);
        self
    }

    pub fn with_defs(mut self, defs: &str) -> Self {
        self.defs += defs;
        self
    }

    pub fn with_body(mut self, body: &str) -> Self {
        self.body.push(BodyElem::Block(body.into()));
        self
    }

    pub fn with_out(mut self, (name, def): (&str, VertexOutDef), expr: &str) -> Self {
        assert!(!self.has_out(name));

        if name != defs::V_POS.0 {
            // Special case: gl_Position does not need to be defined
            self.out_defs.insert(name.into(), def);
        }

        self.body
            .push(BodyElem::Assignment(name.into(), expr.into()));
        self
    }

    pub fn with_out_expr(mut self, name: &str, expr: &str) -> Self {
        assert!(self.has_out(name));
        self.body
            .push(BodyElem::Assignment(name.into(), expr.into()));
        self
    }

    pub fn with_out_def(mut self, (name, def): (&str, VertexOutDef)) -> Self {
        self.out_defs.insert(name.into(), def);
        self
    }
}

impl<P> FragmentCore<P> {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn has_in(&self, name: &str) -> bool {
        self.in_defs.contains_key(name)
    }

    pub fn has_out(&self, name: &str) -> bool {
        self.out_defs.contains_key(name)
    }

    pub fn has_in_def(&self, (name, def): (&str, VertexOutDef)) -> bool {
        self.in_defs
            .get(name)
            .map_or(false, |given_def| given_def.0 == def.0)
    }

    pub fn has_out_def(&self, (name, def): (&str, FragmentOutDef)) -> bool {
        self.out_defs
            .get(name)
            .map_or(false, |given_def| given_def.0 == def.0)
    }

    pub fn with_extra_uniform(mut self, name: &str, t: UniformType) -> Self {
        self.extra_uniforms.insert(name.into(), t);
        self
    }

    pub fn with_defs(mut self, defs: &str) -> Self {
        self.defs += defs;
        self
    }

    pub fn with_body(mut self, body: &str) -> Self {
        self.body.push(BodyElem::Block(body.into()));
        self
    }

    pub fn with_in_def(mut self, (name, def): (&str, VertexOutDef)) -> Self {
        self.in_defs.insert(name.into(), def);
        self
    }

    pub fn with_out(mut self, (name, def): (&str, FragmentOutDef), expr: &str) -> Self {
        assert!(!self.has_out(name));

        self.out_defs.insert(name.into(), def);
        self.body
            .push(BodyElem::Assignment(name.into(), expr.into()));
        self
    }

    pub fn with_out_expr(mut self, name: &str, expr: &str) -> Self {
        assert!(self.has_out(name));
        self.body
            .push(BodyElem::Assignment(name.into(), expr.into()));
        self
    }

    pub fn with_out_def(mut self, (name, def): (&str, FragmentOutDef)) -> Self {
        self.out_defs.insert(name.into(), def);
        self
    }
}

fn does_core_use_variable(defs: &str, body: &[BodyElem], var_name: &str) -> bool {
    struct Visitor<'a> {
        var_name: &'a str,
        is_used: bool,
    }

    impl<'a> glsl::visitor::Visitor for Visitor<'a> {
        fn visit_identifier(
            &mut self,
            identifier: &mut glsl::syntax::Identifier,
        ) -> glsl::visitor::Visit {
            if identifier.as_str() == self.var_name {
                self.is_used = true;
            }

            glsl::visitor::Visit::Children
        }
    }

    let mut visitor = Visitor {
        var_name,
        is_used: false,
    };

    // Putting braces around allows parsing compound statements.
    let compiled_body = "{".to_owned() + &compile_body(body) + "}";

    let mut body = glsl::syntax::Statement::parse(compiled_body).unwrap();
    body.visit(&mut visitor);

    if !defs.is_empty() {
        let mut defs = glsl::syntax::TranslationUnit::parse(defs).unwrap();
        defs.visit(&mut visitor);
    }

    visitor.is_used
}

impl<P, I, V> Core<P, I, V>
where
    P: Clone,
    I: Clone,
    V: Clone,
{
    pub fn link(&self) -> LinkedCore<P, I, V> {
        let mut fragment = self.fragment.clone();

        // Remove unused local fragment shader outputs.
        //
        // We take the transitive closure by looping, since removing one output
        // may cause another output to become unused.
        let mut changed = true;

        while changed {
            changed = false;

            for (out_name, FragmentOutDef(_, q)) in fragment.out_defs.clone().iter() {
                if *q == FragmentOutQualifier::Local {
                    let is_used = does_core_use_variable(&fragment.defs, &fragment.body, &out_name);

                    if !is_used {
                        info!("Removing unused local fragment output {}", out_name);

                        fragment.out_defs.remove(out_name);
                        fragment
                            .body
                            .retain(|elem| !elem.is_assignment_of(out_name));

                        changed = true;
                    }
                }
            }
        }

        // Remove unused inputs from fragment shader.
        fragment.in_defs = fragment
            .in_defs
            .clone()
            .into_iter()
            .filter(|(in_name, _)| {
                let r = does_core_use_variable(&fragment.defs, &fragment.body, &in_name);

                if !r {
                    info!("Removing unused fragment input {}", in_name);
                }

                r
            })
            .collect();

        // Demote vertex shader outputs to local when not needed by fragment
        // shader.
        let mut vertex = self.vertex.clone();

        for (out_name, VertexOutDef(_, q)) in vertex.out_defs.iter_mut() {
            if !fragment.has_in(&out_name) {
                info!("Demoting unconnected vertex output {} to local", out_name);

                *q = VertexOutQualifier::Local;
            }
        }

        // Remove unused local vertex shader outputs.
        //
        // We take the transitive closure by looping, since removing one output
        // may cause another output to become unused.
        let mut changed = true;

        while changed {
            changed = false;

            for (out_name, VertexOutDef(_, q)) in vertex.out_defs.clone().iter() {
                if *q == VertexOutQualifier::Local {
                    let is_used = does_core_use_variable(&vertex.defs, &vertex.body, &out_name);

                    if !is_used {
                        info!("Removing unused local vertex output {}", out_name);

                        vertex.out_defs.remove(out_name);
                        fragment
                            .body
                            .retain(|elem| !elem.is_assignment_of(out_name));

                        changed = true;
                    }
                }
            }
        }

        LinkedCore { vertex, fragment }
    }
}

impl<P, I, V> Core<P, I, V>
where
    P: UniformInput + Clone,
    I: UniformInput + Clone,
    V: glium::vertex::Vertex,
{
    pub fn build_program<F: glium::backend::Facade>(
        &self,
        facade: &F,
        mode: InstancingMode,
    ) -> Result<glium::Program, BuildError> {
        self.link().build_program(facade, mode)
    }
}

impl<P, I, V> LinkedCore<P, I, V>
where
    P: UniformInput,
    I: UniformInput,
    V: glium::vertex::Vertex,
{
    pub fn build_program<F: glium::backend::Facade>(
        &self,
        facade: &F,
        mode: InstancingMode,
    ) -> Result<glium::Program, BuildError> {
        let vertex = self.vertex.compile(mode);
        let fragment = self.fragment.compile();

        //println!("{}", vertex);
        //println!("{}", fragment);

        // We use the long form of `glium::Program` construction here, since
        // glium by default sets `outputs_rgb` to false, which causes it to
        // enable `GL_FRAMEBUFFER_SRGB` later on when rendering. This
        // apparently has the effect of OpenGL applying gamma correction when
        // rendering to the screen, at least from what I could tell on Ubuntu.
        // Thus, everything turns out too light when already using corrected
        // colors. Seems weird, and there's definitely still something else
        // going on here.
        //
        // Related issue: https://github.com/rust-windowing/glutin/issues/1175
        glium::Program::new(
            facade,
            glium::program::ProgramCreationInput::SourceCode {
                vertex_shader: &vertex,
                fragment_shader: &fragment,
                geometry_shader: None,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )
        .map_err(|error| BuildError {
            compiled_vertex_source: vertex,
            compiled_fragment_source: fragment,
            error,
        })
    }
}

#[derive(Debug)]
pub struct BuildError {
    pub compiled_vertex_source: String,
    pub compiled_fragment_source: String,
    pub error: glium::program::ProgramCreationError,
}

fn compile_type(t: Type) -> &'static str {
    match t {
        Type::Float => "float",
        Type::FloatVec2 => "vec2",
        Type::FloatVec3 => "vec3",
        Type::FloatVec4 => "vec4",
        Type::FloatMat2 => "mat2",
        Type::FloatMat3 => "mat3",
        Type::FloatMat4 => "mat4",
        Type::Int => "int",
        Type::IntVec2 => "ivec2",
        Type::IntVec3 => "ivec3",
        Type::IntVec4 => "ivec4",
        Type::Sampler2d => "sampler2D",
        Type::Bool => "bool",
        _ => unimplemented!("Given Type not yet supported: {:?}", t),
    }
}

fn compile_variable_def(prefix: &str, name: &str, t: Type) -> String {
    format!("{} {} {};\n", prefix, compile_type(t), name,)
}

fn compile_variable_defs<I>(prefix: &str, defs: I) -> String
where
    I: Iterator<Item = (String, Type)>,
{
    defs.map(|(name, t)| compile_variable_def(prefix, &name, t))
        .collect::<Vec<_>>()
        .join("")
}

fn compile_vertex_out_defs(
    in_out_prefix: &str,
    defs: &BTreeMap<VariableName, VertexOutDef>,
) -> String {
    defs.iter()
        .map(|(name, VertexOutDef(t, q))| {
            let prefix = match q {
                VertexOutQualifier::Flat => "flat ".to_string() + in_out_prefix,
                VertexOutQualifier::Smooth => "smooth ".to_string() + in_out_prefix,
                VertexOutQualifier::Local => "".to_string(),
            };

            compile_variable_def(&prefix, name, *t)
        })
        .collect::<Vec<_>>()
        .join("")
}

fn compile_fragment_out_defs(defs: &BTreeMap<VariableName, FragmentOutDef>) -> String {
    defs.iter()
        .map(|(name, FragmentOutDef(t, q))| {
            let prefix = match q {
                FragmentOutQualifier::Local => "",
                FragmentOutQualifier::Yield => "out ",
            };

            compile_variable_def(prefix, name, *t)
        })
        .collect::<Vec<_>>()
        .join("")
}

fn compile_uniform_input<P: UniformInput>() -> String {
    let uniforms = P::uniform_input_defs();

    compile_variable_defs("uniform", uniforms.iter().cloned())
}

fn compile_instance_input<P: UniformInput>(mode: InstancingMode) -> String {
    let uniforms = P::uniform_input_defs();

    match mode {
        InstancingMode::Uniforms => compile_variable_defs("uniform", uniforms.iter().cloned()),
        InstancingMode::Vertex => compile_variable_defs("in", uniforms.iter().cloned()),
    }
}

fn compile_body(body: &[BodyElem]) -> String {
    body.iter()
        .map(|elem| match elem {
            BodyElem::Block(body) => body.clone(),
            BodyElem::Assignment(name, expr) => format!("    {} = {};\n", name, expr),
        })
        .collect()
}

fn attribute_type(t: AttributeType) -> Type {
    match t {
        AttributeType::F32 => Type::Float,
        AttributeType::F32F32 => Type::FloatVec2,
        AttributeType::F32F32F32 => Type::FloatVec3,
        AttributeType::F32F32F32F32 => Type::FloatVec4,
        _ => unimplemented!("Given AttributeType not yet supported: {:?}", t),
    }
}

fn compile_vertex_attributes<V: glium::vertex::Vertex>() -> String {
    let bindings = V::build_bindings();

    let mut attributes = Vec::new();
    for i in 0..bindings.len() {
        attributes.push((bindings[i].0.to_string(), attribute_type(bindings[i].2)));
    }

    compile_variable_defs("in", attributes.iter().cloned())
}

impl<P, I, V> VertexCore<P, I, V>
where
    P: UniformInput,
    I: UniformInput,
    V: glium::vertex::Vertex,
{
    pub fn compile(&self, mode: InstancingMode) -> String {
        let mut s = String::new();

        s += "#version 330\n\n";

        s += &compile_uniform_input::<P>();
        s += "\n";
        s += &compile_instance_input::<I>(mode);
        s += "\n";
        s += &compile_variable_defs("uniform", self.extra_uniforms.clone().into_iter());
        s += "\n";
        s += &compile_vertex_attributes::<V>();
        s += "\n";
        s += &compile_vertex_out_defs("out", &self.out_defs);
        s += "\n";

        s += &self.defs;
        s += "\n";

        s += "void main() {\n";
        s += &compile_body(&self.body);
        s += "}\n";

        s
    }
}

impl<P> FragmentCore<P>
where
    P: UniformInput,
{
    pub fn compile(&self) -> String {
        let mut s = String::new();

        s += "#version 330\n\n";

        s += &compile_uniform_input::<P>();
        s += "\n";
        s += &compile_variable_defs("uniform", self.extra_uniforms.clone().into_iter());
        s += "\n";
        s += &compile_vertex_out_defs("in", &self.in_defs);
        s += "\n";
        s += &compile_fragment_out_defs(&self.out_defs);
        s += "\n";

        s += &self.defs;
        s += "\n";

        s += "void main() {\n";
        s += &compile_body(&self.body);
        s += "}\n";

        s
    }
}
