//! This has become horrible.

use glium::uniforms::{
    AsUniformValue, EmptyUniforms, Sampler, UniformType, UniformValue, Uniforms, UniformsStorage,
};

pub trait HasUniforms<'u> {
    type Uniforms: Uniforms;
}

pub trait ToUniforms: for<'u> HasUniforms<'u> {
    fn to_uniforms(&self) -> <Self as HasUniforms<'_>>::Uniforms;
}

pub trait UniformInput: ToUniforms {
    fn uniform_input_defs() -> Vec<(String, UniformType)>;
}

pub trait InstanceInput: UniformInput {
    type Vertex: glium::vertex::Vertex + ToUniforms;

    fn to_vertex(&self) -> Self::Vertex;
}

pub trait CompatibleWith<U: ToUniforms>: ToUniforms {}

impl<'u> HasUniforms<'u> for () {
    type Uniforms = EmptyUniforms;
}

impl<'u, 'b, U> HasUniforms<'u> for &'b U
where
    U: HasUniforms<'u>,
{
    type Uniforms = U::Uniforms;
}

impl<'u, U1, U2> HasUniforms<'u> for (U1, U2)
where
    U1: HasUniforms<'u>,
    U2: HasUniforms<'u>,
{
    type Uniforms =
        UniformsPair<<U1 as HasUniforms<'u>>::Uniforms, <U2 as HasUniforms<'u>>::Uniforms>;
}

impl<'u, U1, U2, U3> HasUniforms<'u> for (U1, U2, U3)
where
    U1: HasUniforms<'u>,
    U2: HasUniforms<'u>,
    U3: HasUniforms<'u>,
{
    type Uniforms =
        UniformsPair<<U1 as HasUniforms<'u>>::Uniforms, <(U2, U3) as HasUniforms<'u>>::Uniforms>;
}

impl<'u, U1, U2, U3, U4> HasUniforms<'u> for (U1, U2, U3, U4)
where
    U1: HasUniforms<'u>,
    U2: HasUniforms<'u>,
    U3: HasUniforms<'u>,
    U4: HasUniforms<'u>,
{
    type Uniforms = UniformsPair<
        <U1 as HasUniforms<'u>>::Uniforms,
        <(U2, U3, U4) as HasUniforms<'u>>::Uniforms,
    >;
}

impl<'u, U> HasUniforms<'u> for Option<U>
where
    U: HasUniforms<'u>,
{
    type Uniforms = UniformsOption<U::Uniforms>;
}

impl<'u, 'a> HasUniforms<'u> for &'a EmptyUniforms {
    type Uniforms = UniformsRef<Self>;
}

impl<'u> HasUniforms<'u> for MyEmptyUniforms {
    type Uniforms = MyEmptyUniforms;
}

impl<'u, 'a, 'n, T: AsUniformValue, R: Uniforms> HasUniforms<'u> for &'a UniformsStorage<'n, T, R> {
    type Uniforms = UniformsRef<Self>;
}

impl<'u, 'a, 'n, T: AsUniformValue, R: Uniforms> HasUniforms<'u> for MyUniformsStorage<'n, T, R> {
    type Uniforms = Self;
}

impl ToUniforms for () {
    fn to_uniforms(&self) -> EmptyUniforms {
        EmptyUniforms
    }
}

impl<'b, U> ToUniforms for &'b U
where
    U: ToUniforms,
{
    fn to_uniforms(&self) -> <U as HasUniforms<'_>>::Uniforms {
        (*self).to_uniforms()
    }
}

impl<U1, U2> ToUniforms for (U1, U2)
where
    U1: ToUniforms,
    U2: ToUniforms,
{
    fn to_uniforms(&self) -> <Self as HasUniforms<'_>>::Uniforms {
        UniformsPair(self.0.to_uniforms(), self.1.to_uniforms())
    }
}

impl<U1, U2, U3> ToUniforms for (U1, U2, U3)
where
    U1: ToUniforms,
    U2: ToUniforms,
    U3: ToUniforms,
{
    fn to_uniforms(&self) -> <Self as HasUniforms<'_>>::Uniforms {
        UniformsPair(
            self.0.to_uniforms(),
            UniformsPair(self.1.to_uniforms(), self.2.to_uniforms()),
        )
    }
}

impl<U1, U2, U3, U4> ToUniforms for (U1, U2, U3, U4)
where
    U1: ToUniforms,
    U2: ToUniforms,
    U3: ToUniforms,
    U4: ToUniforms,
{
    fn to_uniforms(&self) -> <Self as HasUniforms<'_>>::Uniforms {
        UniformsPair(
            self.0.to_uniforms(),
            UniformsPair(
                self.1.to_uniforms(),
                UniformsPair(self.2.to_uniforms(), self.3.to_uniforms()),
            ),
        )
    }
}

