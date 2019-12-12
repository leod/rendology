use crate::shader::{FragmentOutDef, FragmentOutQualifier, Type, VertexOutDef, VertexOutQualifier};

pub const V_POS: (&str, VertexOutDef) = (
    "gl_Position",
    VertexOutDef(Type::FloatVec4, VertexOutQualifier::Smooth),
);

pub const V_WORLD_NORMAL: (&str, VertexOutDef) = (
    "v_world_normal",
    VertexOutDef(Type::FloatVec3, VertexOutQualifier::Smooth),
);

pub const V_WORLD_POS: (&str, VertexOutDef) = (
    "v_world_pos",
    VertexOutDef(Type::FloatVec4, VertexOutQualifier::Smooth),
);

pub const V_COLOR: (&str, VertexOutDef) = (
    "v_color",
    VertexOutDef(Type::FloatVec4, VertexOutQualifier::Smooth),
);

pub const V_TEX_COORD: (&str, VertexOutDef) = (
    "v_tex_coord",
    VertexOutDef(Type::FloatVec2, VertexOutQualifier::Smooth),
);

pub const F_COLOR: (&str, FragmentOutDef) = (
    "f_color",
    FragmentOutDef(Type::FloatVec4, FragmentOutQualifier::Yield),
);

pub const F_FRAGMENT_DEPTH: (&str, FragmentOutDef) = (
    "f_fragment_depth",
    FragmentOutDef(Type::Float, FragmentOutQualifier::Yield),
);

pub const F_SHADOW: (&str, FragmentOutDef) = (
    "f_shadow",
    FragmentOutDef(Type::Float, FragmentOutQualifier::Local),
);
