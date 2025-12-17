use paste::paste;

#[macro_export]
macro_rules! generate_ast {
    (
        $(
            $ast_name:ident {
                $(
                    $struct_name:ident($visitor_fn:ident) {
                        $($field_name:ident : $field_type:ty),* $(,)?
                    }
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        paste! {
            $(

                pub enum [<$ast_name Type>] {
                    $(
                        $struct_name,
                    )*
                }

                pub trait [<$ast_name Visitor>] {
                    $(
                        fn $visitor_fn(&self, [<$ast_name:lower>]: &$struct_name) -> Option<LoxType>;
                    )*
                }

                pub trait $ast_name:Debug {
                    fn accept(&self, visitor: &dyn [<$ast_name Visitor>]) -> Option<LoxType>;
                    fn get_type(&self) -> [<$ast_name Type>];
                }

                $(

                    #[derive(Debug)]
                    pub struct $struct_name {
                        $(pub $field_name: $field_type),*
                    }

                    impl $ast_name for $struct_name {

                        fn accept(&self, visitor: &dyn [<$ast_name Visitor>]) -> Option<LoxType> {
                            visitor.$visitor_fn(self)
                        }

                        fn get_type(&self) -> [<$ast_name Type>] {
                            [<$ast_name Type>]::$struct_name
                        }
                    }

                    impl $struct_name {
                        pub fn new($($field_name : $field_type),*) -> Self {
                            $struct_name { $($field_name),* }
                        }
                    }
                )*
            )*
        }
    };
}
