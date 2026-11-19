/// The AST's for the ZEA language
pub mod ast;
mod typecheck;
pub use typecheck::FudgeTypeChecker;
pub use typecheck::typecheck_module;
mod impls;
pub mod visualisation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, arbitrary::Arbitrary, PartialOrd, Ord)]
pub enum ScalarTypeWidth {
    _8,
    _16,
    _32,
    _64,
}

impl ScalarTypeWidth {
    pub const fn num_bits(self) -> u8 {
        match self {
            ScalarTypeWidth::_8 => 8,
            ScalarTypeWidth::_16 => 16,
            ScalarTypeWidth::_32 => 32,
            ScalarTypeWidth::_64 => 64,
        }
    }
    pub const fn num_bytes(self) -> u8 {
        match self {
            ScalarTypeWidth::_8 => 1,
            ScalarTypeWidth::_16 => 2,
            ScalarTypeWidth::_32 => 4,
            ScalarTypeWidth::_64 => 8,
        }
    }
}

impl TryFrom<u8> for ScalarTypeWidth {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            8 => Ok(Self::_8),
            16 => Ok(Self::_16),
            32 => Ok(Self::_32),
            64 => Ok(Self::_64),
            _ => Err(value),
        }
    }
}

/// Supply a struct's Name to Implement the NodeLabeler trait for it.
/// This macro requires the struct to have a `label: usize` field.
///
/// Additionaly a lebel suffix mat be provided to customize
/// [`NodeLabeler::next_label_with_ident_string`]
macro_rules! impl_nodelabeler {
    ($typ:ident, $label:literal) => {
        impl NodeLabeler for $typ {
            fn from_labeler(mut other_generator: impl NodeLabeler) -> Self {
                Self {
                    label: other_generator.next_id().0,
                }
            }
            fn next_id(&mut self) -> NodeId {
                let label = self.label;
                self.label += 1;
                NodeId(label)
            }
            fn next_label_with_ident_string(&mut self) -> (NodeId, String) {
                let label = self.next_id();
                (label, format!(concat!("__", $label, "{}"), label))
            }
        }
    };
    ($typ:ident) => {
        impl NodeLabeler for $typ {
            fn from_labeler(mut other_generator: impl NodeLabeler) -> Self {
                Self {
                    label: other_generator.next_id().0,
                }
            }
            fn next_id(&mut self) -> NodeId {
                let label = self.label;
                self.label += 1;
                NodeId(label)
            }
            fn next_label_with_ident_string(&mut self) -> (NodeId, String) {
                let label = self.next_id();
                (label, format!("__synthetic{}", label))
            }
        }
    };
}
pub(crate) use impl_nodelabeler;

pub mod attributes {
    //! This module contains compiler directives and annotations for language items
    //! Some Language items like structs, functions, modules etc.
    //! Have a set of default attributes,
    //! Like if they should be a candidate for dead code elimination,
    //! if a function is hot or cold, etc.
    //!
    //! Fudge Attributes in some cases cascade into children items.
    //! As such, deriving the attributes for some item goes as follows:
    //! - start with the default attributes for this item
    //! - inherit applicable attributes from the parent item
    //! - modify any attributes that where specified using supplied directives.
    //!
    //! For some items you may see some attributes prefixed with `cascade_`,
    //! this means that the item itself does not use that attribute,
    //! but solely lets its children items inherit from it.
    #![allow(unused)]
    use arbitrary::Arbitrary;
    pub trait AttrListLike: Default {
        fn from_attr_list(list: Vec<Attribute>) -> Self {
            let mut s = Self::default();
            s.update_from_attr_list(list);
            s
        }
        fn update_from_attr_list(&mut self, list: Vec<Attribute>);
        fn as_attr_list(&self) -> Vec<Attribute>;
    }
    #[derive(Debug, Clone, Arbitrary)]
    pub struct FudgeModuleAttributes {
        /// Should chid items be applicable for dead code elimination?
        cascade_keep: KeepAttribute,
    }

