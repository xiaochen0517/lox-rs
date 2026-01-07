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
                        fn $visitor_fn(&mut self, [<$ast_name:lower>]: &$struct_name) -> Result<OptionLoxType, LoxReturn>;
                    )*
                }

                pub trait $ast_name:Debug + Send + Sync /*+ SimpleHash*/ {
                    fn accept(&self, visitor: &mut dyn [<$ast_name Visitor>]) -> Result<OptionLoxType, LoxReturn>;
                    fn get_type(&self) -> [<$ast_name Type>];
                    fn as_any(&self) -> &dyn std::any::Any;
                    fn box_clone(&self) -> Box<dyn $ast_name>;
                }

                impl Clone for Box<dyn $ast_name> {
                    fn clone(&self) -> Box<dyn $ast_name> {
                        self.box_clone()
                    }
                }

                /*impl SimpleHash for Box<dyn $ast_name> {
                    pub fn get_hash(&self) -> String {
                        (**self).get_hash()
                    }
                }*/

                $(

                    #[derive(Debug, Clone)]
                    pub struct $struct_name {
                        $(pub $field_name: $field_type),*
                    }

                    impl $ast_name for $struct_name {

                        fn accept(&self, visitor: &mut dyn [<$ast_name Visitor>]) -> Result<OptionLoxType, LoxReturn> {
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

                        /*fn get_hash(&self) -> String {
                            // 获取到所有字段的地址，并进行合并后返回
                            let mut hash_str = String::new();
                            $(
                                hash_str += &self.$field_name.get_hash(&mut hasher);
                            )*
                            hash_str
                        }*/

                    }
/*
                    impl SimpleHash for $struct_name {
                        fn get_hash(&self) -> String {
                            // 获取到所有字段的地址，并进行合并后返回
                            let mut hash_str = String::new();
                            $(
                                hash_str += &self.$field_name.get_hash();
                            )*
                            hash_str
                        }
                    }*/

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
