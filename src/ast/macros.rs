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

                #[derive(Debug, Clone)]
                pub enum [<$ast_name Type>] {
                    $(
                        $struct_name,
                    )*
                }

                pub trait [<$ast_name Visitor>] {
                    $(
                        fn $visitor_fn(&mut self, [<$ast_name:lower>]: &$struct_name) -> Result<Option<LoxType>, LoxReturn>;
                    )*
                }

                pub trait $ast_name:Debug + Send + Sync {
                    fn accept(&self, visitor: &mut dyn [<$ast_name Visitor>]) -> Result<Option<LoxType>, LoxReturn>;
                    fn get_type(&self) -> [<$ast_name Type>];
                    fn as_any(&self) -> &dyn std::any::Any;
                    fn box_clone(&self) -> Box<dyn $ast_name>;
                }

                impl Clone for Box<dyn $ast_name> {
                    fn clone(&self) -> Box<dyn $ast_name> {
                        self.box_clone()
                    }
                }

                $(

                    #[derive(Debug, Clone)]
                    pub struct $struct_name {
                        $(pub $field_name: $field_type),*
                    }

                    impl $ast_name for $struct_name {

                        fn accept(&self, visitor: &mut dyn [<$ast_name Visitor>]) -> Result<Option<LoxType>, LoxReturn> {
                            visitor.$visitor_fn(self)
                        }

                        fn get_type(&self) -> [<$ast_name Type>] {
                            [<$ast_name Type>]::$struct_name
                        }

                        fn as_any(&self) -> &dyn std::any::Any {
                            self
                        }

                        fn box_clone(&self) -> Box<dyn $ast_name> {
                            Box::new(self.clone())
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