    impl Default for FudgeModuleAttributes {
        fn default() -> Self {
            Self {
                cascade_keep: KeepAttribute::EliminateUnused,
            }
        }
    }
    impl AttrListLike for FudgeModuleAttributes {
        fn update_from_attr_list(&mut self, list: Vec<Attribute>) {
            for attr in list {
                if let Attribute::Keep(keep) = attr {
                    self.cascade_keep = keep;
                    return;
                }
            }
        }
        fn as_attr_list(&self) -> Vec<Attribute> {
            vec![Attribute::Keep(self.cascade_keep)]
        }
    }

    #[derive(Debug, Clone, Arbitrary)]
    pub struct IPRStructAttributes {
        /// How should the fields of this struct be ordered?
        field_order: FieldOrderAttribute,
        alignment: AlignmentAttribute,
        keep: KeepAttribute,
    }

    impl Default for IPRStructAttributes {
        fn default() -> Self {
            Self {
                field_order: FieldOrderAttribute::Naive,
                alignment: AlignmentAttribute::Inferred,
                keep: KeepAttribute::Keep,
            }
        }
    }
    impl AttrListLike for IPRStructAttributes {
        fn update_from_attr_list(&mut self, list: Vec<Attribute>) {
            for attr in list {
                match attr {
                    Attribute::Align(type_alignment) => {
                        self.alignment = type_alignment;
                    }
                    Attribute::Ordering(struct_field_ordering) => {
                        self.field_order = struct_field_ordering;
                    }
                    Attribute::Keep(keep) => {
                        self.keep = keep;
                    }
                    _ => {}
                }
            }
        }
        fn as_attr_list(&self) -> Vec<Attribute> {
            vec![
                Attribute::Ordering(self.field_order),
                Attribute::Align(self.alignment),
                Attribute::Keep(self.keep),
            ]
        }
    }
    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Hash)]
    pub struct FudgeFunctionAttributes {
        /// should this function be inlined?
        inline: InlineAttribute,
        /// is this function called often?
        temperature: TemperatureAttribute,
        /// Should dead code elimination
        /// be performed on this function?
        keep: KeepAttribute,
        /// Must the caller use the return value?
        must_use: MustUseAttribute,
    }

    impl Default for FudgeFunctionAttributes {
        fn default() -> Self {
            Self {
                inline: InlineAttribute::AsNecessary,
                temperature: TemperatureAttribute::LukeWarm,
                keep: KeepAttribute::Keep,
                must_use: MustUseAttribute::MayDiscard,
            }
        }
    }
    impl AttrListLike for FudgeFunctionAttributes {
        fn update_from_attr_list(&mut self, list: Vec<Attribute>) {
            for attr in list {
                match attr {
                    Attribute::Inline(inline_attribute) => {
                        self.inline = inline_attribute;
                    }
                    Attribute::Temperature(temperature_attribute) => {
                        self.temperature = temperature_attribute;
                    }
                    Attribute::MustUse(must_use_attribute) => {
                        self.must_use = must_use_attribute;
                    }
                    Attribute::Keep(keep_attribute) => {
                        self.keep = keep_attribute;
                    }
                    _ => {}
                }
            }
        }
        fn as_attr_list(&self) -> Vec<Attribute> {
            vec![
                Attribute::Inline(self.inline),
                Attribute::Temperature(self.temperature),
                Attribute::Keep(self.keep),
                Attribute::MustUse(self.must_use),
            ]
        }
    }

    #[derive(Default, Debug, Copy, Clone, Arbitrary, PartialEq, Eq)]
    pub struct IPRBlockAttributes {
        temperature: TemperatureAttribute,
    }
    impl AttrListLike for IPRBlockAttributes {
        fn update_from_attr_list(&mut self, list: Vec<Attribute>) {
            for attr in list {
                if let Attribute::Temperature(t) = attr {
                    self.temperature = t;
                    return;
                }
            }
        }
        fn as_attr_list(&self) -> Vec<Attribute> {
            vec![Attribute::Temperature(self.temperature)]
        }
    }
    #[derive(Default, Debug, Copy, Clone, Arbitrary, PartialEq, Eq)]
    pub struct IPRBranchAttributes {
        taken_path_temperature: TemperatureAttribute,
    }

    impl IPRBranchAttributes {
        pub fn then_path_temperature(&self) -> TemperatureAttribute {
            self.taken_path_temperature
        }
        pub fn else_path_temperature(&self) -> TemperatureAttribute {
            match self.taken_path_temperature {
                TemperatureAttribute::Hot => TemperatureAttribute::Cold,
                TemperatureAttribute::Cold => TemperatureAttribute::Hot,
                TemperatureAttribute::LukeWarm => TemperatureAttribute::LukeWarm,
            }
        }
    }
    impl AttrListLike for IPRBranchAttributes {
        fn update_from_attr_list(&mut self, list: Vec<Attribute>) {
            for attr in list {
                if let Attribute::Temperature(t) = attr {
                    self.taken_path_temperature = t;
                    return;
                }
            }
        }
        fn as_attr_list(&self) -> Vec<Attribute> {
            vec![Attribute::Temperature(self.taken_path_temperature)]
        }
    }

    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Default, Hash)]
    pub enum TemperatureAttribute {
        Hot,
        Cold,
        #[default]
        LukeWarm,
    }

    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Default, Hash)]
    pub enum FieldOrderAttribute {
        #[default]
        Naive,
        Descending,
    }
    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Default, Hash)]
    pub enum AlignmentAttribute {
        #[default]
        Inferred,
        Explicit(usize),
    }

    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Default, Hash)]
    pub enum InlineAttribute {
        Always,
        #[default]
        AsNecessary,
        Never,
    }
    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Default, Hash)]
    pub enum KeepAttribute {
        #[default]
        Keep,
        EliminateUnused,
    }
    #[derive(Debug, Copy, Clone, Arbitrary, PartialEq, Eq, Default, Hash)]
    pub enum MustUseAttribute {
        #[default]
        MayDiscard,
        MustUse,
    }
    pub enum Attribute {
        Inline(InlineAttribute),
        Align(AlignmentAttribute),
        Ordering(FieldOrderAttribute),
        Temperature(TemperatureAttribute),
        MustUse(MustUseAttribute),
        Keep(KeepAttribute),
    }
    pub enum AttributeParseError {
        InvalidKey(String),
        InvalidValueForKey(String, String),
        MissingValue(String),
    }

    static ALLOWED_VALUELESS_KEYS: &[&str] = &["inline", "hot", "cold", "must-use", "keep-unused"];
    static ALLOWED_KV_PAIRS: &[(&str, &str)] = &[
        ("inline", "never"),
        ("inline", "always"),
        ("fields", "descending"),
        ("fields", "c"),
    ];
    pub fn parse_raw_attribute(
        key: String,
        value: Option<String>,
    ) -> Result<Attribute, AttributeParseError> {
        match value {
            Some(value) => parse_raw_keyful_attribute(key, value),
            None => parse_raw_keyless_attribute(key),
        }
    }

    fn parse_raw_keyless_attribute(key: String) -> Result<Attribute, AttributeParseError> {
        match key.as_str() {
            "inline" => Ok(Attribute::Inline(InlineAttribute::Always)),
            "hot" => Ok(Attribute::Temperature(TemperatureAttribute::Hot)),
            "cold" => Ok(Attribute::Temperature(TemperatureAttribute::Cold)),
            "must-use" => Ok(Attribute::MustUse(MustUseAttribute::MayDiscard)),
            "keep-unused" => Ok(Attribute::Keep(KeepAttribute::Keep)),
            _ => Err(AttributeParseError::InvalidKey(key)),
        }
    }

    fn parse_raw_keyful_attribute(
        key: String,
        value: String,
    ) -> Result<Attribute, AttributeParseError> {
        match (key.as_str(), value.as_str()) {
            ("inline", "always") => Ok(Attribute::Inline(InlineAttribute::Always)),
            ("inline", "never") => Ok(Attribute::Inline(InlineAttribute::Never)),
            ("inline", _) => Err(AttributeParseError::InvalidValueForKey(key, value)),
            ("fields", "desc") => Ok(Attribute::Ordering(FieldOrderAttribute::Descending)),
            ("fields", "c") => Ok(Attribute::Ordering(FieldOrderAttribute::Naive)),
            ("fields", _) => Err(AttributeParseError::InvalidValueForKey(key, value)),
            _ => Err(AttributeParseError::InvalidKey(key)),
        }
    }
}
