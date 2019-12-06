use glium::uniforms::{
    AsUniformValue, EmptyUniforms, UniformType, UniformValue, Uniforms, UniformsStorage,
};

pub struct ToUniformsWrapper<'b, U: ?Sized>(&'b U);

impl<'b, U: ToUniforms> Uniforms for ToUniformsWrapper<'b, U> {
    fn visit_values<'a, F>(&'a self, output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        ToUniforms::visit_values(self.0, output);
    }
}

pub trait ToUniforms {
    fn visit_values<'a, F>(&'a self, output: F)
    where
        F: FnMut(&str, UniformValue<'a>);

    fn to_uniforms(&self) -> ToUniformsWrapper<'_, Self> {
        ToUniformsWrapper(self)
    }
}

pub trait UniformInput: ToUniforms {
    fn uniform_input_defs() -> Vec<(String, UniformType)>;
}

pub trait InstanceInput: UniformInput {
    type Vertex: glium::vertex::Vertex + ToUniforms;

    fn to_vertex(&self) -> Self::Vertex;
}

// The following type aliases have the same name as variants in glium's
// `UniformValue`. This allows us to use the same macro parameters foor
// implementing both `ToUniforms` and `ToVertex`. Yeah, it's hacky though.
#[allow(dead_code)]
pub type Bool = bool;
#[allow(dead_code)]
pub type Float = f32;
#[allow(dead_code)]
pub type Vec2 = [f32; 2];
#[allow(dead_code)]
pub type Vec3 = [f32; 3];
#[allow(dead_code)]
pub type Vec4 = [f32; 4];
#[allow(dead_code)]
pub type Mat2 = [[f32; 2]; 2];
#[allow(dead_code)]
pub type Mat3 = [[f32; 3]; 3];
#[allow(dead_code)]
pub type Mat4 = [[f32; 4]; 4];

/// Dummy enum to ease mapping from UniformValue variants to UniformType.
/// This is just a helper for the `impl_to_uniforms` macro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UniformTypeDummy {
    Bool,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl UniformTypeDummy {
    pub fn to_uniform_type(self) -> UniformType {
        match self {
            UniformTypeDummy::Bool => UniformType::Bool,
            UniformTypeDummy::Float => UniformType::Float,
            UniformTypeDummy::Vec2 => UniformType::FloatVec2,
            UniformTypeDummy::Vec3 => UniformType::FloatVec3,
            UniformTypeDummy::Vec4 => UniformType::FloatVec4,
            UniformTypeDummy::Mat2 => UniformType::FloatMat2,
            UniformTypeDummy::Mat3 => UniformType::FloatMat3,
            UniformTypeDummy::Mat4 => UniformType::FloatMat4,
        }
    }
}

#[macro_export]
macro_rules! impl_uniform_input {
    ($ty:ident, $this:ident => { $( $field:ident: $variant:ident => $value:expr, )* } $(,)? ) => {
        impl $crate::shader::ToUniforms for $ty {
            fn visit_values<'a, F>(&'a $this, mut output: F)
            where
                F: FnMut(&str, glium::uniforms::UniformValue<'a>),
            {
                $(
                    output(stringify!($field), glium::uniforms::UniformValue::$variant($value));
                )*
            }
        }

        impl $crate::shader::UniformInput for $ty {
            fn uniform_input_defs() -> Vec<(String, glium::uniforms::UniformType)> {
                vec![
                    $(
                        (
                            stringify!($field).to_string(),
                            $crate::shader::input::UniformTypeDummy::$variant.to_uniform_type()
                        ),
                    )*
                ]
            }
        }
    }
}

#[macro_export]
macro_rules! impl_to_vertex {
    ($ty:ident, $this:ident => { $( $field:ident: $variant:ident => $value:expr, )* } $(,)? ) => {
        #[derive(Copy, Clone, Debug)]
        pub struct MyVertex {
            $(
                $field: $crate::shader::input::$variant,
            )*
        }

        use glium::implement_vertex;
        implement_vertex!(MyVertex, $($field,)*);

        impl $crate::shader::ToVertex for $ty {
            type Vertex = MyVertex;

            fn to_vertex(&$this) -> Self::Vertex {
                Self::Vertex {
                    $(
                        $field: $value,
                    )*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! impl_instance_input {
    ($ty:ident, $this:ident => { $( $field:ident: $variant:ident => $value:expr, )* } $(,)? ) => {
        $crate::impl_uniform_input!($ty, $this => { $($field: $variant => $value, )* });

        #[derive(Copy, Clone, Debug)]
        pub struct MyVertex {
            $(
                $field: $crate::shader::input::$variant,
            )*
        }

        use glium::implement_vertex;
        implement_vertex!(MyVertex, $($field,)*);

        impl $crate::shader::ToUniforms for MyVertex {
            fn visit_values<'a, F>(&'a self, mut output: F)
            where
                F: FnMut(&str, glium::uniforms::UniformValue<'a>),
            {
                $(
                    output(stringify!($field), glium::uniforms::UniformValue::$variant(self.$field));
                )*
            }
        }

        impl $crate::shader::UniformInput for MyVertex {
            fn uniform_input_defs() -> Vec<(String, glium::uniforms::UniformType)> {
                $ty::uniform_input_defs()
            }
        }

        impl $crate::shader::InstanceInput for $ty {
            type Vertex = MyVertex;

            fn to_vertex(&$this) -> Self::Vertex {
                Self::Vertex {
                    $(
                        $field: $value,
                    )*
                }
            }
        }
    }
}

impl ToUniforms for () {
    fn visit_values<'a, F>(&'a self, _: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
    }
}

impl<U1, U2> ToUniforms for (U1, U2)
where
    U1: ToUniforms,
    U2: ToUniforms,
{
    fn visit_values<'a, F>(&'a self, mut output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        // F is not Copy, so we have to wrap into a lambda here
        self.0.visit_values(|x, y| output(x, y));
        self.1.visit_values(|x, y| output(x, y));
    }
}

impl<U1, U2, U3> ToUniforms for (U1, U2, U3)
where
    U1: ToUniforms,
    U2: ToUniforms,
    U3: ToUniforms,
{
    fn visit_values<'a, F>(&'a self, mut output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        // F is not Copy, so we have to wrap into a lambda here
        self.0.visit_values(|x, y| output(x, y));
        self.1.visit_values(|x, y| output(x, y));
        self.2.visit_values(|x, y| output(x, y));
    }
}

impl<'b, U> ToUniforms for &'b U
where
    U: ToUniforms,
{
    fn visit_values<'a, F>(&'a self, output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        U::visit_values(self, output)
    }
}

impl<U> ToUniforms for Option<U>
where
    U: ToUniforms,
{
    fn visit_values<'a, F>(&'a self, output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        if let Some(uniforms) = self.as_ref() {
            uniforms.visit_values(output);
        }
    }
}

impl ToUniforms for EmptyUniforms {
    fn visit_values<'a, F>(&'a self, _output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
    }
}

impl<'n, T: AsUniformValue, R: Uniforms> ToUniforms for UniformsStorage<'n, T, R> {
    fn visit_values<'a, F>(&'a self, output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        Uniforms::visit_values(self, output);
    }
}

impl UniformInput for () {
    fn uniform_input_defs() -> Vec<(String, UniformType)> {
        Vec::new()
    }
}

impl<U1, U2> UniformInput for (U1, U2)
where
    U1: UniformInput,
    U2: UniformInput,
{
    fn uniform_input_defs() -> Vec<(String, UniformType)> {
        let mut result = U1::uniform_input_defs();
        result.append(&mut U2::uniform_input_defs());

        result
    }
}