impl<U> ToUniforms for Option<U>
where
    U: ToUniforms,
{
    fn to_uniforms(&self) -> <Self as HasUniforms<'_>>::Uniforms {
        UniformsOption(self.as_ref().map(ToUniforms::to_uniforms))
    }
}

impl<'a> ToUniforms for &'a EmptyUniforms {
    fn to_uniforms(&self) -> UniformsRef<Self> {
        UniformsRef(self)
    }
}

impl<'a, 'n, T, R> ToUniforms for &'a UniformsStorage<'n, T, R>
where
    T: AsUniformValue,
    R: Uniforms,
{
    fn to_uniforms(&self) -> UniformsRef<Self> {
        UniformsRef(self)
    }
}

impl<'a> ToUniforms for MyEmptyUniforms {
    fn to_uniforms(&self) -> Self {
        MyEmptyUniforms
    }
}

impl<'a, 'n, T, R> ToUniforms for MyUniformsStorage<'n, T, R>
where
    T: AsUniformValue,
    R: Uniforms,
    Self: Clone,
{
    fn to_uniforms(&self) -> Self {
        (*self).clone()
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

impl CompatibleWith<()> for () {}

impl<'b, U> CompatibleWith<&'b U> for &'b U where U: ToUniforms {}

impl<U1, U2> CompatibleWith<(U1, U2)> for (U1, U2)
where
    U1: ToUniforms,
    U2: ToUniforms,
{
}

impl<U> CompatibleWith<Option<U>> for Option<U> where U: ToUniforms {}

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

impl<'a> StaticUniformType for &'a glium::texture::Texture2d {
    const TYPE: UniformType = UniformType::Sampler2d;
}

impl<'a> StaticUniformType for Sampler<'a, glium::texture::Texture2d> {
    const TYPE: UniformType = UniformType::Sampler2d;
}

impl<'a> StaticUniformType for &'a glium::texture::DepthTexture2d {
    const TYPE: UniformType = UniformType::Sampler2d;
}

impl<'a> StaticUniformType for Sampler<'a, glium::texture::DepthTexture2d> {
    const TYPE: UniformType = UniformType::Sampler2d;
}

impl<'a> StaticUniformType for &'a glium::texture::CompressedSrgbTexture2d {
    const TYPE: UniformType = UniformType::Sampler2d;
}

impl<'a> StaticUniformType for Sampler<'a, glium::texture::CompressedSrgbTexture2d> {
    const TYPE: UniformType = UniformType::Sampler2d;
}

#[derive(Debug, Copy, Clone)]
pub struct MyEmptyUniforms;

impl Uniforms for MyEmptyUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, _: F) {}
}

#[derive(Clone)]
pub struct MyUniformsStorage<'n, T, R>
where
    T: AsUniformValue,
    R: Uniforms,
{
    name: &'n str,
    value: T,
    rest: R,
}

impl<'n, T> MyUniformsStorage<'n, T, MyEmptyUniforms>
where
    T: AsUniformValue,
{
    pub fn new(name: &'n str, value: T) -> Self {
        Self {
            name,
            value,
            rest: MyEmptyUniforms,
        }
    }
}

impl<'n, T, R> MyUniformsStorage<'n, T, R>
where
    T: AsUniformValue,
    R: Uniforms,
{
    pub fn add<U>(
        self,
        name: &'n str,
        value: U,
    ) -> MyUniformsStorage<'n, U, MyUniformsStorage<'n, T, R>>
    where
        U: AsUniformValue,
    {
        MyUniformsStorage {
            name,
            value,
            rest: self,
        }
    }
}

