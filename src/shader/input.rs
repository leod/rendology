use glium::uniforms::{
    AsUniformValue, EmptyUniforms, UniformType, UniformValue, Uniforms, UniformsStorage,
};

pub trait ToUniforms {
    type Uniforms: Uniforms;

    fn to_uniforms(&self) -> Self::Uniforms;
}

pub trait UniformInput: ToUniforms {
    fn uniform_input_defs() -> Vec<(String, UniformType)>;
}

pub trait InstanceInput: UniformInput {
    type Vertex: glium::vertex::Vertex + ToUniforms;

    fn to_vertex(&self) -> Self::Vertex;
}

#[macro_export]
macro_rules! impl_uniform_input_detail {
    ($ty:ident, $mod:ident, $this:ident => { $( $field:ident: $type:ty => $value:expr, )* } $(,)? ) => {
        #[derive(Copy, Clone, Debug)]
        pub struct MyUniforms {
            $(
                $field: $type,
            )*
        }

        impl glium::uniforms::Uniforms for MyUniforms {
            fn visit_values<'a, F>(&'a self, mut output: F)
            where
                F: FnMut(&str, glium::uniforms::UniformValue<'a>),
            {
                use glium::uniforms::AsUniformValue;

                $(
                    output(stringify!($field), self.$field.as_uniform_value());
                )*
            }
        }

        impl $crate::shader::ToUniforms for super::$ty {
            type Uniforms = MyUniforms;

            fn to_uniforms(&$this) -> MyUniforms {
                MyUniforms {
                    $(
                        $field: $value,
                    )*
                }
            }
        }

        impl $crate::shader::UniformInput for super::$ty {
            fn uniform_input_defs() -> Vec<(String, glium::uniforms::UniformType)> {
                vec![
                    $(
                        (stringify!($field).to_string(), <$type as $crate::shader::input::StaticUniformType>::TYPE),
                    )*
                ]
            }
        }
    }
}

#[macro_export]
macro_rules! impl_uniform_input {
    ($ty:ident, $mod:ident, $this:ident => { $( $field:ident: $type:ty => $value:expr, )* } $(,)? ) => {
        mod $mod {
            $crate::impl_uniform_input_detail!($ty, $mod, $this => { $($field: $type => $value, )* });
        }
    }
}

#[macro_export]
macro_rules! impl_instance_input {
    ($ty:ident, $mod:ident, $this:ident => { $( $field:ident: $type:ty => $value:expr, )* } $(,)? ) => {
        mod $mod {
            $crate::impl_uniform_input_detail!($ty, $mod, $this => { $($field: $type => $value, )* });

            use glium::implement_vertex;
            implement_vertex!(MyUniforms, $($field,)*);

            impl $crate::shader::ToUniforms for MyUniforms {
                type Uniforms = Self;

                fn to_uniforms(&self) -> Self {
                    self.clone()
                }
            }

            impl $crate::shader::InstanceInput for super::$ty {
                type Vertex = MyUniforms;

                fn to_vertex(&$this) -> Self::Vertex {
                    Self::Vertex {
                        $(
                            $field: $value,
                        )*
                    }
                }
            }
        }

        /*#[derive(Copy, Clone, Debug)]
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
        }*/
    }
}

impl ToUniforms for () {
    type Uniforms = EmptyUniforms;

    fn to_uniforms(&self) -> Self::Uniforms {
        EmptyUniforms
    }
}

impl<'b, U> ToUniforms for &'b U
where
    U: ToUniforms,
{
    type Uniforms = U::Uniforms;

    fn to_uniforms(&self) -> Self::Uniforms {
        (*self).to_uniforms()
    }
}

impl<U1, U2> ToUniforms for (U1, U2)
where
    U1: ToUniforms,
    U2: ToUniforms,
{
    type Uniforms = UniformsPair<<U1 as ToUniforms>::Uniforms, <U2 as ToUniforms>::Uniforms>;

    fn to_uniforms(&self) -> Self::Uniforms {
        UniformsPair(self.0.to_uniforms(), self.1.to_uniforms())
    }
}

impl<U> ToUniforms for Option<U>
where
    U: ToUniforms,
{
    type Uniforms = UniformsOption<U::Uniforms>;

    fn to_uniforms(&self) -> Self::Uniforms {
        UniformsOption(self.as_ref().map(ToUniforms::to_uniforms))
    }
}

impl<'a> ToUniforms for &'a EmptyUniforms {
    type Uniforms = UniformsRef<Self>;

    fn to_uniforms(&self) -> Self::Uniforms {
        UniformsRef(self)
    }
}

impl<'a, 'n, T: AsUniformValue, R: Uniforms> ToUniforms for &'a UniformsStorage<'n, T, R> {
    type Uniforms = UniformsRef<Self>;

    fn to_uniforms(&self) -> Self::Uniforms {
        UniformsRef(self)
    }
}

pub struct UniformsPair<U1, U2>(U1, U2);

impl<U1, U2> Uniforms for UniformsPair<U1, U2>
where
    U1: Uniforms,
    U2: Uniforms,
{
    fn visit_values<'a, F>(&'a self, mut output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        self.0.visit_values(|name, value| output(name, value));
        self.1.visit_values(output);
    }
}

impl<'b, U1, U2> Uniforms for &'b UniformsPair<U1, U2>
where
    U1: Uniforms,
    U2: Uniforms,
{
    fn visit_values<'a, F>(&'a self, mut output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        self.0.visit_values(|name, value| output(name, value));
        self.1.visit_values(output);
    }
}

pub struct UniformsOption<U>(Option<U>);

impl<U> Uniforms for UniformsOption<U>
where
    U: Uniforms,
{
    fn visit_values<'a, F>(&'a self, output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        if let Some(uniforms) = self.0.as_ref() {
            uniforms.visit_values(output);
        }
    }
}

pub struct UniformsRef<U>(U);

impl<'b, U> Uniforms for UniformsRef<&'b U>
where
    U: Uniforms,
{
    fn visit_values<'a, F>(&'a self, output: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        self.0.visit_values(output);
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

pub trait StaticUniformType {
    const TYPE: UniformType;
}

impl StaticUniformType for bool {
    const TYPE: UniformType = UniformType::Bool;
}

impl StaticUniformType for f32 {
    const TYPE: UniformType = UniformType::Float;
}

impl StaticUniformType for [f32; 2] {
    const TYPE: UniformType = UniformType::FloatVec2;
}

impl StaticUniformType for [f32; 3] {
    const TYPE: UniformType = UniformType::FloatVec3;
}

impl StaticUniformType for [f32; 4] {
    const TYPE: UniformType = UniformType::FloatVec4;
}

impl StaticUniformType for [[f32; 2]; 2] {
    const TYPE: UniformType = UniformType::FloatMat2;
}

impl StaticUniformType for [[f32; 3]; 3] {
    const TYPE: UniformType = UniformType::FloatMat3;
}

impl StaticUniformType for [[f32; 4]; 4] {
    const TYPE: UniformType = UniformType::FloatMat4;
}