impl<'n, T, R> Uniforms for MyUniformsStorage<'n, T, R>
where
    T: AsUniformValue,
    R: Uniforms,
{
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut output: F) {
        output(self.name, self.value.as_uniform_value());
        self.rest.visit_values(output);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EmptyVertexHack {
    __rendology_dummy_field: f32,
}

glium::implement_vertex!(EmptyVertexHack, __rendology_dummy_field);

impl<'u> HasUniforms<'u> for EmptyVertexHack {
    type Uniforms = EmptyUniforms;
}

impl ToUniforms for EmptyVertexHack {
    fn to_uniforms(&self) -> EmptyUniforms {
        EmptyUniforms
    }
}

impl InstanceInput for () {
    type Vertex = EmptyVertexHack;

    fn to_vertex(&self) -> EmptyVertexHack {
        EmptyVertexHack {
            __rendology_dummy_field: 42.0,
        }
    }
}

#[macro_export]
macro_rules! plain_uniforms {
    () => {
        $crate::shader::input::MyEmptyUniforms
    };
    ($field:ident: $value:expr) => {
        $crate::shader::input::MyUniformsStorage::new(stringify!($field), $value)
    };
    ($field1:ident: $value1:expr, $($field:ident: $value:expr),+) => {
        {
            let uniforms =
                $crate::shader::input::MyUniformsStorage::new(stringify!($field1), $value1);
            $(
                let uniforms = uniforms.add(stringify!($field), $value);
            )+
            uniforms
        }
    };
    ($($field:ident: $value:expr),*,) => {
        plain_uniforms!($($field: $value),*)
    };
}

#[macro_export]
macro_rules! impl_uniform_input_detail {
    (
        $ty:ident,
        $this:ident => { $( $field:ident: $type:ty = $value:expr, )* } $(,)?
    ) => {
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

        impl<'u> $crate::shader::input::HasUniforms<'u> for $ty {
            type Uniforms = MyUniforms;
        }

        impl $crate::shader::ToUniforms for $ty {
            fn to_uniforms(&$this) -> MyUniforms {
                MyUniforms {
                    $(
                        $field: $value.into(),
                    )*
                }
            }
        }

        impl $crate::shader::UniformInput for $ty {
            fn uniform_input_defs() -> Vec<(String, glium::uniforms::UniformType)> {
                vec![
                    $(
                        (
                            stringify!($field).to_string(),
                            <$type as $crate::shader::input::StaticUniformType>::TYPE
                        ),
                    )*
                ]
            }
        }

        impl $crate::shader::input::CompatibleWith<$ty> for $ty {}
    }
}

#[macro_export]
macro_rules! impl_uniform_input {
    (
        $ty:ident,
        $this:ident => { $( $field:ident: $type:ty = $value:expr, )* } $(,)?
    ) => {
        const _: () = {
            $crate::impl_uniform_input_detail!(
                $ty,
                $this => { $($field: $type = $value, )* }
            );

            ()
        };
    };
    (
        $ty:ident<$life:lifetime>,
        $this:ident => { $( $field:ident: $type:ty = $value:expr, )* } $(,)?
    ) => {
        const _: () = {
            #[derive(Copy, Clone, Debug)]
            pub struct MyUniforms<$life> {
                $(
                    $field: $type,
                )*
            }

            impl<'u> glium::uniforms::Uniforms for MyUniforms<'u> {
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

            impl<'a, 'u> $crate::shader::input::HasUniforms<'u> for $ty<'a> {
                type Uniforms = MyUniforms<'a>;
            }

            impl<'a> $crate::shader::ToUniforms for $ty<'a> {
                fn to_uniforms(&$this) -> MyUniforms<'a> {
                    MyUniforms {
                        $(
                            $field: $value,
                        )*
                    }
                }
            }

            impl<$life> $crate::shader::UniformInput for $ty<$life> {
                fn uniform_input_defs() -> Vec<(String, glium::uniforms::UniformType)> {
                    vec![
                        $(
                            (
                                stringify!($field).to_string(),
                                <$type as $crate::shader::input::StaticUniformType>::TYPE
                            ),
                        )*
                    ]
                }
            }

            impl<'a> $crate::shader::input::CompatibleWith<$ty<'static>> for $ty<'a> {}

            ()
        };
    }
}

#[macro_export]
macro_rules! impl_instance_input {
    (
        $ty:ident,
        $this:ident => { $( $field:ident: $type:ty = $value:expr, )* } $(,)?
    ) => {
        const _: () = {
            $crate::impl_uniform_input_detail!(
                $ty,
                $this => { $($field: $type = $value, )* }
            );

            use glium::implement_vertex;
            implement_vertex!(MyUniforms, $($field,)*);

            impl<'u> $crate::shader::input::HasUniforms<'u> for MyUniforms {
                type Uniforms = Self;
            }

            impl $crate::shader::ToUniforms for MyUniforms {
                fn to_uniforms(&self) -> Self {
                    self.clone()
                }
            }

            impl $crate::shader::InstanceInput for $ty {
                type Vertex = MyUniforms;

                fn to_vertex(&self) -> Self::Vertex {
                    $crate::shader::ToUniforms::to_uniforms(self)
                }
            }

            ()
        };
    }
}
